#!/usr/bin/env python3
import argparse
import json
import sys
from pathlib import Path

from PIL import Image
import pytesseract
from pytesseract import Output


def run_ocr(image_path: Path) -> list[dict]:
    """Run Tesseract OCR on the given image and return tokens.

    Each token is a dict: {"text": str, "bbox": [x0, y0, x1, y1]} in pixel coords.
    """
    image = Image.open(image_path)

    data = pytesseract.image_to_data(image, output_type=Output.DICT)

    tokens: list[dict] = []
    n = len(data.get("text", []))
    for i in range(n):
        text = (data["text"][i] or "").strip()
        if not text:
            continue

        try:
            x = int(data["left"][i])
            y = int(data["top"][i])
            w = int(data["width"][i])
            h = int(data["height"][i])
        except (KeyError, ValueError, TypeError):
            continue

        if w <= 0 or h <= 0:
            continue

        bbox = [float(x), float(y), float(x + w), float(y + h)]
        tokens.append({"text": text, "bbox": bbox})

    return tokens


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
