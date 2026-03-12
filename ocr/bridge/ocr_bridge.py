#!/usr/bin/env python3
import argparse
from collections import defaultdict
from concurrent.futures import ThreadPoolExecutor
import json
import os
import re
import sys
import unicodedata
from pathlib import Path

import cv2
import numpy as np
from PIL import Image
import pytesseract
from pytesseract import Output

# Lazy import for pix2tex to avoid slow startup when not needed
_latex_model = None


def _resolve_ocr_workers() -> int:
    """Resolve OCR worker count from env (default: physical CPU-friendly)."""
    raw = os.environ.get("DOCSTRUCT_OCR_WORKERS", "")
    if raw:
        try:
            return max(1, int(raw))
        except ValueError:
            pass
    cpu_count = os.cpu_count() or 1
    return max(1, min(cpu_count, 8))


def _cuda_enabled() -> bool:
    return os.environ.get("DOCSTRUCT_OCR_USE_CUDA", "0") == "1"


def _to_grayscale(img: np.ndarray) -> np.ndarray:
    if len(img.shape) == 2:
        return img

    if _cuda_enabled() and hasattr(cv2, "cuda"):
        try:
            gpu = cv2.cuda_GpuMat()
            gpu.upload(img)
            gray_gpu = cv2.cuda.cvtColor(gpu, cv2.COLOR_BGR2GRAY)
            return gray_gpu.download()
        except Exception:
            pass

    return cv2.cvtColor(img, cv2.COLOR_BGR2GRAY)


def _otsu_threshold(gray: np.ndarray) -> np.ndarray:
    if _cuda_enabled() and hasattr(cv2, "cuda"):
        try:
            gpu = cv2.cuda_GpuMat()
            gpu.upload(gray)
            # Otsu is not available in cv2.cuda; use a fixed threshold from Otsu estimate.
            otsu_val, _ = cv2.threshold(gray, 0, 255, cv2.THRESH_BINARY + cv2.THRESH_OTSU)
            thresh_gpu = cv2.cuda.threshold(gpu, otsu_val, 255, cv2.THRESH_BINARY)[1]
            return thresh_gpu.download()
        except Exception:
            pass

    _, roi_bin = cv2.threshold(gray, 0, 255, cv2.THRESH_BINARY + cv2.THRESH_OTSU)
    return roi_bin


def combine_hangul_jamos(text: str) -> str:
    """
    Combine separated Hangul jamos into complete syllables.
    
    Korean characters can be decomposed into jamos (ᄀ, ᅡ, ᆨ, etc.)
    This function recombines them into complete syllables (가, 각, etc.)
    """
    def is_hangul_jamo_or_compat(ch: str) -> bool:
        code = ord(ch)
        return (
            0x1100 <= code <= 0x11FF or
            0x3130 <= code <= 0x318F or
            0xA960 <= code <= 0xA97F or
            0xD7B0 <= code <= 0xD7FF
        )

    # NFKC converts compatibility jamo (ㄱㅏ) to canonical forms.
    normalized = unicodedata.normalize("NFKC", text)
    chars = list(normalized)
    compact = []

    for i, c in enumerate(chars):
        if c.isspace():
            prev = next((p for p in reversed(chars[:i]) if not p.isspace()), None)
            nxt = next((n for n in chars[i + 1:] if not n.isspace()), None)
            if prev and nxt and is_hangul_jamo_or_compat(prev) and is_hangul_jamo_or_compat(nxt):
                continue
        compact.append(c)

    # NFC recomposes canonical jamo sequences to complete Hangul syllables.
    return unicodedata.normalize("NFC", "".join(compact))


def korean_counts(text: str) -> tuple[int, int]:
    syllables = 0
    jamos = 0
    for ch in text:
        code = ord(ch)
        if 0xAC00 <= code <= 0xD7A3:
            syllables += 1
        elif (
            0x1100 <= code <= 0x11FF
            or 0x3130 <= code <= 0x318F
            or 0xA960 <= code <= 0xA97F
            or 0xD7B0 <= code <= 0xD7FF
        ):
            jamos += 1
    return syllables, jamos


