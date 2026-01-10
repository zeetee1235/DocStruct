# Docker 배포

## 빠른 시작

```bash
# 이미지 빌드
docker build -t pdfocr .

# PDF 처리
docker compose run --rm pdfocr /work/test/document.pdf

# 출력 디렉토리 지정
docker compose run --rm pdfocr /work/document.pdf -o /work/output

# 여러 PDF 병합
docker compose run --rm pdfocr /work/pdfs/*.pdf --merge

# 영어+한국어
docker compose run --rm pdfocr /work/document.pdf --lang eng+kor
```

## Alias 설정

```fish
alias pdfocr='docker compose -f /path/to/pdfocr/docker-compose.yml run --rm pdfocr'
```

```bash
pdfocr /work/$(pwd)/document.pdf
```

## 사용 예시

### 단일 PDF

```bash
# 현재 디렉토리
docker run --rm -v $(pwd):/work pdfocr /work/document.pdf

# 절대 경로
docker run --rm -v /home/user/docs:/work pdfocr /work/document.pdf
```

### 일괄 처리

```bash
# 모든 PDF 병합
docker compose run --rm pdfocr /work/pdfs/*.pdf --merge -o /work/output

# 개별 처리
docker compose run --rm pdfocr /work/pdfs/*.pdf -o /work/output
```

### 고급 옵션

```bash
# 고해상도
docker compose run --rm pdfocr /work/document.pdf --dpi 600

# 영어 전용
docker compose run --rm pdfocr /work/document.pdf --lang eng

# 이미지 보존
docker compose run --rm pdfocr /work/document.pdf --keep-images -i /work/images
```

## 볼륨 마운트

```bash
-v <호스트_경로>:<컨테이너_경로>
```

예시:
- `-v $(pwd):/work` - 현재 디렉토리를 `/work`에 마운트
- `-v /home/user/pdfs:/input` - 특정 폴더를 `/input`에 마운트

## 이미지 관리

### 빌드 및 태그

```bash
# 최신
docker build -t pdfocr .

# 특정 버전
docker build -t pdfocr:v0.0.2 .
```
