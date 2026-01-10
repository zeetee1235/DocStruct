#!/usr/bin/env python3
"""Compatibility entrypoint for the PDFOCR CLI."""
import sys
from pathlib import Path


def _add_src_to_path() -> None:
    """현재 repo 루트 기준 src 디렉토리를 sys.path에 추가."""
    repo_root = Path(__file__).resolve().parent
    src_dir = repo_root / "src"
    if src_dir.exists():
        sys.path.insert(0, str(src_dir))


if __name__ == "__main__":
    _add_src_to_path()
    from pdfocr.main import main

    main()
