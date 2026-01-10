"""
블록 감지 + 블록별 OCR 결과를 JSON 형태로 제공하는 유틸리티.
"""
import json
from pathlib import Path
from typing import Dict, List, Sequence

import pytesseract

from pdfocr.image_utils import read_image
from pdfocr.layout import Block, detect_blocks
from pdfocr.types import PathLike


def ocr_blocks(image_path: PathLike,
               blocks: Sequence[Block],
               lang: str = "kor") -> List[Dict]:
    """
    감지된 블록 리스트에 대해 OCR을 수행해 구조화된 dict 리스트를 반환한다.
    """
    image = read_image(image_path)
    results: List[Dict] = []

    for idx, block in enumerate(blocks, start=1):
        x, y, w, h = block.as_bbox()
        roi = image[y:y + h, x:x + w]
        text = pytesseract.image_to_string(roi, lang=lang)
        results.append({
            "index": idx,
            "bbox": {"x": x, "y": y, "w": w, "h": h},
            "type": "text",  # 추후 수식/표 등으로 확장
            "lang": lang,
            "text": text.strip(),
        })

    return results


def extract_blocks_to_json(image_path: PathLike,
                           output_path: PathLike,
                           lang: str = "kor",
                           min_area: int = 800,
                           merge_kernel: tuple[int, int] = (15, 7)) -> Path:
    """
    이미지 한 장을 블록 단위로 OCR하고 JSON 파일로 저장한다.
    """
    blocks = detect_blocks(image_path, min_area=min_area, merge_kernel=merge_kernel)
    ocr_results = ocr_blocks(image_path, blocks, lang=lang)

    out_path = Path(output_path)
    out_path.parent.mkdir(parents=True, exist_ok=True)

    payload = {
        "source_image": str(Path(image_path).resolve()),
        "block_count": len(ocr_results),
        "blocks": ocr_results,
    }

    with out_path.open("w", encoding="utf-8") as f:
        json.dump(payload, f, ensure_ascii=False, indent=2)

    return out_path


if __name__ == "__main__":  # pragma: no cover - CLI 헬퍼
    import argparse

    parser = argparse.ArgumentParser(description="단일 이미지에서 블록 OCR JSON 생성")
    parser.add_argument("image", help="입력 이미지 경로 (PNG 등)")
    parser.add_argument("-o", "--output", default="blocks.json", help="출력 JSON 경로")
    parser.add_argument("-l", "--lang", default="kor", help="OCR 언어 (기본: kor)")
    parser.add_argument("--min-area", type=int, default=800, help="감지 블록 최소 면적")
    parser.add_argument("--merge-kernel", type=int, nargs=2, default=(15, 7),
                        metavar=("W", "H"), help="팽창 커널 크기 (W H)")

    args = parser.parse_args()
    out = extract_blocks_to_json(
        args.image,
        args.output,
        lang=args.lang,
        min_area=args.min_area,
        merge_kernel=tuple(args.merge_kernel),
    )
    print(f"✓ 저장 완료: {out}")
