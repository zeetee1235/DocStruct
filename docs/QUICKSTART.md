# pdfocr - Quick Reference

## Installation

```bash
# Setup environment
./setup.sh

# Install CLI tool (optional)
./install.sh
```

## Basic Usage

```bash
# Single PDF
pdfocr document.pdf

# Multiple PDFs
pdfocr file1.pdf file2.pdf file3.pdf

# Wildcards
pdfocr *.pdf
pdfocr lectures/*.pdf
```

## Processing PDFs from Different Locations

```bash
# Parent directory
pdfocr ../document.pdf

# Other project
pdfocr ../other-project/files/lecture.pdf

# Absolute path
pdfocr ~/Documents/important.pdf
pdfocr /home/user/pdfs/report.pdf
```

## Output Control

```bash
# Specify output directory
pdfocr document.pdf -o ~/output

# Keep temporary images
pdfocr document.pdf --keep-images

# Specify image directory
pdfocr document.pdf -i ./temp_images
```

## Merge Feature

```bash
# Merge multiple PDFs into one text file
pdfocr lec1.pdf lec2.pdf lec3.pdf --merge

# Merge PDFs from different locations
pdfocr ../proj1/doc.pdf ../proj2/doc.pdf --merge -o ~/merged
```

## OCR Options

```bash
# High resolution (better accuracy)
pdfocr document.pdf --dpi 600

# English document
pdfocr document.pdf --lang eng

# Mixed Korean + English
pdfocr document.pdf --lang kor+eng
```

## Examples

### Course Materials

```bash
# Math lectures
pdfocr ~/courses/math/*.pdf --merge -o ~/notes/math

# Physics lectures
pdfocr ~/courses/physics/*.pdf --merge -o ~/notes/physics
```

### Project Documents

```bash
# Current location: /home/dev/pdfocr
# Process documents from another project
pdfocr ../research/papers/*.pdf --merge -o ~/research/extracted
```

### Batch Processing

```bash
# All PDFs from multiple directories
pdfocr ~/Documents/2024/*/*.pdf --merge -o ~/archive/2024_all.txt
```

## Help

```bash
pdfocr --help
```
