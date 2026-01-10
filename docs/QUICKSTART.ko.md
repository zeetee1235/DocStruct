# pdfocr - 빠른 참조 가이드

## 설치

```bash
# 환경 설정
./setup.sh

# CLI 도구 설치 (선택사항)
./install.sh
```

## 기본 사용법

```bash
# 단일 PDF
pdfocr document.pdf

# 여러 PDF
pdfocr file1.pdf file2.pdf file3.pdf

# 와일드카드 사용
pdfocr *.pdf
pdfocr lectures/*.pdf
```

## 다른 디렉토리의 PDF 처리

```bash
# 상위 디렉토리
pdfocr ../document.pdf

# 다른 프로젝트
pdfocr ../other-project/files/lecture.pdf

# 절대 경로
pdfocr ~/Documents/important.pdf
pdfocr /home/user/pdfs/report.pdf
```

## 출력 제어

```bash
# 출력 디렉토리 지정
pdfocr document.pdf -o ~/output

# 임시 이미지 보존
pdfocr document.pdf --keep-images

# 이미지 디렉토리 지정
pdfocr document.pdf -i ./temp_images
```

## 병합 기능

```bash
# 여러 PDF를 하나의 텍스트로
pdfocr lec1.pdf lec2.pdf lec3.pdf --merge

# 다른 위치의 PDF들 병합
pdfocr ../proj1/doc.pdf ../proj2/doc.pdf --merge -o ~/merged
```

## OCR 옵션

```bash
# 고해상도 (더 정확)
pdfocr document.pdf --dpi 600

# 영어 문서
pdfocr document.pdf --lang eng

# 한국어+영어 혼합
pdfocr document.pdf --lang kor+eng
```

## 실제 예시

### 과목별 정리

```bash
# 수학 강의 노트
pdfocr ~/courses/math/*.pdf --merge -o ~/notes/math

# 물리 강의 노트
pdfocr ~/courses/physics/*.pdf --merge -o ~/notes/physics
```

### 프로젝트 문서 처리

```bash
# 현재 위치: /home/dev/pdfocr
# 다른 프로젝트의 문서 처리
pdfocr ../research/papers/*.pdf --merge -o ~/research/extracted
```

### 대량 처리

```bash
# 여러 디렉토리의 모든 PDF
pdfocr ~/Documents/2024/*/*.pdf --merge -o ~/archive/2024_all.txt
```

## 도움말

```bash
pdfocr --help
```
