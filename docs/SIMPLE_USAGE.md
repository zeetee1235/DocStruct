# Simple Usage Guide

## Setup

```bash
docker build -t pdfocr .
```

## Basic Usage

```bash
docker compose run --rm pdfocr /work/path/to/document.pdf
```

Creates `document.txt` in same directory.

## With Options

```bash
# Custom output directory
docker compose run --rm pdfocr /work/document.pdf -o /work/output

# Multiple files
docker compose run --rm pdfocr /work/pdfs/*.pdf --merge

# Custom language
docker compose run --rm pdfocr /work/document.pdf --lang kor

# Keep images
docker compose run --rm pdfocr /work/document.pdf --keep-images
```

## Notes

- Use `/work/...` paths inside container
- Current directory is mounted at `/work`
- Default language: eng+kor
- Output: `filename.pdf` → `filename.txt`