def count_hanja(text: str) -> int:
    count = 0
    for ch in text:
        code = ord(ch)
        if (
            0x3400 <= code <= 0x4DBF
            or 0x4E00 <= code <= 0x9FFF
            or 0xF900 <= code <= 0xFAFF
        ):
            count += 1
    return count


def is_degraded_korean_text(text: str) -> bool:
    syllables, jamos = korean_counts(text)
    total = syllables + jamos
    return total >= 6 and jamos > syllables * 2


def normalize_ocr_text(text: str) -> str:
    text = combine_hangul_jamos(text)
    # Tesseract(eng+kor) can misread Korean/Math glyphs as Hanja.
    # For this pipeline, treat Hanja codepoints as OCR noise and drop them.
    text = "".join(
        ch
        for ch in text
        if not (
            0x3400 <= ord(ch) <= 0x4DBF
            or 0x4E00 <= ord(ch) <= 0x9FFF
            or 0xF900 <= ord(ch) <= 0xFAFF
        )
    )
    text = re.sub(r"\s+", " ", text).strip()
    return text


def is_single_hangul_syllable(token: str) -> bool:
    return len(token) == 1 and bool(re.fullmatch(r"[가-힣]", token))


def collapse_short_hangul_runs(text: str, min_run: int = 3, max_run: int = 4) -> str:
    """Collapse short runs like '문 서 입 니다' -> '문서입니다' without over-merging long phrases."""
    tokens = text.split()
    if not tokens:
        return text

    out = []
    i = 0
    n = len(tokens)
    while i < n:
        j = i
        while j < n and is_single_hangul_syllable(tokens[j]):
            j += 1
        run_len = j - i
        if min_run <= run_len <= max_run:
            out.append("".join(tokens[i:j]))
            i = j
            continue
        out.extend(tokens[i:j] if run_len > 0 else [tokens[i]])
        i = j if run_len > 0 else i + 1

    return " ".join(out)


def fix_common_korean_split_endings(text: str) -> str:
    """Fix frequent OCR spacing artifacts in Korean polite endings."""
    fixes = [
        (r"([가-힣])입\s+니다", r"\1입니다"),
        (r"([가-힣])합\s+니다", r"\1합니다"),
        (r"([가-힣])습\s+니다", r"\1습니다"),
        (r"([가-힣])됩\s+니다", r"\1됩니다"),
        (r"([가-힣])입\s+니까", r"\1입니까"),
        (r"([가-힣])할\s+수", r"\1할 수"),
    ]
    out = text
    for pattern, repl in fixes:
        out = re.sub(pattern, repl, out)
    return out


def is_probably_noise_text(text: str) -> bool:
    norm = normalize_ocr_text(text)
    if len(norm) <= 1:
        return True

    alnum_or_korean = len(re.findall(r"[A-Za-z0-9가-힣]", norm))
    if alnum_or_korean <= 1:
        return True

    compact = re.sub(r"\s+", "", norm)
    if len(compact) >= 6 and len(set(compact)) <= 2:
        return True

    syllables, _ = korean_counts(norm)
    hanja = count_hanja(norm)
    if syllables >= 3 and hanja >= 2:
        return True

    if is_degraded_korean_text(norm):
        return True

    return False


