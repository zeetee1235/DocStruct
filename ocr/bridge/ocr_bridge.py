#!/usr/bin/env python3
import argparse
import json
import sys
from pathlib import Path


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--image", required=True)
    args = parser.parse_args()

    image_path = Path(args.image)
    if not image_path.exists():
        print(f"Image not found: {image_path}", file=sys.stderr)
        return 1

    # Placeholder for OCR engine integration.
    tokens = []
    print(json.dumps(tokens))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
