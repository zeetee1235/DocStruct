# DocStruct 아키텍처 문서

## 📋 목차

1. [프로젝트 개요](#프로젝트-개요)
2. [시스템 아키텍처](#시스템-아키텍처)
3. [핵심 컴포넌트](#핵심-컴포넌트)
4. [데이터 흐름](#데이터-흐름)
5. [모듈 상세 설명](#모듈-상세-설명)
6. [데이터 모델](#데이터-모델)
7. [실행 흐름](#실행-흐름)

---

## 프로젝트 개요

### 목표
**DocStruct**는 PDF 문서의 구조를 복원하는 시스템입니다. PDF Parser와 OCR(광학 문자 인식)을 **크로스체킹**하여 정확도를 높이고, 각 텍스트 블록의 출처와 신뢰도를 제공합니다.

### 주요 특징
- **이중 검증**: Parser Track과 OCR Track이 독립적으로 문서를 분석
- **Fusion Engine**: 두 결과를 비교·정렬·병합하여 최종 구조 생성
- **신뢰도 스코어링**: 0~1 사이의 confidence 값으로 각 블록의 품질 평가
- **Provenance 추적**: 각 데이터가 parser/ocr/fused 중 어디서 왔는지 기록
- **디버그 뷰어**: HTML 기반 시각화 도구로 parser/ocr/fused 레이어를 색상으로 구분

### 입력/출력
- **입력**: PDF 파일
- **출력**:
  - `document.json`: 최종 구조화된 문서 데이터
  - `debug/page_NNN.html`: 각 페이지의 디버그 시각화 페이지

---

## 시스템 아키텍처

### 전체 구조도

```
┌────────────────────────────────────────────────────────────┐
│                        DocStruct                           │
│                                                            │
│  ┌─────────────┐          ┌──────────────┐                 │
│  │   PDF Input │          │  Pipeline    │                 │
│  │   (*.pdf)   │─────────▶│  Controller  │                 │
│  └─────────────┘          └──────┬───────┘                 │
│                                  │                         │
│         ┌────────────────────────┼──────────────────┐      │
│         │                        │                  │      │
│         ▼                        ▼                  ▼      │
│  ┌──────────────┐        ┌──────────────┐   ┌───────────┐  │
│  │ Parser Track │        │  OCR Track   │   │  Renderer │  │
│  │              │        │              │   │  (Image)  │  │
│  │ • PdfReader  │        │ • PyBridge   │   └───────────┘  │
│  │ • TextExtr   │        │ • LayoutBld  │                  │
│  │ • LayoutBld  │        │              │                  │
│  └──────┬───────┘        └──────┬───────┘                  │
│         │                       │                          │
│         │   PageHypothesis A    │  PageHypothesis B        │
│         └───────────┬───────────┘                          │
│                     ▼                                      │
│            ┌─────────────────┐                             │
│            │ Fusion Engine   │                             │
│            │                 │                             │
│            │ • Align         │                             │
│            │ • Compare       │                             │
│            │ • Resolve       │                             │
│            │ • Finalize      │                             │
│            └────────┬────────┘                             │
│                     ▼                                      │
│            ┌─────────────────┐                             │
│            │   PageFinal     │                             │
│            │ (DocumentFinal) │                             │
│            └────────┬────────┘                             │
│                     │                                      │
│         ┌───────────┴───────────┐                          │
│         ▼                       ▼                          │
│  ┌──────────────┐      ┌────────────────┐                  │
│  │ JSON Export  │      │  HTML Debug    │                  │
│  │ document.json│      │  page_NNN.html │                  │
│  └──────────────┘      └────────────────┘                  │
└────────────────────────────────────────────────────────────┘
```

### 기술 스택
- **주 언어**: Rust (시스템 코어)
- **보조 언어**: Python (OCR 브리지)
- **빌드 시스템**: Cargo (Rust), uv (Python)
- **주요 라이브러리**:
  - Rust: `clap`, `serde`, `serde_json`, `image`, `strsim`, `thiserror`, `anyhow`
  - Python: `pytesseract`, `pdf2image`, `opencv-python`, `Pillow`

---

## 핵심 컴포넌트

### 1. Pipeline (`src/pipeline.rs`)

**역할**: 전체 문서 처리 프로세스를 조율하는 메인 컨트롤러

```rust
pub fn build_document(config: &PipelineConfig) -> Result<DocumentFinal>
```

**동작**:
1. PDF 파일을 읽어 페이지 수 확인
2. 각 페이지마다:
   - 이미지로 렌더링
   - Parser Track으로 분석 → `PageHypothesis` A
   - OCR Track으로 분석 → `PageHypothesis` B
   - Fusion Engine으로 병합 → `PageFinal`
   - 디버그 정보 첨부
3. 모든 페이지를 `DocumentFinal`로 패키징

**설정 파라미터** (`PipelineConfig`):
- `input`: PDF 파일 경로
- `output`: 출력 디렉토리
- `dpi`: 렌더링 해상도 (기본 200)

---

### 2. Parser Track (`src/parser/`)

**역할**: PDF 내부의 텍스트 데이터를 직접 추출

#### 2.1 PdfReader (`pdf_reader.rs`)
```rust
pub fn page_count(&self) -> usize
```
- PDF 파일을 열고 페이지 수를 반환
- *현재는 stub 구현으로 항상 1 반환 (추후 실제 PDF 라이브러리 통합 필요)*

#### 2.2 TextExtractor (`text_extractor.rs`)
```rust
pub fn extract_glyph_runs(pdf_path: &Path, page_idx: usize) -> Vec<GlyphRun>
```
- PDF에서 glyph(문자 단위) 데이터와 bounding box 추출
- *현재는 stub으로 더미 데이터 반환*

#### 2.3 ParserLayoutBuilder (`layout_builder.rs`)
```rust
impl ParserTrack for ParserLayoutBuilder {
    fn analyze_page(&self, pdf_path: &Path, page_idx: usize) -> Result<PageHypothesis>
}
```
- Glyph run을 Line으로 그룹핑
- Line을 Block으로 그룹핑
- `Provenance::Parser` 태그와 함께 `PageHypothesis` 생성
- 현재는 모든 glyph를 하나의 TextBlock으로 병합

---

### 3. OCR Track (`src/ocr/`)

**역할**: 렌더링된 페이지 이미지에서 텍스트를 광학적으로 인식

#### 3.1 PageRenderer (`renderer.rs`)
```rust
pub fn render_page(&self, pdf_path: &Path, page_idx: usize) -> Result<RenderedPage>
```
- PDF 페이지를 PNG 이미지로 변환
- DPI 설정에 따라 해상도 조정
- *현재는 빈 흰색 이미지 생성 (추후 실제 렌더링 엔진 통합 필요)*

#### 3.2 OcrBridge (`bridge.rs`)
```rust
pub fn run(&self, image_path: &Path) -> Result<Vec<OcrToken>>
```
- Python OCR 스크립트 (`ocr/bridge/ocr_bridge.py`) 실행
- 이미지 경로를 전달하고 JSON 형식의 OCR 결과 수신
- 각 토큰은 `text`와 `bbox` 정보 포함

**Python 브리지 스크립트** (`ocr/bridge/ocr_bridge.py`):
- 현재는 빈 배열 반환 (placeholder)
- 추후 tesseract/paddleocr/easyocr 등 실제 OCR 엔진 통합 예정

#### 3.3 OcrLayoutBuilder (`layout_builder.rs`)
```rust
impl OcrTrack for OcrLayoutBuilder {
    fn analyze_page(&self, rendered_image: &Path, page_idx: usize) -> Result<PageHypothesis>
}
```
- OCR 토큰들을 Line과 Block으로 그룹핑
- `Provenance::Ocr` 태그 부여
- 현재는 모든 토큰을 하나의 TextBlock으로 병합

---

### 4. Fusion Engine (`src/fusion/`)

**역할**: Parser와 OCR의 두 가지 가설을 비교·정렬·병합하여 최종 결과 생성

#### 4.1 Align (`align.rs`)
```rust
pub fn align_blocks(a_blocks: &[Block], b_blocks: &[Block]) -> AlignmentResult
```

**알고리즘**:
1. 각 Parser Block에 대해 가장 유사한 OCR Block 찾기
2. 유사도 점수 계산:
   - IoU (Intersection over Union)
   - 중심점 거리
   - 블록 타입 일치 여부 보너스
3. 임계값 이상이면 매칭 쌍으로 분류
4. 매칭되지 않은 블록들은 각각 별도 리스트로 관리

**출력**:
```rust
pub struct AlignmentResult {
    pub matched: Vec<MatchedPair>,       // 매칭된 쌍
    pub unmatched_a: Vec<Block>,         // Parser만 있는 블록
    pub unmatched_b: Vec<Block>,         // OCR만 있는 블록
}
```

#### 4.2 Compare (`compare.rs`)
```rust
pub fn text_similarity(a: &str, b: &str) -> f32
```

**텍스트 유사도 계산**:
1. **Normalized Levenshtein Distance**: 편집 거리 기반 유사도
2. **Token Overlap**: 단어 집합의 Jaccard 유사도
3. **Numeric Mismatch Penalty**: 숫자가 다르면 -0.1 감점
4. 최종 점수는 0.0~1.0 사이로 정규화

#### 4.3 Resolve (`resolve.rs`)
```rust
pub fn resolve_blocks(alignment: &AlignmentResult) -> Vec<Block>
```

**블록 해결 전략**:

**매칭된 쌍의 경우**:
- 텍스트 유사도 ≥ 0.7 → `Provenance::Fused` (Parser 텍스트 선택)
- 텍스트 유사도 < 0.7 → `Provenance::Parser` (Parser 우선)
- 신뢰도는 `score_confidence()` 함수로 계산

**매칭되지 않은 블록**:
- Parser만: `Provenance::Parser`로 승격
- OCR만: `Provenance::Ocr`로 승격

**디버그 정보 첨부**:
```rust
pub struct BlockDebug {
    pub parser_text: Option<String>,
    pub ocr_text: Option<String>,
    pub final_text: Option<String>,
    pub similarity: Option<f32>,
}
```

#### 4.4 Finalize (`finalize.rs`)
```rust
pub fn classify_page(parser: &PageHypothesis, ocr: &PageHypothesis) -> PageClass
```

**페이지 분류**:
- `Digital`: Parser glyph가 많고 (>200), 이미지 커버리지 낮음 (<0.3)
- `Scanned`: Parser glyph가 적고 (<50), OCR 텍스트 밀도 높음 (>0.5)
- `Hybrid`: 그 외 중간 케이스

---

### 5. Core Models (`src/core/`)

#### 5.1 Geometry (`geometry.rs`)
```rust
pub struct BBox {
    pub x0: f32, pub y0: f32,
    pub x1: f32, pub y1: f32,
}
```

**핵심 메서드**:
- `width()`, `height()`, `area()`: 기하학적 속성
- `center()`: 중심점 좌표
- `union()`: 두 bbox의 합집합
- `iou()`: Intersection over Union 계산
- `center_distance()`: 중심점 간 거리

#### 5.2 Model (`model.rs`)

**핵심 데이터 구조**:

```rust
pub struct DocumentFinal {
    pub pages: Vec<PageFinal>,
}

pub struct PageFinal {
    pub page_idx: usize,
    pub class: PageClass,           // Digital/Scanned/Hybrid
    pub blocks: Vec<Block>,
    pub width: u32,
    pub height: u32,
    pub debug: Option<PageDebug>,   // parser/ocr 원본 블록 저장
}

pub struct PageHypothesis {
    pub page_idx: usize,
    pub blocks: Vec<Block>,
    pub width: u32,
    pub height: u32,
}

pub enum Block {
    TextBlock {
        bbox: BBox,
        lines: Vec<Line>,
        confidence: f32,
        source: Provenance,
        debug: Option<BlockDebug>,
    },
    TableBlock { /* ... */ },
    FigureBlock { /* ... */ },
    MathBlock { /* ... */ },
}

pub struct Line {
    pub spans: Vec<Span>,
}

pub struct Span {
    pub text: String,
    pub bbox: BBox,
    pub source: Provenance,
    pub style: Option<TextStyle>,
}

pub enum Provenance {
    Parser,    // PDF parser에서 추출
    Ocr,       // OCR 엔진에서 인식
    Fused,     // 두 결과를 융합
}
```

#### 5.3 Confidence (`confidence.rs`)
```rust
pub fn score_confidence(
    has_parser: bool,
    has_ocr: bool,
    similarity: Option<f32>,
    geometry_good: bool,
) -> f32
```

**신뢰도 계산 로직**:
- Parser 존재: +0.4
- OCR 존재: +0.3
- 텍스트 유사도 ≥ 0.9: +0.3
- 텍스트 유사도 ≥ 0.7: +0.15
- 텍스트 유사도 < 0.7: -0.2
- 기하학적 정렬 양호: +0.1
- 기하학적 정렬 불량: -0.1
- 최종 점수는 0.0~1.0으로 클램핑

---

### 6. Export (`src/export/`)

#### 6.1 JSON Export (`json_export.rs`)
```rust
impl Exporter for JsonExporter {
    fn export(&self, document: &DocumentFinal) -> Result<()>
}
```
- `DocumentFinal`을 JSON으로 직렬화
- `output_dir/document.json`에 저장
- Pretty-print 형식

#### 6.2 HTML Debug Export (`html_debug_export.rs`)
```rust
impl Exporter for HtmlDebugExporter {
    fn export(&self, document: &DocumentFinal) -> Result<()>
}
```

**생성 파일**: `output_dir/debug/page_NNN.html`

**HTML 구조**:
- 페이지 이미지 위에 블록을 `<div>` 오버레이
- 3가지 레이어: `.parser` (파란색), `.ocr` (빨간색), `.fused` (녹색)
- 블록 클릭 시 `#info` 영역에 상세 정보 표시:
  - provenance
  - confidence
  - similarity
  - parser_text / ocr_text / final_text

---

## 데이터 흐름

### Phase 1: 독립 분석
```
PDF File
   │
   ├──▶ Parser Track
   │      └──▶ extract_glyph_runs()
   │           └──▶ build_layout()
   │                └──▶ PageHypothesis A (Provenance::Parser)
   │
   └──▶ OCR Track
          └──▶ render_page()
               └──▶ ocr_bridge.py
                    └──▶ build_layout()
                         └──▶ PageHypothesis B (Provenance::Ocr)
```

### Phase 2: 융합
```
PageHypothesis A + PageHypothesis B
   │
   ├──▶ align_blocks()
   │      └──▶ AlignmentResult { matched, unmatched_a, unmatched_b }
   │
   └──▶ resolve_blocks()
          │
          ├──▶ for matched pairs:
          │      └──▶ text_similarity()
          │           └──▶ score_confidence()
          │                └──▶ Block (Provenance::Fused or Parser)
          │
          ├──▶ for unmatched_a:
          │      └──▶ promote_single(Provenance::Parser)
          │
          └──▶ for unmatched_b:
                 └──▶ promote_single(Provenance::Ocr)
```

### Phase 3: 내보내기
```
DocumentFinal
   │
   ├──▶ JsonExporter
   │      └──▶ output_dir/document.json
   │
   └──▶ HtmlDebugExporter
          └──▶ output_dir/debug/page_001.html
               output_dir/debug/page_002.html
               ...
```

---

## 모듈 상세 설명

### Python PDFOCR 모듈 (`src/pdfocr/`)

별도의 Python 기반 OCR 파이프라인 (Rust 파이프라인과 독립적으로 사용 가능)

#### main.py
```python
def process_single_pdf(pdf_path, output_dir=None, image_dir=None, 
                       lang="kor", dpi=300, keep_images=False)
```

**3단계 파이프라인**:
1. **PDF → Image**: `convert_pdf_to_images()` (pdf2image 사용)
2. **Image → Text**: `extract_text_from_images()` (pytesseract 사용)
3. **Save**: `save_extracted_text()` (페이지별로 구분된 텍스트 파일 저장)

#### pdf_to_image.py
```python
def convert_pdf_to_images(pdf_path, output_dir="images", dpi=300) -> List[str]
```
- `pdf2image.convert_from_path()` 사용
- 각 페이지를 `{basename}_page_{NNN}.png`로 저장

#### image_to_text.py
```python
def extract_text_from_image(image_path, lang="kor") -> str
def extract_text_from_images(image_paths, lang="kor") -> TextDict
```
- `pytesseract.image_to_string()` 사용
- 언어 설정 가능 (기본: `kor`)

#### layout.py
```python
@dataclass
class Block:
    x: int; y: int; w: int; h: int

def detect_blocks(image_path, min_area=800, merge_kernel=(15,7)) -> List[Block]
```

**블록 감지 알고리즘**:
1. 그레이스케일 변환
2. 적응형 이진화 (Adaptive Threshold)
3. 모폴로지 팽창 (Dilation) → 인접 문자 병합
4. 외곽선 검출 (findContours)
5. 면적 필터링 (min_area 이상만)
6. 좌상단→우하단 순서로 정렬

#### block_ocr.py
```python
def ocr_blocks(image_path, blocks, lang="kor") -> List[Dict]
```
- 각 Block 영역을 ROI(Region of Interest)로 추출
- pytesseract로 개별 OCR 수행
- JSON 형식으로 결과 반환:
```json
{
  "index": 1,
  "bbox": {"x": 10, "y": 20, "w": 100, "h": 50},
  "type": "text",
  "lang": "kor",
  "text": "추출된 텍스트"
}
```

---

## 실행 흐름

### 커맨드라인 실행
```bash
cargo run -- input.pdf --out ./output --dpi 200
```

### 메인 플로우 (`main.rs` → `pipeline.rs`)

```rust
fn main() -> Result<()> {
    // 1. CLI 파싱
    let cli = Cli::parse();
    let config = PipelineConfig::new(cli.input, cli.out, cli.dpi);

    // 2. 문서 빌드
    let document = build_document(&config)?;
    
    // 3. 내보내기
    export_document(&document, &config.output)?;
    
    Ok(())
}
```

### 페이지별 처리 루프 (`pipeline.rs::build_document`)

```rust
for page_idx in 0..page_count {
    // Step 1: 페이지 렌더링
    let rendered = renderer.render_page(&config.input, page_idx)?;
    
    // Step 2: Parser 분석
    let parser_hypo = parser_track.analyze_page(&config.input, page_idx)?;
    
    // Step 3: OCR 분석
    let ocr_hypo = ocr_track.analyze_page(&rendered.path, page_idx)?;
    
    // Step 4: 융합
    let mut fused = fusion.fuse(&parser_hypo, &ocr_hypo)?;
    
    // Step 5: 디버그 정보 첨부
    attach_debug_info(&mut fused, &parser_hypo, &ocr_hypo);
    
    pages.push(fused);
}
```

### Fusion 상세 플로우 (`SimpleFusionEngine::fuse`)

```rust
fn fuse(&self, parser: &PageHypothesis, ocr: &PageHypothesis) -> Result<PageFinal> {
    // 1. 블록 정렬
    let aligned = align::align_blocks(&parser.blocks, &ocr.blocks);
    
    // 2. 블록 해결
    let resolved = resolve::resolve_blocks(&aligned);
    
    // 3. 페이지 분류
    let page_class = finalize::classify_page(parser, ocr);
    
    // 4. PageFinal 생성
    Ok(PageFinal {
        page_idx: parser.page_idx,
        class: page_class,
        blocks: resolved,
        width: parser.width.max(ocr.width),
        height: parser.height.max(ocr.height),
        debug: None,
    })
}
```

---

## 주요 설계 원칙

### 1. Trait 기반 추상화
```rust
pub trait ParserTrack {
    fn analyze_page(&self, pdf_path: &Path, page_idx: usize) -> Result<PageHypothesis>;
}

pub trait OcrTrack {
    fn analyze_page(&self, rendered_image: &Path, page_idx: usize) -> Result<PageHypothesis>;
}

pub trait FusionEngine {
    fn fuse(&self, parser: &PageHypothesis, ocr: &PageHypothesis) -> Result<PageFinal>;
}
```
- 각 Track과 Engine을 교체 가능하게 설계
- 테스트와 확장이 용이

### 2. Provenance 추적
모든 데이터에 출처 정보를 태깅:
```rust
pub enum Provenance {
    Parser,  // PDF 내부 텍스트
    Ocr,     // 광학 인식 텍스트
    Fused,   // 두 결과의 융합
}
```

### 3. 신뢰도 기반 선택
- 텍스트 유사도, 기하학적 정렬, 출처를 종합하여 0~1 점수 계산
- 낮은 신뢰도 블록은 수동 검토 대상으로 표시 가능

### 4. 디버그 가능성
- `PageDebug`: parser/ocr 원본 블록 보존
- `BlockDebug`: parser_text, ocr_text, final_text, similarity 저장
- HTML 뷰어로 시각적 검증

---

## 확장 가능성

### 현재 구현 상태
많은 부분이 **stub/placeholder** 상태:
- `PdfReader::page_count()`: 항상 1 반환
- `extract_glyph_runs()`: 더미 데이터 반환
- `PageRenderer::render_page()`: 빈 이미지 생성
- `ocr_bridge.py`: 빈 배열 반환

### 향후 개선 방향

#### 1. PDF 파싱 라이브러리 통합
- `pdfium`, `mupdf`, `poppler` 등의 라이브러리 사용
- 실제 glyph 좌표와 폰트 정보 추출

#### 2. 실제 렌더링 엔진
- `pdfium`, `cairo` 등으로 고품질 이미지 렌더링
- 벡터 그래픽 정확한 래스터화

#### 3. OCR 엔진 통합
Python 브리지에 실제 OCR 엔진 연결:
- **Tesseract**: 오픈소스, 다국어 지원
- **PaddleOCR**: 중국어/한국어 성능 우수
- **EasyOCR**: 80+ 언어 지원
- **Azure/Google Cloud Vision**: 클라우드 API

#### 4. 고급 레이아웃 분석
- 행/열 감지 (Line/Block grouping)
- 표 구조 인식
- 수식 영역 분리
- 그림/차트 감지

#### 5. 텍스트 정렬 개선
- 스펠링 체크
- 언어 모델 기반 수정 (GPT 등)
- 도메인별 용어집 적용

#### 6. 성능 최적화
- 멀티스레드 페이지 처리
- 이미지 캐싱
- 증분 처리 (변경된 페이지만)

---

## 테스트 전략

### 유닛 테스트
각 모듈별 테스트 존재:
- `geometry.rs`: IoU 계산 검증
- `align.rs`: 블록 매칭 로직
- `compare.rs`: 텍스트 유사도 계산
- `pipeline.rs`: 디버그 정보 첨부, 파일 생성 확인

### 통합 테스트
`tests/integration.rs`에 엔드투엔드 테스트 작성 가능

### 수동 테스트
- `test/` 디렉토리에 샘플 PDF 제공
- `test.fish` / `test-docker.fish` 스크립트로 실행

---

## 결론

DocStruct는 **Parser와 OCR의 크로스체킹**을 통해 PDF 문서의 구조를 정확하게 복원하는 시스템입니다.

**핵심 강점**:
1. **이중 검증**: 두 가지 독립적인 방법으로 텍스트 추출
2. **신뢰도 측정**: 각 블록의 품질을 정량화
3. **투명성**: Provenance 추적으로 데이터 출처 명확화
4. **확장성**: Trait 기반 설계로 컴포넌트 교체 용이
5. **디버깅**: HTML 뷰어로 시각적 검증

**현재 상태**: MVP(최소 기능 제품) 단계로, 핵심 아키텍처는 완성되었으나 실제 PDF 파싱 및 OCR 엔진 통합이 필요합니다.