def merge_adjacent_text_blocks(tokens: list[dict]) -> list[dict]:
    """Merge nearby text blocks into line-like segments to reduce fragmentation.
    
    IMPORTANT: Preserves input order - assumes tokens are already sorted in reading order.
    """
    text_blocks = [t for t in tokens if t.get("block_type") == "text"]
    other_blocks = [t for t in tokens if t.get("block_type") != "text"]

    # Don't resort! Preserve the reading order from column detection
    # text_blocks.sort(key=lambda t: (t["bbox"][1], t["bbox"][0]))  # REMOVED
    merged = []

    for tok in text_blocks:
        text = normalize_ocr_text(tok.get("text", ""))
        if not text:
            continue
        tok["text"] = text
        tok_conf = float(tok.get("confidence", 0.5))

        if not merged:
            tok["confidence"] = tok_conf
            merged.append(tok)
            continue

        prev = merged[-1]
        if prev.get("block_type") != "text":
            tok["confidence"] = tok_conf
            merged.append(tok)
            continue

        ax0, ay0, ax1, ay1 = prev["bbox"]
        bx0, by0, bx1, by1 = tok["bbox"]
        ah = max(1.0, ay1 - ay0)
        bh = max(1.0, by1 - by0)
        a_cy = (ay0 + ay1) * 0.5
        b_cy = (by0 + by1) * 0.5

        same_line = abs(a_cy - b_cy) <= max(8.0, 0.45 * max(ah, bh))
        horizontal_gap = bx0 - ax1
        
        # Don't merge blocks that are too far apart (likely different columns)
        # For multi-column layouts, typical column gap is 40-100px at 200 DPI
        # Use conservative threshold to avoid merging across columns
        max_merge_distance = min(max(24.0, 1.1 * max(ah, bh)), 80.0)
        close_horizontally = 0 <= horizontal_gap <= max_merge_distance

        # Merge if two OCR snippets likely belong to the same visual line.
        if same_line and close_horizontally:
            merged_text = normalize_ocr_text(f"{prev['text']} {text}")
            prev["text"] = fix_common_korean_split_endings(collapse_short_hangul_runs(merged_text))
            prev["bbox"] = [min(ax0, bx0), min(ay0, by0), max(ax1, bx1), max(ay1, by1)]
            prev["confidence"] = max(float(prev.get("confidence", 0.5)), tok_conf)
        else:
            tok["confidence"] = tok_conf
            tok["text"] = fix_common_korean_split_endings(collapse_short_hangul_runs(tok["text"]))
            merged.append(tok)

    for tok in merged:
        if tok.get("block_type") == "text":
            tok["text"] = fix_common_korean_split_endings(collapse_short_hangul_runs(tok.get("text", "")))

    return merged + other_blocks


def post_process_tokens(tokens: list[dict]) -> list[dict]:
    cleaned = []
    seen = set()
    for tok in tokens:
        text = normalize_ocr_text(tok.get("text", ""))
        tok["text"] = text
        if not text:
            continue
        if tok.get("block_type") == "text" and is_probably_noise_text(text):
            continue
        x0, y0, x1, y1 = tok["bbox"]
        sig = (tok.get("block_type"), round(x0, 1), round(y0, 1), round(x1, 1), round(y1, 1), text)
        if sig in seen:
            continue
        seen.add(sig)
        cleaned.append(tok)

    return merge_adjacent_text_blocks(cleaned)


def get_latex_model():
    """Lazy load LaTeX OCR model."""
    global _latex_model
    if _latex_model is None:
        try:
            from pix2tex.cli import LatexOCR
            _latex_model = LatexOCR()
        except Exception as e:
            print(f"Warning: Could not load LaTeX OCR model: {e}", file=sys.stderr)
            _latex_model = False
    return _latex_model if _latex_model is not False else None


def detect_blocks(image_path: Path, min_area: int = 2000, merge_kernel: tuple = (15, 10)) -> list[dict]:
    """Detect text blocks in the image using morphological operations.
    
    Args:
        min_area: Minimum block area in pixels
        merge_kernel: Kernel size for dilating to merge nearby text
                     Reduced from (25,15) to (15,10) to avoid over-merging equations
    """
    img = cv2.imread(str(image_path))
    gray = _to_grayscale(img)
    thresh = cv2.adaptiveThreshold(
        gray, 255, cv2.ADAPTIVE_THRESH_GAUSSIAN_C, cv2.THRESH_BINARY_INV, 35, 15
    )
    kernel = cv2.getStructuringElement(cv2.MORPH_RECT, merge_kernel)
    dilated = cv2.dilate(thresh, kernel, iterations=1)  # Reduced iterations to preserve separation
    contours, _ = cv2.findContours(dilated, cv2.RETR_EXTERNAL, cv2.CHAIN_APPROX_SIMPLE)
    
    blocks = []
    for contour in contours:
        x, y, w, h = cv2.boundingRect(contour)
        if w * h < min_area:
            continue
        blocks.append({"x": x, "y": y, "w": w, "h": h})
    
    blocks = sort_blocks_by_reading_order(blocks)
    return blocks


