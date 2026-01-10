# Architecture

## Overview

pdfocr is a command-line tool that extracts text from PDF documents using OCR (Optical Character Recognition). The pipeline consists of three main stages:

1. **PDF to Image Conversion** - Convert PDF pages to high-resolution PNG images
2. **OCR Processing** - Extract text from images using Tesseract OCR
3. **Text Output** - Save extracted text with proper formatting

## Module Structure

### Core Modules

#### `src/pdfocr/main.py`
Main entry point for the CLI application.
- Command-line argument parsing
- Pipeline orchestration
- File path resolution
- Progress reporting

**Key Functions:**
- `process_single_pdf()` - Process one PDF file
- `process_multiple_pdfs()` - Batch processing with optional merge
- `main()` - CLI entry point with argument parsing

#### `src/pdfocr/pdf_to_image.py`
PDF to image conversion using pdf2image (poppler wrapper).

**Key Functions:**
- `convert_pdf_to_images()` - Convert PDF pages to PNG images
  - Uses poppler's `pdftoppm` underneath
  - Configurable DPI (default: 300)
  - Returns list of image paths

**Dependencies:**
- `pdf2image` - Python wrapper for poppler
- `poppler-utils` - System package for PDF rendering

#### `src/pdfocr/image_to_text.py`
OCR text extraction using pytesseract (Tesseract wrapper).

**Key Functions:**
- `extract_text_from_image()` - Extract text from single image
- `extract_text_from_images()` - Batch extraction from multiple images
- `save_extracted_text()` - Save results to formatted text file

**Features:**
- Multi-language support (default: Korean)
- Error handling for failed extractions
- Page-separated output format

**Dependencies:**
- `pytesseract` - Python wrapper for Tesseract
- `Pillow` - Image processing
- `tesseract-ocr` - System package for OCR engine

### Advanced Modules

#### `src/pdfocr/layout.py`
Layout analysis and block detection using OpenCV.

**Key Functions:**
- `detect_blocks()` - Detect text blocks in images
- `draw_blocks()` - Visualize detected blocks
- Uses morphological operations for block detection

**Use Cases:**
- Complex document layouts
- Multi-column text
- Table extraction

#### `src/pdfocr/block_ocr.py`
Block-based OCR for better accuracy on complex layouts.

**Key Functions:**
- `extract_blocks_to_json()` - Extract text blocks with position data
- Outputs JSON with block coordinates and text

**Use Cases:**
- Preserving spatial information
- Structured document parsing
- Layout-aware text extraction

## Execution Flow

### Single PDF Processing

```
User Input
    ↓
Command Line Parser (main.py)
    ↓
Path Resolution & Validation
    ↓
PDF to Image (pdf_to_image.py)
    ↓
OCR Extraction (image_to_text.py)
    ↓
Text File Output
    ↓
Cleanup (optional)
```

### Batch Processing

```
Multiple PDF Files
    ↓
For each PDF:
    ├─→ Convert to Images
    ├─→ Extract Text
    └─→ Save Individual File
        ↓
    (Optional) Merge All Texts
        ↓
    Output Merged File
```

## Configuration

### Environment Variables
None currently used.

### Command-Line Options
- `-o, --output-dir` - Output directory
- `-i, --image-dir` - Temporary image directory
- `-l, --lang` - OCR language code (default: "kor")
- `-d, --dpi` - Image resolution (default: 300)
- `--keep-images` - Preserve temporary images
- `--merge` - Merge multiple outputs into one file

## Dependencies

### Python Packages
- `pdf2image` - PDF to image conversion
- `pytesseract` - Tesseract OCR wrapper
- `Pillow>=11.0.0` - Image processing (Python 3.13 compatible)
- `opencv-python>=4.8.0` - Computer vision (optional, for layout)
- `numpy>=1.24.0` - Numerical operations (optional, for layout)

### System Packages
- `poppler-utils` - PDF rendering (pdftoppm, pdfinfo)
- `tesseract-ocr` - OCR engine
- `tesseract-langpack-kor` - Korean language support (optional)

## Error Handling

### File Not Found
- Validates all input paths before processing
- Provides clear error messages with file paths

### OCR Failures
- Continues processing remaining files
- Logs errors without stopping pipeline
- Returns empty string for failed extractions

### Resource Cleanup
- Temporary images deleted by default
- Can be preserved with `--keep-images`
- Handles cleanup failures gracefully

## Performance Considerations

### Memory Usage
- Images loaded one at a time
- Temporary files cleaned up after processing
- No accumulated memory for batch processing

### Processing Speed
Factors affecting speed:
- DPI setting (higher = slower but more accurate)
- Number of pages
- Image complexity
- System OCR performance

Typical speeds:
- 300 DPI: ~2-5 seconds per page
- 600 DPI: ~5-10 seconds per page

### Parallelization
Currently single-threaded. Future improvements:
- Parallel page processing
- Batch OCR operations
- Async I/O for file operations

## Future Enhancements

### Planned Features
- [ ] Parallel processing for multiple pages
- [ ] PDF text layer detection (skip OCR if text exists)
- [ ] Confidence scores for extracted text
- [ ] Better table extraction
- [ ] Mathematical formula recognition
- [ ] Progress bars for long operations

### Potential Improvements
- Configuration file support
- Plugin system for custom processors
- Web API interface
- GUI application
- Docker container

## Testing

### Test Suite
Located in `test/` directory:
- `test_document.tex` - LaTeX source for test PDF
- `test_document.pdf` - Generated test document
- `test.fish` - Integration test script

### Running Tests
```bash
./test.fish
```

This will:
1. Build test PDF from LaTeX
2. Run OCR pipeline
3. Perform block detection
4. Verify outputs

## Logging

### Log Levels
- `INFO` - Normal operation messages
- `DEBUG` - Detailed processing information
- `WARNING` - Non-critical issues
- `ERROR` - Failure messages

### Log Output
Currently logs to stdout. Future: configurable log files.

## Compatibility

### Python Versions
- Python 3.13+ (recommended)
- Python 3.11+ (supported)

### Operating Systems
- Linux (Fedora, Ubuntu, Debian) - Primary
- macOS - Supported
- Windows - Not tested

### Known Limitations
- Right-to-left text not optimized
- Vertical text may require rotation
- Handwritten text has low accuracy
- Mathematical symbols often misrecognized
