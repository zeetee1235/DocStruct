# pdfocr

PDF to text extraction CLI tool using OCR.

## Quick Start

```bash
# Build Docker image
docker build -t pdfocr .

# Process PDF
docker compose run --rm pdfocr /work/path/to/document.pdf
```

Output: `document.txt` in same directory.

## Documentation

- [docs/SIMPLE_USAGE.md](docs/SIMPLE_USAGE.md) - Simplest usage guide
- [docs/DOCKER_QUICKSTART.md](docs/DOCKER_QUICKSTART.md) - Docker quick reference
- [docs/DOCKER_USAGE.md](docs/DOCKER_USAGE.md) - Detailed Docker usage
- [docs/QUICKSTART.md](docs/QUICKSTART.md) | [한국어](docs/QUICKSTART.ko.md) - Quick start guide
- [docs/DOCKER.md](docs/DOCKER.md) | [한국어](docs/DOCKER.ko.md) - Docker deployment
- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | [한국어](docs/ARCHITECTURE.ko.md) - Architecture overview
- [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md) | [한국어](docs/DEVELOPMENT.ko.md) - Development guide

## Pipeline

1. PDF to Image: Convert pages to PNG (pdf2image)
2. OCR: Extract text (Tesseract OCR)
3. Output: Save as UTF-8 text file

## Local Installation

### Setup

```bash
chmod +x setup.sh
./setup.sh
```

Installs dependencies, creates virtual environment, installs packages.

### Install CLI (Optional)

```bash
./install.sh
```

Options: system-wide, user-local, or development mode.

## Docker Usage

### Basic

```bash
# Build
docker build -t pdfocr .

# Run
docker compose run --rm pdfocr /work/document.pdf
```

### With Options

```bash
# Custom output directory
docker compose run --rm pdfocr /work/document.pdf -o /work/output

# Multiple files
docker compose run --rm pdfocr /work/pdfs/*.pdf --merge

# Custom language (default: eng+kor)
docker compose run --rm pdfocr /work/document.pdf --lang kor

# Keep images for debugging
docker compose run --rm pdfocr /work/document.pdf --keep-images
```

## CLI Usage

After installation:

```bash
# Simple
pdfocr document.pdf

# Multiple files
pdfocr *.pdf --merge

# Custom output
pdfocr document.pdf -o ./output
```

# Keep images for debugging
docker compose run --rm pdfocr /work/document.pdf --keep-images -i /work/images

# All options
docker compose run --rm pdfocr \
  /work/test/test_document.pdf \
  -o /work/test/output \
  --keep-images -i /work/images \
  --lang eng+kor --dpi 300
```

**Key Points**:
- 📁 Output saves to **same directory** as PDF by default
- 📄 Creates `filename.txt` from `filename.pdf`
- 🗂️ Use `/work/...` paths inside container
- 💾 All files persist on your host filesystem

See [Docker Documentation](docs/DOCKER.md) ([한국어](docs/DOCKER.ko.md)) for detailed usage.

### 4. Usage

#### Docker (simplest, no installation):

```bash
# Just specify the PDF - output auto-saves to same directory
docker compose run --rm pdfocr /work/path/to/document.pdf

# With options
docker compose run --rm pdfocr /work/document.pdf -o /work/output --lang eng+kor
```

#### After CLI installation:

```bash
# Simple - creates document.txt in same directory
pdfocr ~/Documents/lecture.pdf

# Multiple files
pdfocr /path/to/*.pdf --merge

# Custom output directory
pdfocr document.pdf -o ./output
```

## Project Structure

```
pdfocr/
├── src/pdfocr/          # Main package
│   ├── main.py          # Pipeline entry point
│   ├── pdf_to_image.py  # PDF converter
│   ├── image_to_text.py # OCR module
│   ├── layout.py        # Layout analysis
│   ├── block_ocr.py     # Block-based OCR
│   └── types.py         # Type definitions
├── docs/                # Documentation
│   ├── SIMPLE_USAGE.md
│   ├── DOCKER_QUICKSTART.md
│   ├── DOCKER_USAGE.md
│   ├── QUICKSTART.md / QUICKSTART.ko.md
│   ├── DOCKER.md / DOCKER.ko.md
│   ├── ARCHITECTURE.md / ARCHITECTURE.ko.md
│   └── DEVELOPMENT.md / DEVELOPMENT.ko.md
├── test/                # Test files and scripts
├── pdfocr               # CLI executable
├── main.py              # CLI entry point
├── requirements.txt     # Python dependencies
├── setup.sh             # Environment setup
├── install.sh           # CLI installation
├── Dockerfile           # Docker image definition
└── docker-compose.yml   # Docker compose config
```

## Requirements

System: poppler-utils, tesseract-ocr
Python: pdf2image, pytesseract, Pillow, opencv-python

## CLI Options

```
pdfocr [OPTIONS] <PDF_FILES...>

Options:
  -o, --output-dir DIR  Output directory (default: same as PDF)
  -i, --image-dir DIR   Temporary image directory
  -l, --lang LANG       OCR language (default: eng+kor)
  -d, --dpi DPI         Image resolution (default: 300)
  --keep-images         Keep temporary images
  --merge              Merge all outputs into one file
```

## Examples

```bash
# Basic
pdfocr document.pdf

# Multiple files
pdfocr *.pdf --merge

# Custom options
pdfocr document.pdf -o ./output --dpi 600 --keep-images
```