def detect_column_layout(blocks: list[dict]) -> tuple[bool, float]:
    """Detect if blocks are in multi-column layout.
    
    Returns:
        (is_multi_column, divider_x): True if 2+ columns detected, and x-coordinate dividing them
    """
    if len(blocks) < 4:
        return False, 0.0
    
    # Calculate page width from rightmost block
    page_width = max(b["x"] + b["w"] for b in blocks)
    if page_width < 100:
        return False, 0.0
    
    # Try middle divider
    middle_x = page_width / 2.0
    
    # Count blocks in left and right halves based on their center
    left_blocks = [b for b in blocks if (b["x"] + b["w"] / 2) < middle_x]
    right_blocks = [b for b in blocks if (b["x"] + b["w"] / 2) >= middle_x]
    
    # For column detection, ignore blocks that span across the middle (titles, headers)
    # A block is considered "spanning" if it extends significantly beyond its column's expected range
    left_column_max = page_width * 0.45  # Left column should end around 45% of page width
    right_column_min = page_width * 0.55  # Right column should start around 55% of page width
    
    # Filter blocks that stay within their column boundaries
    left_blocks_in_column = [b for b in left_blocks if b["x"] + b["w"] < left_column_max + 50]
    right_blocks_in_column = [b for b in right_blocks if b["x"] > right_column_min - 50]
    
    # Need at least 2 blocks in each column
    if len(left_blocks_in_column) < 2 or len(right_blocks_in_column) < 2:
        return False, 0.0
    
    # Check if there's a clear separation between columns
    left_edges = [b["x"] + b["w"] for b in left_blocks_in_column]
    right_edges = [b["x"] for b in right_blocks_in_column]
    
    if left_edges and right_edges:
        max_left_edge = max(left_edges)
        min_right_edge = min(right_edges)
        gap = min_right_edge - max_left_edge
        
        # If there's a gap (or small overlap) and enough blocks in each column, it's multi-column
        if gap > -20 and len(left_blocks_in_column) >= 2 and len(right_blocks_in_column) >= 2:
            return True, middle_x
    
    return False, 0.0


def sort_blocks_by_reading_order(blocks: list[dict]) -> list[dict]:
    """Sort blocks in natural reading order, handling multi-column layouts.
    
    For single-column: top to bottom
    For multi-column: left column top to bottom, then right column top to bottom
    """
    if not blocks:
        return blocks
    
    is_multi_column, divider_x = detect_column_layout(blocks)
    
    if is_multi_column:
        # Separate into columns
        left_column = [b for b in blocks if (b["x"] + b["w"] / 2) < divider_x]
        right_column = [b for b in blocks if (b["x"] + b["w"] / 2) >= divider_x]
        
        # Sort each column by y, then x
        left_column.sort(key=lambda b: (b["y"], b["x"]))
        right_column.sort(key=lambda b: (b["y"], b["x"]))
        
        # Combine: left column first, then right column
        return left_column + right_column
    else:
        # Single column: sort by y, then x (original behavior)
        blocks.sort(key=lambda b: (b["y"], b["x"]))
        return blocks


