#!/usr/bin/env fish
# Test Docker execution with same options as local test.fish

set -l project_dir (realpath (dirname (status -f)))
cd $project_dir

echo "==> Building Docker image"
docker build -t pdfocr . ; or exit $status

echo ""
echo "==> Running OCR with Docker (same options as test.fish)"
docker run --rm \
  -v "$PWD":/work -w /work \
  pdfocr \
  /work/test/test_document.pdf \
  -o /work/test/docker_output \
  --keep-images -i /work/test/docker_images \
  --lang eng+kor

echo ""
echo "==> Docker execution completed!"
echo "Output: test/docker_output/test_document.txt"
echo "Images: test/docker_images/"
echo ""
echo "Compare with local result:"
echo "  diff test/output/test_document.txt test/docker_output/test_document.txt"
