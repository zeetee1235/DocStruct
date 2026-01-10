# 아키텍처 및 개발 가이드

## 파이프라인 개요
- PDF → 이미지 변환(pdf2image/poppler)
- 이미지 → 텍스트 추출(pytesseract/Tesseract)
- 페이지별 텍스트 파일 저장, 필요 시 병합 출력

## 주요 모듈
- `src/pdfocr/main.py`: CLI 진입점, 인자 파싱, 경로 해석, 파이프라인 오케스트레이션.
- `src/pdfocr/pdf_to_image.py`: PDF를 PNG로 변환, DPI 설정 지원.
- `src/pdfocr/image_to_text.py`: 이미지 단위 OCR, 다국어 지원, 페이지별 텍스트 저장.
- `src/pdfocr/layout.py`: OpenCV 기반 블록 감지, 블록 시각화.
- `src/pdfocr/block_ocr.py`: 감지된 블록 단위 OCR → JSON 출력.
- `src/pdfocr/types.py`: 공통 경로 타입 정의.

## 개발 환경
1) 시스템 의존성: poppler-utils, tesseract-ocr(+kor).  
2) Python: `./setup.sh` 또는 venv 생성 후 `pip install -r requirements.txt`.  
3) 실행 확인: `python main.py --help`, `./test.fish` (샘플 PDF 변환).

## 코드 스타일 요약
- 들여쓰기 4칸, 최대 줄 길이 120자.
- 임포트 순서: 표준 → 서드파티 → 로컬.
- 함수/클래스/상수는 snake_case/PascalCase/UPPER_SNAKE_CASE.
- Docstring은 간단히 목적/인자/반환/예외만 기술.

## 테스트/검증
- 통합 테스트: `./test.fish` (PDF→이미지→텍스트, 블록 디버그 포함).
- Docker 환경: `docker build -t pdfocr .` 후 `docker compose run --rm pdfocr /work/test/test_document.pdf -o /work/test/output --lang eng+kor --keep-images`.

## 개발 워크플로우
- 기능 설계 후 브랜치 생성(`git checkout -b feature/...`).
- 구현 시 오류 처리/로그 출력/경로 해석 일관성 유지.
- 커밋: 작은 단위로, 메시지는 `feat/fix/chore` 등 간단 태그 + 내용.

### 환경 변수
현재 사용되지 않음.

### 명령줄 옵션
- `-o, --output-dir` - 출력 디렉토리
- `-i, --image-dir` - 임시 이미지 디렉토리
- `-l, --lang` - OCR 언어 코드 (기본값: "kor")
- `-d, --dpi` - 이미지 해상도 (기본값: 300)
- `--keep-images` - 임시 이미지 보존
- `--merge` - 여러 출력을 하나의 파일로 병합

## 의존성

### Python 패키지
- `pdf2image` - PDF to Image 변환
- `pytesseract` - Tesseract OCR 래퍼
- `Pillow>=11.0.0` - 이미지 처리 (Python 3.13 호환)
- `opencv-python>=4.8.0` - 컴퓨터 비전 (선택사항, 레이아웃용)
- `numpy>=1.24.0` - 수치 연산 (선택사항, 레이아웃용)

### 시스템 패키지
- `poppler-utils` - PDF 렌더링 (pdftoppm, pdfinfo)
- `tesseract-ocr` - OCR 엔진
- `tesseract-langpack-kor` - 한국어 지원 (선택사항)

## 오류 처리

### 파일을 찾을 수 없음
- 처리 전 모든 입력 경로 유효성 검사
- 파일 경로와 함께 명확한 오류 메시지 제공

### OCR 실패
- 나머지 파일 처리 계속
- 파이프라인을 중단하지 않고 오류 기록
- 실패한 추출에 대해 빈 문자열 반환

### 리소스 정리
- 기본적으로 임시 이미지 삭제
- `--keep-images`로 보존 가능
- 정리 실패를 우아하게 처리

## 성능 고려사항

### 메모리 사용량
- 이미지를 한 번에 하나씩 로드
- 처리 후 임시 파일 정리
- 일괄 처리를 위한 메모리 누적 없음

### 처리 속도
속도에 영향을 미치는 요인:
- DPI 설정 (높을수록 느리지만 더 정확)
- 페이지 수
- 이미지 복잡도
- 시스템 OCR 성능

일반적인 속도:
- 300 DPI: 페이지당 약 2-5초
- 600 DPI: 페이지당 약 5-10초

### 병렬화
현재 단일 스레드. 향후 개선 사항:
- 병렬 페이지 처리
- 일괄 OCR 작업
- 파일 작업을 위한 비동기 I/O

## 향후 개선사항

### 계획된 기능
- [ ] 여러 페이지에 대한 병렬 처리
- [ ] PDF 텍스트 레이어 감지 (텍스트가 있으면 OCR 건너뛰기)
- [ ] 추출된 텍스트에 대한 신뢰도 점수
- [ ] 더 나은 표 추출
- [ ] 수학 공식 인식
- [ ] 긴 작업을 위한 진행률 표시줄

### 잠재적 개선사항
- 설정 파일 지원
- 사용자 정의 프로세서를 위한 플러그인 시스템
- 웹 API 인터페이스
- GUI 애플리케이션
- Docker 컨테이너

## 테스트

### 테스트 스위트
`test/` 디렉토리에 위치:
- `test_document.tex` - 테스트 PDF용 LaTeX 소스
- `test_document.pdf` - 생성된 테스트 문서
- `test.fish` - 통합 테스트 스크립트

### 테스트 실행
```bash
./test.fish
```

다음을 수행합니다:
1. LaTeX에서 테스트 PDF 빌드
2. OCR 파이프라인 실행
3. 블록 감지 수행
4. 출력 검증

## 로깅

### 로그 레벨
- `INFO` - 일반 작업 메시지
- `DEBUG` - 상세한 처리 정보
- `WARNING` - 중요하지 않은 문제
- `ERROR` - 실패 메시지

### 로그 출력
현재 stdout으로 로깅. 향후: 설정 가능한 로그 파일.

## 호환성

### Python 버전
- Python 3.13+ (권장)
- Python 3.11+ (지원)

### 운영 체제
- Linux (Fedora, Ubuntu, Debian) - 주요
- macOS - 지원
- Windows - 테스트되지 않음

### 알려진 제한사항
- 오른쪽에서 왼쪽 텍스트 최적화되지 않음
- 세로 텍스트는 회전 필요할 수 있음
- 손글씨 텍스트는 정확도가 낮음
- 수학 기호는 종종 잘못 인식됨