def classify_block_type(roi: np.ndarray, text: str) -> str:
    """Classify block type based on visual features and text content.
    
    Classification priority:
    1. Math equations (display math with high symbol density or specific patterns)
    2. Tables (clear grid structure)
    3. Figures (large graphics with minimal text)
    4. Text (default for everything else)
    """
    h, w = roi.shape[:2]
    area = w * h
    text_stripped = text.strip()
    text_len = len(text_stripped)
    
    # Check for mathematical content first
    # Display equations often have specific patterns and high symbol density
    math_indicators = [
        r'[∫∑∏∂∇]',  # integral, sum, product, partial, nabla
        r'[α-ωΑ-Ω]',  # Greek letters
        r'[λμσπΔΣΩ]',  # Common Greek letters
        r'lim\s|sin\s|cos\s|tan\s|exp\s|log\s',  # functions
        r'\^\d+|\^-|\^t',  # exponents
        r'\\frac|\\int|\\sum|\\prod',  # LaTeX commands (if OCR catches them)
        r'dx\s|dt\s|dy\s',  # differentials
        r'[≤≥≠∞⊂⊃∪∩∈∉±×÷√]',  # math symbols
    ]
    
    math_pattern_matches = sum(1 for pattern in math_indicators if re.search(pattern, text, re.IGNORECASE))
    
    # Count special math characters
    math_symbols = r'[∫∑∏∂∇±≤≥≠∞⊂⊃∪∩∧∨¬∀∃√πλμσΣΔΩαβγδεζηθικλμνξπρστυφχψω()=\[\]]'
    math_char_count = len(re.findall(math_symbols, text))
    
    # Classify as math if:
    # 1. Multiple math indicators OR
    # 2. High density of math symbols OR  
    # 3. Moderate symbols + reasonable size (likely equation block)
    if text_len > 5:
        math_density = math_char_count / text_len if text_len > 0 else 0
        
        # Strong math indicators
        if math_pattern_matches >= 2:
            return "math"
        
        # High symbol density
        if math_density > 0.2 and text_len > 10:
            return "math"
        
        # Moderate indicators with good size (equations are often 100-400px area)
        if math_pattern_matches >= 1 and 5000 < area < 100000 and math_char_count >= 3:
            return "math"
    
    # If there's substantial regular text, it's a text block
    if text_len > 30 and math_pattern_matches < 2:
        return "text"
    
    # For blocks with little or no text, check visual features
    gray = cv2.cvtColor(roi, cv2.COLOR_BGR2GRAY) if len(roi.shape) == 3 else roi
    edges = cv2.Canny(gray, 50, 150, apertureSize=3)
    edge_density = cv2.countNonZero(edges) / area if area > 0 else 0
    
    # Figure detection FIRST: complex graphics (TikZ, charts, diagrams)
    # High edge density indicates complex visual content
    # TikZ figures typically have:
    # - Large size (>50000px at 200dpi = ~250x200 px)
    # - High edge density (lots of curves, lines, annotations)
    # - Minimal or moderate text (labels, annotations)
    if area > 50000 and edge_density > 0.08:
        return "figure"
    
    # Table detection: look for grid structure (but not if it's a complex figure)
    h_kernel = cv2.getStructuringElement(cv2.MORPH_RECT, (max(w // 6, 20), 1))
    h_lines = cv2.morphologyEx(edges, cv2.MORPH_OPEN, h_kernel)
    h_line_count = cv2.countNonZero(h_lines)
    
    v_kernel = cv2.getStructuringElement(cv2.MORPH_RECT, (1, max(h // 6, 20)))
    v_lines = cv2.morphologyEx(edges, cv2.MORPH_OPEN, v_kernel)
    v_line_count = cv2.countNonZero(v_lines)
    
    h_density = h_line_count / area if area > 0 else 0
    v_density = v_line_count / area if area > 0 else 0
    
    # Table: strong grid structure with lower edge density (not a complex figure)
    if h_density > 0.01 and v_density > 0.01 and area > 10000 and edge_density < 0.05:
        return "table"
    
    # Default to text
    return "text"


def extract_text_with_conf(roi: np.ndarray, lang: str, psm: int) -> tuple[str, float]:
    """Extract OCR text and mean confidence for one ROI/config pair."""
    data = pytesseract.image_to_data(
        roi,
        lang=lang,
        config=f"--psm {psm} --oem 1",
        output_type=Output.DICT,
    )
    tokens = []
    confs = []
    n = len(data.get("text", []))
    for i in range(n):
        text = (data["text"][i] or "").strip()
        if not text:
            continue
        try:
            conf = float(data["conf"][i])
        except (ValueError, TypeError):
            continue
        if conf < 0:
            continue
        tokens.append(text)
        confs.append(conf)

    joined = " ".join(tokens).strip()
    mean_conf = float(sum(confs) / len(confs)) if confs else 0.0
    return joined, mean_conf


def ocr_candidate_score(text: str, conf_0_to_100: float) -> float:
    """Heuristic quality score for choosing best OCR candidate."""
    norm = normalize_ocr_text(text)
    if not norm:
        return -1.0
    conf = max(0.0, min(100.0, conf_0_to_100)) / 100.0
    length = min(len(norm), 180) / 180.0
    alnum_or_korean = len(re.findall(r"[A-Za-z0-9가-힣]", norm))
    alpha_ratio = alnum_or_korean / max(1, len(norm))
    noise_penalty = 0.35 if is_probably_noise_text(norm) else 0.0
    return conf * 0.52 + length * 0.28 + alpha_ratio * 0.20 - noise_penalty


def run_ocr(image_path: Path, lang: str = "eng") -> list[dict]:
    """Run block-wise OCR with type classification.

    Args:
        image_path: Path to the image file
        lang: Tesseract language code (e.g., 'eng', 'kor', 'eng+kor')

    Returns list of blocks with structure:
    {"text": str, "bbox": [x0, y0, x1, y1], "block_type": str, "latex": str (optional)}
    """
    img = cv2.imread(str(image_path))
    blocks = detect_blocks(image_path)
    
    results = []
    latex_model = None

    def process_block(block: dict) -> dict | None:
        x, y, w, h = block["x"], block["y"], block["w"], block["h"]
        roi = img[y:y+h, x:x+w]

        if roi.size == 0:
            return None

        # Multi-pass OCR for higher recall/precision:
        # - original ROI
        # - grayscale ROI
        # - Otsu-thresholded ROI
        # with both PSM 6 (block) and PSM 11 (sparse text).
        roi_gray = _to_grayscale(roi)
        roi_bin = _otsu_threshold(roi_gray)
        variants = [roi, roi_gray, roi_bin]
        psm_modes = [6, 11]
        best_text = ""
        best_conf = 0.0
        best_score = -1.0
        for variant in variants:
            for psm in psm_modes:
                cand_text, cand_conf = extract_text_with_conf(variant, lang=lang, psm=psm)
                score = ocr_candidate_score(cand_text, cand_conf)
                if score > best_score:
                    best_score = score
                    best_text = cand_text
                    best_conf = cand_conf

        text = normalize_ocr_text(best_text)
        block_type = classify_block_type(roi, text)

        return {
            "text": text,
            "bbox": [float(x), float(y), float(x + w), float(y + h)],
            "block_type": block_type,
            "confidence": max(0.05, min(1.0, best_conf / 100.0)),
        }

    worker_count = _resolve_ocr_workers()
    indexed_blocks = list(enumerate(blocks))
    ordered_results: list[tuple[int, dict]] = []

    if worker_count <= 1 or len(indexed_blocks) <= 1:
        for idx, block in indexed_blocks:
            result = process_block(block)
            if result is not None:
                ordered_results.append((idx, result))
    else:
        def process_indexed(pair: tuple[int, dict]) -> tuple[int, dict | None]:
            idx, block = pair
            return idx, process_block(block)

        with ThreadPoolExecutor(max_workers=worker_count) as pool:
            for idx, result in pool.map(process_indexed, indexed_blocks):
                if result is not None:
                    ordered_results.append((idx, result))

    ordered_results.sort(key=lambda item: item[0])

    for _, result in ordered_results:
        # For math blocks, try LaTeX OCR
        if result["block_type"] == "math":
            x0, y0, x1, y1 = (int(v) for v in result["bbox"])
            roi = img[y0:y1, x0:x1]
            if roi.size == 0:
                results.append(result)
                continue

            if latex_model is None:
                latex_model = get_latex_model()
            
            if latex_model:
                try:
                    # Convert ROI to PIL Image for pix2tex
                    roi_rgb = cv2.cvtColor(roi, cv2.COLOR_BGR2RGB)
                    pil_img = Image.fromarray(roi_rgb)
                    latex = latex_model(pil_img)
                    result["latex"] = latex
                except Exception as e:
                    print(f"LaTeX OCR failed: {e}", file=sys.stderr)
                    result["latex"] = ""
            else:
                result["latex"] = ""

        results.append(result)
    
    return results


def fallback_full_page_ocr(
    img: np.ndarray,
    lang: str,
    existing_blocks: list[dict],
    min_conf: float = 55.0,
    min_token_len: int = 3,
) -> list[dict]:
    """Fallback OCR that scans the whole page and groups words into line blocks."""
    data = pytesseract.image_to_data(
        img,
        lang=lang,
        config="--psm 11 --oem 1",
        output_type=Output.DICT,
    )

    line_groups: dict[tuple[int, int, int], list[dict]] = defaultdict(list)
    n = len(data.get("text", []))
    for i in range(n):
        raw_text = data["text"][i].strip()
        if not raw_text:
            continue

        try:
            conf = float(data["conf"][i])
        except (ValueError, TypeError):
            continue

        if conf < min_conf or len(raw_text) < min_token_len:
            continue

        key = (data["block_num"][i], data["par_num"][i], data["line_num"][i])
        line_groups[key].append(
            {
                "text": raw_text,
                "conf": conf,
                "x": int(data["left"][i]),
                "y": int(data["top"][i]),
                "w": int(data["width"][i]),
                "h": int(data["height"][i]),
            }
        )

    fallback = []
    for words in line_groups.values():
        text = " ".join(w["text"] for w in words).strip()
        if len(text) < 3:
            continue

        x0 = min(w["x"] for w in words)
        y0 = min(w["y"] for w in words)
        x1 = max(w["x"] + w["w"] for w in words)
        y1 = max(w["y"] + w["h"] for w in words)
        bbox = [float(x0), float(y0), float(x1), float(y1)]
        avg_conf = sum(float(w["conf"]) for w in words) / max(1, len(words))

        # Skip lines that are already mostly covered by detected OCR blocks.
        covered = False
        for block in existing_blocks:
            bx0, by0, bx1, by1 = block["bbox"]
            ix0, iy0 = max(bx0, bbox[0]), max(by0, bbox[1])
            ix1, iy1 = min(bx1, bbox[2]), min(by1, bbox[3])
            iw, ih = max(0.0, ix1 - ix0), max(0.0, iy1 - iy0)
            inter = iw * ih
            area = max(1.0, (bbox[2] - bbox[0]) * (bbox[3] - bbox[1]))
            if inter / area > 0.7:
                covered = True
                break
        if covered:
            continue

        fallback.append(
            {
                "text": normalize_ocr_text(text),
                "bbox": bbox,
                "block_type": "text",
                "confidence": (avg_conf / 100.0),
            }
        )

    return fallback


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--image", required=True)
    parser.add_argument("--lang", default="eng+kor", help="Tesseract language (e.g., eng, kor, eng+kor)")
    args = parser.parse_args()

    image_path = Path(args.image)
    if not image_path.exists():
        print(f"Image not found: {image_path}", file=sys.stderr)
        return 1

    tokens = run_ocr(image_path, lang=args.lang)

    # If block-level OCR returned too little text, add sparse full-page OCR as a recall boost.
    total_text_len = sum(len(t.get("text", "").strip()) for t in tokens)
    if len(tokens) <= 2 or total_text_len < 50:
        img = cv2.imread(str(image_path))
        if img is not None:
            fallback = fallback_full_page_ocr(img, args.lang, tokens)
            tokens.extend(fallback)
    tokens = post_process_tokens(tokens)

    print(json.dumps(tokens, ensure_ascii=False))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
