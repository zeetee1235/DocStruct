# DocStruct CLI 사용법

이 문서는 DocStruct의 커맨드라인 인터페이스(CLI) 사용법을 상세히 설명합니다. (README.md와 별도)

## 1. 개요
DocStruct는 문서 구조 분석 및 변환을 위한 도구로, 다양한 입력 파일을 처리하여 구조화된 결과를 제공합니다. CLI를 통해 주요 기능을 사용할 수 있습니다.

## 2. 설치 및 환경설정
- Python 및 Rust 환경 필요
- 의존성 설치: `pip install -r requirements.txt` 및 `cargo build --release`
- (선택) 가상환경 권장: `python -m venv .venv && source .venv/bin/activate`

## 3. 기본 명령어

### 변환 실행
```
./run-gui convert <입력파일> [옵션]
```
- 예시: `./run-gui convert input.pdf --output-dir output_folder`

### 주요 옵션
- `--output-dir <경로>`: 결과 파일 저장 폴더 지정
- `--format <형식>`: 결과 형식 지정 (예: text, json, html)
- `--lang <언어>`: 언어 지정 (예: ko, en)
- `--debug`: 디버그 정보 출력
- `--no-ocr`: OCR 미사용(파서만 사용)
- `--no-parser`: 파서 미사용(OCR만 사용)
- `--help`: 사용법 출력

## 4. 출력 파일
- `document.txt`: 추출된 텍스트 결과
- `document.json`: 구조화된 블록 정보(JSON)
- `document.html`: 시각화 결과(옵션)

## 5. 예시
```
./run-gui convert sample.pdf --output-dir result --format json --lang ko
```

## 6. 고급 기능
- 2단 논문 등 복잡한 레이아웃 자동 감지 및 처리
- OCR/파서/퓨전 단계별 디버그 로그 확인 가능(`--debug`)
- 입력 파일로 PDF, 이미지, docx 등 지원

## 7. 참고
- 자세한 구조/아키텍처: docs/ARCHITECTURE.md
- GUI 사용법: gui/README.md (존재 시)

---

문의 및 기여 방법은 CONTRIBUTING.md 참고
