# DocStruct

PDF document structure recovery system using parser-OCR cross-validation.

## Overview

DocStruct extracts structured content from PDF documents by combining two independent analysis paths: a parser track that analyzes embedded text and fonts, and an OCR track that processes rendered page images. The fusion engine merges both hypotheses, resolving conflicts and assigning confidence scores.

## Features

- **Block Type Classification**: Automatically detects and classifies text, tables, figures, and math equations
- **Dual-Track Analysis**: Parser-based and OCR-based layout hypotheses
- **Confidence Scoring**: Each element tagged with provenance (parser/ocr/fused) and confidence (0-1)
- **Multiple Export Formats**:
  - JSON: Structured data with full metadata
  - Markdown: Text with embedded images for tables/figures
  - TXT: Plain text with block type annotations
  - HTML: Interactive debug viewer
- **LaTeX OCR**: Extracts mathematical equations as LaTeX using pix2tex

## Installation

### Requirements

- Rust 1.93.0+
- Python 3.12+
- poppler-utils (pdfinfo, pdftotext, pdftoppm)
- tesseract 5.3+

### Setup

#### Option 1: Using Nix (Recommended)

```bash
# With Nix flakes (recommended)
nix develop

# Or with legacy nix-shell
nix-shell

# Install pix2tex (not available in nixpkgs)
pip install --user 'pix2tex[gui]>=0.1.2'

# Build
cargo build --release
```

#### Option 2: Using direnv (auto-loading)

```bash
# Install direnv if not already installed
# Then allow the directory
direnv allow

# Environment will be automatically loaded
cargo build --release
```

#### Option 3: Manual Installation

```bash
# Install system dependencies (Ubuntu/Debian)
sudo apt install poppler-utils tesseract-ocr

# Install Python dependencies
pip install -r requirements.txt

# Build
cargo build --release
```

## Usage

### Basic Usage

```bash
# Convert a single PDF
docstruct convert input.pdf

# Specify output directory
docstruct convert input.pdf -o output_dir

# Adjust DPI for OCR
docstruct convert input.pdf --dpi 150

# Enable debug outputs
docstruct convert input.pdf --debug

# Quiet mode (no progress output)
docstruct convert input.pdf --quiet
```

### Batch Processing

```bash
# Convert multiple PDFs
docstruct batch file1.pdf file2.pdf file3.pdf

# With custom output directory
docstruct batch *.pdf -o results/
```

### PDF Information

```bash
# Show PDF metadata
docstruct info input.pdf
```

### Output Files

```
output_dir/
├── document.json         # Structured data (all blocks, lines, spans)
├── document.md          # Markdown with embedded images
├── document.txt         # Plain text with block markers
├── page_001.md          # Per-page markdown
├── figures/
│   └── page_NNN_TYPE__NN.png  # Extracted images
└── debug/
    ├── page_001.html    # Interactive debug viewer
    └── page_001.png     # Rendered page image
```

## Architecture

### Pipeline

1. **Parser Track**: Extract text positions from PDF internal structure
2. **OCR Track**: Render pages to images, detect blocks, classify types, run OCR
3. **Fusion**: Align blocks, compare content, resolve conflicts, assign confidence
4. **Export**: Generate JSON, Markdown, TXT, and HTML outputs

### Block Classification

The OCR bridge classifies blocks based on visual features:

- **Math**: Pattern matching (∫∑∏∂∇, Greek letters, function names) + symbol density
- **Figure**: High edge density (>0.08) for graphics and diagrams
- **Table**: Grid structure detection (horizontal/vertical line density)
- **Text**: Default classification

### Coordinate System

All coordinates are in page pixel space based on the rendering DPI (default 200).

## Document Schema

```json
{
  "pages": [
    {
      "page_idx": 0,
      "class": "digital",
      "width": 1000,
      "height": 1400,
      "blocks": [
        {
          "type": "TextBlock",
          "bbox": {"x0": 10.0, "y0": 20.0, "x1": 400.0, "y1": 80.0},
          "lines": [{"spans": [...]}],
          "confidence": 0.85,
          "source": "fused"
        },
        {
          "type": "MathBlock",
          "bbox": {"x0": 50.0, "y0": 100.0, "x1": 300.0, "y1": 150.0},
          "latex": "\\int_{0}^{\\infty} e^{-x} dx",
          "confidence": 0.72,
          "source": "ocr"
        }
      ]
    }
  ]
}
```

## Project Structure

```
src/
  core/           # Geometry, data models, confidence scoring
  parser/         # PDF text extraction and layout analysis
  ocr/            # Image rendering, OCR bridge, layout building
  fusion/         # Hypothesis alignment and conflict resolution
  export/         # JSON, Markdown, TXT, HTML exporters
ocr/bridge/       # Python OCR integration (Tesseract, pix2tex)
test/             # Test documents
docs/             # Architecture and implementation documentation
```

## Debug Viewer

The HTML debug viewer provides interactive visualization:

- Color-coded blocks by type (text/table/figure/math)
- Click blocks to view parser text, OCR text, confidence, and similarity scores
- Toggle between parser, OCR, and fused hypotheses

## Configuration

Key parameters in `ocr/bridge/ocr_bridge.py`:

```python
detect_blocks(
    min_area=2000,           # Minimum block area in pixels
    merge_kernel=(15, 10)    # Morphological kernel for block merging
)
```

Classification thresholds:
- Math: symbol_density > 0.2 or 2+ pattern matches
- Figure: edge_density > 0.08 and area > 50000
- Table: h_density > 0.01 and v_density > 0.01

## Testing

```bash
# Unit tests
cargo test

# Integration test
./target/release/docstruct test/test_document.pdf --out test_output --dpi 200
```

## License

MIT
