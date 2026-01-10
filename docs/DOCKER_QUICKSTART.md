# Docker Quick Start

## Build

```bash
docker build -t pdfocr .
```

## Run

```bash
# Basic
docker run --rm -v "$PWD":/work -w /work pdfocr /work/document.pdf

# Using docker-compose
docker compose run --rm pdfocr /work/document.pdf
```

Output: `document.txt` in same directory.

## Options

```bash
# Custom output
docker compose run --rm pdfocr /work/document.pdf -o /work/output

# Multiple files
docker compose run --rm pdfocr /work/pdfs/*.pdf --merge

# Custom language
docker compose run --rm pdfocr /work/document.pdf --lang kor

# Keep images
docker compose run --rm pdfocr /work/document.pdf --keep-images
```

See [DOCKER_USAGE.md](DOCKER_USAGE.md) for full documentation.
