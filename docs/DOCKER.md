# Docker Deployment

## Quick Start

```bash
# Build image
docker build -t pdfocr .

# Process PDF
docker compose run --rm pdfocr /work/test/document.pdf

# Custom output directory
docker compose run --rm pdfocr /work/document.pdf -o /work/output

# Multiple PDFs
docker compose run --rm pdfocr /work/pdfs/*.pdf --merge

# English + Korean
docker compose run --rm pdfocr /work/document.pdf --lang eng+kor
```

## Setup Alias

```fish
alias pdfocr='docker compose -f /path/to/pdfocr/docker-compose.yml run --rm pdfocr'
```

```bash
pdfocr /work/$(pwd)/document.pdf
```

## Usage Examples

### Single PDF

```bash
# Current directory
docker run --rm -v $(pwd):/work pdfocr /work/document.pdf

# Absolute path
docker run --rm -v /home/user/docs:/work pdfocr /work/document.pdf
```

### Batch Processing

```bash
# Merge all PDFs
docker compose run --rm pdfocr /work/pdfs/*.pdf --merge -o /work/output

# Process individually
docker compose run --rm pdfocr /work/pdfs/*.pdf -o /work/output
```

### Advanced Options

```bash
# High resolution
docker compose run --rm pdfocr /work/document.pdf --dpi 600

# English only
docker compose run --rm pdfocr /work/document.pdf --lang eng

# Keep images
docker compose run --rm pdfocr /work/document.pdf --keep-images -i /work/images
```

## Volume Mounting

```bash
-v <host_path>:<container_path>
```

Examples:
- `-v $(pwd):/work` - Mount current directory to `/work`
- `-v /home/user/pdfs:/input` - Mount specific folder to `/input`

## Image Management

### Build and Tag

```bash
# Latest
docker build -t pdfocr .

# Specific version
docker build -t pdfocr:v0.0.2 .
```

### Cleanup

```bash
# Remove specific image
docker rmi pdfocr

# Remove unused images
docker image prune -a
```

## Troubleshooting

### Permission Issues

```bash
# Run as current user
docker run --rm -u $(id -u):$(id -g) -v $(pwd):/work pdfocr /work/document.pdf
```

### Memory Issues

```bash
# Increase memory limit
docker run --rm -m 4g -v $(pwd):/work pdfocr /work/large.pdf
```

## Deployment

### Docker Hub

```bash
docker login
docker tag pdfocr username/pdfocr:v0.0.2
docker push username/pdfocr:v0.0.2
```

### GitHub Container Registry

```bash
echo $GITHUB_TOKEN | docker login ghcr.io -u USERNAME --password-stdin
docker tag pdfocr ghcr.io/username/pdfocr:v0.0.2
docker push ghcr.io/username/pdfocr:v0.0.2
```

