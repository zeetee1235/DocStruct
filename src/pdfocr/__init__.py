"""PDFOCR package initialization."""

from pdfocr.main import main
from pdfocr.layout import Block, detect_blocks, draw_blocks
from pdfocr.block_ocr import ocr_blocks, extract_blocks_to_json

__all__ = [
    "main",
    "Block",
    "detect_blocks",
    "draw_blocks",
    "ocr_blocks",
    "extract_blocks_to_json",
]
