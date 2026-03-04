<p align="left">
  <img src="./assets/docstruct_logo.png" alt="DocStruct 로고" width="220" />
</p>

# DocStruct (한국어 문서)

![Build](https://img.shields.io/github/actions/workflow/status/zeetee1235/DocStruct/ci.yml?branch=main&style=for-the-badge)
![Docker Image](https://img.shields.io/github/actions/workflow/status/zeetee1235/DocStruct/docker-image.yml?style=for-the-badge&label=Docker%20Image)
![Last Commit](https://img.shields.io/github/last-commit/zeetee1235/DocStruct?style=for-the-badge)
![License: MIT](https://img.shields.io/badge/License-MIT-1f2937?style=for-the-badge)

DocStruct는 Parser + OCR + Fusion 파이프라인으로 문서 구조를 복원하고, 구조화된 결과를 내보냅니다.

지원 입력 형식: **PDF, DOCX, PPT, PPTX**

영문 문서: [../README.md](../README.md)

## GUI 스냅샷

<img src="./assets/gui.png" alt="DocStruct GUI" width="100%" />

## 빠른 시작 (GUI)

실행:

```bash
./run-gui
```

`run-gui`가 자동으로 수행하는 작업:
- `.venv` 생성/재사용
- `requirements.txt` 기반 Python 패키지 설치
- Tauri 데스크톱 앱 실행

앱 사용 순서:
1. 하나 이상 입력 파일 선택
2. 출력 디렉터리 선택(선택 사항)
3. DPI 설정 (기본 `200`)
4. **Convert** 클릭

출력 디렉터리를 비워도 변환은 진행되며, 추출 텍스트를 앱 내부에서 바로 확인할 수 있습니다.
**Copy Text** 버튼으로 클립보드 복사가 가능합니다.

## 설치 요구사항

필수 런타임 도구:
- Rust toolchain (`cargo`)
- Python 3.8+
- `tesseract` (예: `eng`, `kor` 언어 데이터)
- `poppler-utils` (`pdfinfo`, `pdftotext`, `pdftoppm`)

Linux GUI 빌드/실행 참고:
- WebKitGTK 패키지
- Wayland 개발/런타임 패키지 (`wayland-client.pc` 제공 패키지)

선택 사항(수식 OCR):

```bash
pip install --user 'pix2tex[gui]>=0.1.2'
```

## CLI 사용법 (Linux 중심)

빌드:

```bash
cargo build --release
```

단일 파일 변환:

```bash
./target/release/docstruct convert input.pdf -o output_dir --debug
```

여러 파일 변환:

```bash
./target/release/docstruct batch file1.pdf file2.pdf -o output_dir --debug
```

문서 정보 확인:

```bash
./target/release/docstruct info input.pdf
```

주요 옵션:
- `--dpi <int>`: OCR 렌더링 DPI (기본 `200`)
- `--debug`: 디버그 산출물 저장
- `--quiet`: 콘솔 로그 최소화

## 배포 정책

`v*` 태그 푸시 시 GitHub Releases에 자동 업로드됩니다.

- **Windows/macOS**: GUI 데스크톱 설치 파일(Tauri)
- **Linux**:
  - CLI 바이너리/패키지
  - GUI 패키지 (`.deb`, `.rpm`)

워크플로우:
- [`../.github/workflows/release.yml`](../.github/workflows/release.yml): Linux CLI 릴리즈
- [`../.github/workflows/gui-release.yml`](../.github/workflows/gui-release.yml): GUI 릴리즈 (Windows/macOS/Linux GUI)

## GUI 설치 파일 로컬 빌드

```bash
./scripts/build-gui-app.sh
```

출력 위치:
- `gui/src-tauri/target/release/bundle/`

Linux에서는 기본적으로 `deb`, `rpm`을 빌드합니다.

## 출력 구조

```text
output_dir/
├── document.json
├── document.md
├── document.txt
├── page_001.md
├── page_001.txt
├── figures/
│   └── page_NNN_TYPE__NN.png
└── debug/
    ├── page_001.html
    └── page_001-1.png
```

## 개발

```bash
cargo build
cargo test
cargo test parser::hangul
```

아키텍처 문서: [ARCHITECTURE.md](./ARCHITECTURE.md)  
기여 가이드: [../CONTRIBUTING.md](../CONTRIBUTING.md)
