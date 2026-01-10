#!/usr/bin/env python3
"""
PDF to Image to Text extraction pipeline.
"""
import argparse
import glob
import logging
import os
import sys
import tempfile
from pathlib import Path
from typing import Iterable, List, Sequence

from pdfocr.image_to_text import extract_text_from_images, save_extracted_text
from pdfocr.pdf_to_image import convert_pdf_to_images
from pdfocr.types import PathLike

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(message)s'
)
logger = logging.getLogger(__name__)


def _resolve_pdf_path(pdf_path: PathLike) -> Path:
    path = Path(pdf_path).expanduser().resolve()
    if not path.exists():
        raise FileNotFoundError(f"PDF file not found: {pdf_path}")
    return path


def _resolve_output_dir(pdf_path: Path, output_dir: PathLike | None) -> Path:
    if output_dir is None:
        return pdf_path.parent
    return Path(output_dir).expanduser().resolve()


def _resolve_image_dir(image_dir: PathLike | None) -> tuple[Path, bool]:
    if image_dir is None:
        temp_path = Path(tempfile.mkdtemp(prefix="pdf2txt_"))
        return temp_path, True
    return Path(image_dir).expanduser().resolve(), False


def _cleanup_images(image_paths: Sequence[str], image_dir: Path, remove_dir: bool) -> None:
    logger.debug("Cleaning up temporary image files...")
    for img_path in image_paths:
        try:
            Path(img_path).unlink(missing_ok=True)
        except Exception as exc:
            logger.warning(f"Failed to delete {img_path}: {exc}")

    if remove_dir:
        try:
            if image_dir.exists() and not any(image_dir.iterdir()):
                image_dir.rmdir()
        except Exception:
            pass

    logger.debug("Cleanup completed")


def process_single_pdf(pdf_path: PathLike,
                       output_dir: PathLike | None = None,
                       image_dir: PathLike | None = None,
                       lang: str = "kor",
                       dpi: int = 300,
                       keep_images: bool = False):
    """
    Process a single PDF file through the OCR pipeline.
    
    Args:
        pdf_path: Path to PDF file
        output_dir: Output directory for text (defaults to PDF directory)
        image_dir: Directory for temporary images (defaults to temp directory)
        lang: OCR language code (default: "kor")
        dpi: Image resolution
        keep_images: Keep images after processing
    
    Returns:
        Path to generated text file
    """
    pdf_path = _resolve_pdf_path(pdf_path)
    output_dir = _resolve_output_dir(pdf_path, output_dir)
    image_dir, is_temp_dir = _resolve_image_dir(image_dir)
    
    print("=" * 80)
    print(f"Processing: {pdf_path.name}")
    print(f"Location: {pdf_path}")
    print(f"Output: {output_dir}")
    print("=" * 80)
    
    # Step 1: PDF to Image
    print("\n[1/3] Converting PDF to images...")
    try:
        image_paths = convert_pdf_to_images(pdf_path, output_dir=image_dir, dpi=dpi)
    except Exception as exc:
        print(f"Error: PDF conversion failed - {exc}")
        return None
    
    # Step 2: Image to Text OCR
    print("[2/3] Extracting text via OCR...")
    try:
        text_results = extract_text_from_images(image_paths, lang=lang)
    except Exception as exc:
        print(f"Error: OCR extraction failed - {exc}")
        return None
    
    # Step 3: Save text file
    print("[3/3] Saving text file...")
    pdf_basename = pdf_path.stem
    output_path = Path(output_dir) / f"{pdf_basename}.txt"
    
    try:
        save_extracted_text(text_results, output_path)
    except Exception as exc:
        print(f"Error: File save failed - {exc}")
        return None
    
    # Cleanup temporary images
    if not keep_images:
        _cleanup_images(image_paths, image_dir, is_temp_dir)
    
    print(f"\nCompleted: {output_path}")
    print("=" * 80 + "\n")
    
    return output_path


