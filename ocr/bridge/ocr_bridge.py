#!/usr/bin/env python3
import argparse
import json
import re
import sys
from pathlib import Path

import cv2
import numpy as np
from PIL import Image
import pytesseract
from pytesseract import Output

# Lazy import for pix2tex to avoid slow startup when not needed
_latex_model = None

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
    gray = cv2.cvtColor(img, cv2.COLOR_BGR2GRAY)
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


def run_ocr(image_path: Path) -> list[dict]:
    """Run block-wise OCR with type classification.

    Returns list of blocks with structure:
    {"text": str, "bbox": [x0, y0, x1, y1], "block_type": str, "latex": str (optional)}
    """
    img = cv2.imread(str(image_path))
    blocks = detect_blocks(image_path)
    
    results = []
    latex_model = None
    
    for block in blocks:
        x, y, w, h = block["x"], block["y"], block["w"], block["h"]
        roi = img[y:y+h, x:x+w]
        
        # Run regular OCR
        text = pytesseract.image_to_string(roi).strip()
        block_type = classify_block_type(roi, text)
        
        result = {
            "text": text,
            "bbox": [float(x), float(y), float(x + w), float(y + h)],
            "block_type": block_type
        }
        
        # For math blocks, try LaTeX OCR
        if block_type == "math":
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


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--image", required=True)
    args = parser.parse_args()

    image_path = Path(args.image)
    if not image_path.exists():
        print(f"Image not found: {image_path}", file=sys.stderr)
        return 1

    tokens = run_ocr(image_path)
    print(json.dumps(tokens, ensure_ascii=False))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
