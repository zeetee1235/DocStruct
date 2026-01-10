"""
Convert PDF to page-by-page images.
"""
import logging
from pathlib import Path
from typing import List

from pdf2image import convert_from_path

from pdfocr.types import PathLike

logger = logging.getLogger(__name__)


def _ensure_output_dir(output_dir: Path) -> None:
    created = not output_dir.exists()
    output_dir.mkdir(parents=True, exist_ok=True)
    if created:
        logger.debug(f"Created directory: {output_dir}")


def convert_pdf_to_images(pdf_path: PathLike, output_dir: PathLike = "images", dpi: int = 300) -> List[str]:
    """
    Convert PDF file to page-by-page images.
    
    Args:
        pdf_path: Path to PDF file
        output_dir: Directory to save images (default: "images")
        dpi: Image resolution (default: 300)
    
    Returns:
        List of generated image file paths
    """
    pdf_path = Path(pdf_path).expanduser().resolve()
    output_dir = Path(output_dir).expanduser().resolve()

    if not pdf_path.exists():
        raise FileNotFoundError(f"PDF file not found: {pdf_path}")

    _ensure_output_dir(output_dir)

    logger.info(f"Converting PDF: {pdf_path}")
    try:
        images = convert_from_path(str(pdf_path), dpi=dpi)
        logger.info(f"Detected {len(images)} page(s)")
    except Exception as exc:
        raise RuntimeError(f"PDF conversion error: {exc}") from exc

    image_paths: List[str] = []
    pdf_basename = pdf_path.stem

    for i, image in enumerate(images, start=1):
        image_path = output_dir / f"{pdf_basename}_page_{i:03d}.png"
        image.save(image_path, "PNG")
        image_paths.append(str(image_path))
        logger.debug(f"Saved: {image_path}")

    logger.info(f"Generated {len(image_paths)} image(s)")
    return image_paths


if __name__ == "__main__":
    import sys

    if len(sys.argv) < 2:
        print("Usage: python pdf_to_image.py <PDF_FILE>")
        sys.exit(1)

    pdf_file = sys.argv[1]
    try:
        image_files = convert_pdf_to_images(pdf_file)
        print(f"Generated {len(image_files)} image(s)")
    except Exception as exc:  # pragma: no cover - CLI helper
        print(f"Error: {exc}")
        sys.exit(1)
