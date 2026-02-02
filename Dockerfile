FROM ubuntu:22.04

# 시스템 패키지 설치: poppler, tesseract(한글 포함), python, opencv 의존성, uv 설치용 도구
RUN apt-get update && \
    DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
      python3 python3-venv curl ca-certificates \
      tesseract-ocr tesseract-ocr-kor \
      poppler-utils \
      libgl1-mesa-glx libglib2.0-0 \
      locales && \
    apt-get clean && rm -rf /var/lib/apt/lists/*

# 로케일 설정 (UTF-8)
RUN locale-gen en_US.UTF-8 && update-locale LANG=en_US.UTF-8
ENV LANG=en_US.UTF-8

WORKDIR /app

# uv 설치 (standalone 바이너리)
RUN curl -LsSf https://astral.sh/uv/install.sh | sh -s -- --yes && \
  ln -s "$HOME/.local/bin/uv" /usr/local/bin/uv

# 파이썬 의존성 설치 (uv 사용)
COPY requirements.txt .
RUN uv venv /opt/venv && \
  . /opt/venv/bin/activate && \
  uv pip install -r requirements.txt

# 소스 복사
COPY src ./src
COPY pdfocr ./pdfocr
COPY main.py ./main.py
COPY README.md ./

# 출력 디렉토리 생성
RUN mkdir -p /output /input

# Python 경로 및 가상환경 설정
ENV PYTHONPATH=/app/src
ENV VIRTUAL_ENV=/opt/venv
ENV PATH="/opt/venv/bin:${PATH}"

# 실행 기본값: help 출력
ENTRYPOINT ["/app/pdfocr"]
CMD ["--help"]

# 작업 디렉토리를 /work로 설정 (볼륨 마운트용)
WORKDIR /work
