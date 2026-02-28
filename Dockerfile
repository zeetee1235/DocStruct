# syntax=docker/dockerfile:1

FROM rust:bookworm AS builder
WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY ocr ./ocr

RUN cargo build --release

FROM debian:bookworm-slim AS runtime
WORKDIR /app

ARG INSTALL_PIX2TEX=0

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        ca-certificates \
        python3 \
        python3-pip \
        python3-venv \
        tesseract-ocr \
        tesseract-ocr-kor \
        poppler-utils \
        libgl1 \
        libglib2.0-0 \
    && rm -rf /var/lib/apt/lists/*

COPY requirements.txt /tmp/requirements.txt
RUN python3 -m venv /opt/venv
ENV PATH="/opt/venv/bin:${PATH}"
RUN sed -i '/^pix2tex\[gui\].*/d' /tmp/requirements.txt \
    && pip install --no-cache-dir -r /tmp/requirements.txt \
    && if [ "$INSTALL_PIX2TEX" = "1" ]; then pip install --no-cache-dir 'pix2tex[gui]>=0.1.2'; fi

COPY --from=builder /app/target/release/docstruct /usr/local/bin/docstruct
COPY ocr ./ocr

ENTRYPOINT ["docstruct"]
CMD ["--help"]
