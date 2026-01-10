FROM ubuntu:22.04

# 시스템 패키지 설치: poppler, tesseract(한글 포함), python, opencv 의존성
RUN apt-get update && \
    DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
      python3 python3-venv python3-pip \
      tesseract-ocr tesseract-ocr-kor \
      poppler-utils \
      libgl1-mesa-glx libglib2.0-0 \
      locales && \
    apt-get clean && rm -rf /var/lib/apt/lists/*

# 로케일 설정 (UTF-8)
RUN locale-gen en_US.UTF-8 && update-locale LANG=en_US.UTF-8
ENV LANG=en_US.UTF-8

WORKDIR /app

# 파이썬 의존성 설치
COPY requirements.txt .
RUN python3 -m pip install --no-cache-dir --upgrade pip && \
    python3 -m pip install --no-cache-dir -r requirements.txt

# 소스 복사
COPY src ./src
COPY pdfocr ./pdfocr
COPY main.py ./main.py
COPY README.md ./

# 출력 디렉토리 생성
RUN mkdir -p /output /input

# Python 경로 설정
ENV PYTHONPATH=/app/src

# 실행 기본값: help 출력
ENTRYPOINT ["/app/pdfocr"]
CMD ["--help"]

# 작업 디렉토리를 /work로 설정 (볼륨 마운트용)
WORKDIR /work