def process_multiple_pdfs(pdf_paths: Sequence[PathLike],
                         output_dir: PathLike | None = None,
                         image_dir: PathLike | None = None,
                         lang: str = "kor",
                         dpi: int = 300,
                         keep_images: bool = False,
                         merge: bool = False):
    """
    Process multiple PDF files in batch.
    
    Args:
        pdf_paths: List of PDF file paths
        output_dir: Output directory for text files
        image_dir: Directory for temporary images
        lang: OCR language code
        dpi: Image resolution
        keep_images: Keep images after processing
        merge: Merge all texts into one file
    """
    print(f"\nProcessing {len(pdf_paths)} PDF file(s)\n")
    
    # Use current directory if merge is enabled but output_dir is not specified
    if merge and output_dir is None:
        output_dir = Path.cwd()
    
    output_files: List[Path] = []
    all_texts: List[str] = []
    
    for i, pdf_path in enumerate(pdf_paths, start=1):
        print(f"\n[{i}/{len(pdf_paths)}] Processing...")
        output_file = process_single_pdf(
            pdf_path,
            output_dir=output_dir,
            image_dir=image_dir,
            lang=lang,
            dpi=dpi,
            keep_images=keep_images
        )
        
        if output_file:
            output_files.append(Path(output_file))
            if merge:
                with open(output_file, 'r', encoding='utf-8') as f:
                    all_texts.append(f.read())
    
    # Create merged file
    if merge and all_texts:
        merged_path = Path(output_dir) / "merged_all_texts.txt"
        print(f"\nMerging all texts into one file...")
        with merged_path.open('w', encoding='utf-8') as f:
            for i, text in enumerate(all_texts, start=1):
                f.write(f"\n{'#'*80}\n")
                f.write(f"# Document {i}: {Path(pdf_paths[i-1]).name}\n")
                f.write(f"{'#'*80}\n\n")
                f.write(text)
                f.write("\n\n")
        print(f"Merged file created: {merged_path}\n")
    
    print("\n" + "="*80)
    print(f"Completed: {len(output_files)}/{len(pdf_paths)} successful")
    print("="*80)


def _collect_valid_pdfs(patterns: Iterable[str]) -> List[Path]:
    expanded: list[str] = []
    for pattern in patterns:
        if "*" in pattern or "?" in pattern:
            expanded.extend(glob.glob(pattern))
        else:
            expanded.append(pattern)

    valid: List[Path] = []
    seen: set[Path] = set()

    for raw_path in expanded:
        resolved = Path(raw_path).expanduser().resolve()
        if resolved in seen:
            continue
        seen.add(resolved)

        if resolved.exists():
            valid.append(resolved)
        else:
            logger.warning(f"File not found: {raw_path}")

    return sorted(valid, key=lambda p: str(p))


def main():
    parser = argparse.ArgumentParser(
        description="PDF to Text extraction pipeline using OCR",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Process single PDF
  pdfocr lecture1.pdf
  
  # Process multiple PDFs
  pdfocr lecture1.pdf lecture2.pdf lecture3.pdf
  
  # Process all PDFs in directory
  pdfocr pdfs/*.pdf
  
  # Merge all texts into one file
  pdfocr pdfs/*.pdf --merge
  
  # Keep images for debugging
  pdfocr lecture.pdf --keep-images
        """
    )
    
    parser.add_argument(
        'pdf_files',
        nargs='+',
        help='PDF file path(s) to process'
    )
    
    parser.add_argument(
        '-o', '--output-dir',
        default=None,
        help='Output directory for text files (default: same as PDF)'
    )
    
    parser.add_argument(
        '-i', '--image-dir',
        default=None,
        help='Directory for temporary images (default: auto-generated temp directory)'
    )
    
    parser.add_argument(
        '-l', '--lang',
        default='eng+kor',
        help='OCR language code (default: eng+kor)'
    )
    
    parser.add_argument(
        '-d', '--dpi',
        type=int,
        default=300,
        help='Image resolution (default: 300)'
    )
    
    parser.add_argument(
        '--keep-images',
        action='store_true',
        help='Keep images after processing (default: delete)'
    )
    
    parser.add_argument(
        '--merge',
        action='store_true',
        help='Merge all texts into one file'
    )
    
    args = parser.parse_args()
    
    valid_pdfs = _collect_valid_pdfs(args.pdf_files)
     
    if not valid_pdfs:
        print("Error: No PDF files to process")
        sys.exit(1)
    
    # Run pipeline
    if len(valid_pdfs) == 1:
        process_single_pdf(
            valid_pdfs[0],
            output_dir=args.output_dir,
            image_dir=args.image_dir,
            lang=args.lang,
            dpi=args.dpi,
            keep_images=args.keep_images
        )
    else:
        process_multiple_pdfs(
            valid_pdfs,
            output_dir=args.output_dir,
            image_dir=args.image_dir,
            lang=args.lang,
            dpi=args.dpi,
            keep_images=args.keep_images,
            merge=args.merge
        )


if __name__ == "__main__":
    main()
