# Docker Usage Guide

## Build

```bash
docker build -t pdfocr .
```

## Usage

### docker run

```bash
# Basic
docker run --rm -v "$PWD":/work -w /work pdfocr /work/document.pdf

# With options
docker run --rm -v "$PWD":/work -w /work pdfocr \
  /work/document.pdf -o /work/output --keep-images
```

### docker-compose (recommended)

```bash
# Basic
docker compose run --rm pdfocr /work/document.pdf

# With options
docker compose run --rm pdfocr /work/document.pdf \
  -o /work/output --keep-images --lang kor
```

## Path Mapping

- Host `./test/document.pdf` → Container `/work/test/document.pdf`
- Host `./output` → Container `/work/output`

Always use `/work/...` paths inside container.

## Options

- `-o DIR`: Output directory (default: same as PDF)
- `-i DIR`: Image directory
- `-l LANG`: OCR language (default: eng+kor)
- `-d DPI`: Resolution (default: 300)
- `--keep-images`: Keep temporary images
- `--merge`: Merge multiple outputs

## Examples

```bash
# Multiple files
docker compose run --rm pdfocr /work/pdfs/*.pdf --merge

# High resolution
docker compose run --rm pdfocr /work/document.pdf --dpi 600

# Debug with images
docker compose run --rm pdfocr /work/document.pdf --keep-images -i /work/images
```

## Notes

- Output persists on host filesystem
- Default: `filename.pdf` → `filename.txt`
- Files created as root (use `sudo chown` if needed)

