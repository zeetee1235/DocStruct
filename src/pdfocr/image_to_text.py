"""
Extract text from images using OCR.
"""
import logging
from pathlib import Path
from typing import Dict, Sequence

import pytesseract
from PIL import Image

from pdfocr.types import PathLike

TextDict = Dict[str, str]
logger = logging.getLogger(__name__)


def extract_text_from_image(image_path: PathLike, lang: str = "kor") -> str:
    """
    Extract text from a single image.
    
    Args:
        image_path: Path to image file
        lang: OCR language code (default: "kor")
    
    Returns:
        Extracted text
    """
    image_path = Path(image_path)
    if not image_path.exists():
        raise FileNotFoundError(f"Image file not found: {image_path}")

    try:
        with Image.open(image_path) as image:
            text = pytesseract.image_to_string(image, lang=lang)
        return text
    except Exception as exc:
        raise RuntimeError(f"Text extraction failed for {image_path}: {exc}") from exc


def extract_text_from_images(image_paths: Sequence[PathLike], lang: str = "kor") -> TextDict:
    """
    Extract text from multiple images.
    
    Args:
        image_paths: List of image file paths
        lang: OCR language code (default: "kor")
    
    Returns:
        Dictionary mapping image paths to extracted text
    """
    image_paths = [Path(p) for p in image_paths]
    logger.info(f"Starting OCR (language: {lang})")
    logger.info(f"Processing {len(image_paths)} image(s)")
    
    results: TextDict = {}
    
    for i, image_path in enumerate(image_paths, start=1):
        logger.debug(f"[{i}/{len(image_paths)}] Processing: {image_path.name}")
        try:
            text = extract_text_from_image(image_path, lang=lang)
            results[str(image_path)] = text
            logger.debug(f"Extracted {len(text)} characters")
        except Exception as exc:
            logger.error(f"Error: {exc}")
            results[str(image_path)] = ""
    
    logger.info("OCR extraction completed")
    return results


def save_extracted_text(text_dict: TextDict, output_path: PathLike = "output/extracted_text.txt") -> None:
    """
    Save extracted text to a file.
    
    Args:
        text_dict: Dictionary mapping image paths to extracted text
        output_path: Output file path (default: "output/extracted_text.txt")
    """
    output_path = Path(output_path)
    output_dir = output_path.parent

    if output_dir and not output_dir.exists():
        output_dir.mkdir(parents=True, exist_ok=True)
        logger.debug(f"Created directory: {output_dir}")
    
    logger.info(f"Saving text file: {output_path}")
    
    with output_path.open('w', encoding='utf-8') as f:
        for i, (image_path, text) in enumerate(sorted(text_dict.items()), start=1):
            page_name = Path(image_path).name
            f.write(f"{'='*80}\n")
            f.write(f"Page {i}: {page_name}\n")
            f.write(f"{'='*80}\n\n")
            f.write(text)
            f.write("\n\n\n")
    
    logger.info(f"Saved: {output_path}")


if __name__ == "__main__":  # pragma: no cover - CLI helper
    import glob
    import sys
    
    if len(sys.argv) < 2:
        print("Usage: python image_to_text.py <IMAGE_DIRECTORY>")
        sys.exit(1)
    
    image_dir = Path(sys.argv[1])
    image_files = sorted(glob.glob(str(image_dir / "*.png")))
    
    if not image_files:
        print(f"Error: No PNG files found in {image_dir}")
        sys.exit(1)
    
    try:
        text_results = extract_text_from_images(image_files)
        save_extracted_text(text_results)
        print("Text extraction and save completed!")
    except Exception as exc:
        print(f"Error: {exc}")
        sys.exit(1)
