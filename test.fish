#!/usr/bin/env fish
# Build the LaTeX stress-test PDF and run it through the OCR pipeline.

set -l project_dir (realpath (dirname (status -f)))
cd $project_dir

set -l tex_file test/test_document.tex
set -l pdf_file test/test_document.pdf
set -l output_dir test/output
set -l image_dir test/output/images
set -l pdf_basename (basename $pdf_file .pdf)
set -l text_file "$output_dir/$pdf_basename.txt"
set -l venv_python "$project_dir/.venv/bin/python"

if not test -x $venv_python
  set venv_python (command -v python3)
end

if not command -q pdflatex
  echo "pdflatex not found; install TeX Live/LaTeX first."
  exit 1
end

if not command -q python3
  echo "python3 not found; install Python 3 to continue."
  exit 1
end

echo "==> Building test PDF from $tex_file"
pdflatex -interaction=nonstopmode -halt-on-error -output-directory test $tex_file; or exit $status

echo "==> Cleaning auxiliary TeX files"
rm -f test/test_document.aux test/test_document.log test/test_document.out

echo "==> Running OCR pipeline"
mkdir -p $output_dir
$venv_python main.py $pdf_file -o $output_dir --keep-images -i $image_dir --lang eng+kor; or exit $status

# 1페이지 블록 감지 + JSON/시각화
set -l first_image "$image_dir/$pdf_basename"_page_001.png
if test -f $first_image
  echo "==> Running block detection on first page"
  env PYTHONPATH="$project_dir/src" $venv_python -c 'from pathlib import Path
from pdfocr.layout import detect_blocks, draw_blocks
from pdfocr.block_ocr import extract_blocks_to_json

image_path = Path("test/output/images/test_document_page_001.png")
json_out = Path("test/output/block_debug_page1.json")
annotated = Path("test/output/block_debug_page1.png")

blocks = detect_blocks(image_path, min_area=1200, merge_kernel=(18, 8))
extract_blocks_to_json(image_path, json_out, lang="eng+kor", min_area=1200, merge_kernel=(18, 8))
draw_blocks(image_path, blocks, annotated)

print(f"블록 {len(blocks)}개 감지 -> {json_out} / {annotated}")'
end

echo ""
echo "Test finished."
echo "PDF: $pdf_file"
echo "Extracted text: $text_file"
