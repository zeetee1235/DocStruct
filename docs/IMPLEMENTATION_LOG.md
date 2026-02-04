# DocStruct 구현 기록

## 2026년 2월 4일 - 블록 타입 분류 및 구조화 출력 구현

### 📌 개요
PDF 문서의 구조를 최대한 보존하면서 출력하기 위한 블록 타입 분류 시스템과 다양한 출력 포맷(TXT, Markdown) 구현.

---

## 🎯 구현 내역

### 1. 블록 타입 분류 시스템

#### 1.1 Block Enum 확장
**파일**: `src/core/model.rs`

기존 단순 TextBlock에서 4가지 타입으로 확장:

```rust
pub enum Block {
    TextBlock { bbox, lines, confidence, source, debug },
    TableBlock { bbox, confidence, source, debug },
    FigureBlock { bbox, confidence, source, debug },
    MathBlock { bbox, confidence, source, latex: Option<String>, debug },
}
```

#### 1.2 OCR 브리지 블록 분류 로직
**파일**: `ocr/bridge/ocr_bridge.py`

**분류 알고리즘**:

1. **Math Block 감지**
   - 수학 기호 패턴 매칭: `[∫∑∏∂∇±≤≥≠∞⊂⊃∪∩...]`
   - 함수명 감지: `sin, cos, tan, exp, log, lim`
   - 그리스 문자: `π, λ, μ, σ, α-ω`
   - 미분 기호: `dx, dt, dy`
   - 조건: 패턴 2개 이상 OR 기호 밀도 > 20% OR (패턴 1개 + 5000 < 면적 < 100000 + 기호 3개 이상)

2. **Figure Block 감지** (우선순위 높음)
   - Edge density 기반: Canny edge detection 사용
   - 조건: 면적 > 50,000px² AND edge_density > 0.08
   - 복잡한 그래픽(TikZ, 차트, 다이어그램) 감지
   - Table과 구분: edge density가 높으면 figure

3. **Table Block 감지**
   - Morphological operations로 수평/수직 선 검출
   - 조건: h_density > 0.01 AND v_density > 0.01 AND 면적 > 10,000px² AND edge_density < 0.05
   - 명확한 그리드 구조 필요

4. **Text Block** (기본값)
   - 위 조건에 해당하지 않으면 모두 텍스트로 분류

**파라미터 최적화**:
```python
def detect_blocks(image_path: Path, 
                 min_area: int = 2000,      # 최소 블록 면적
                 merge_kernel: tuple = (15, 10)):  # 병합 커널 크기
```

- 초기값 (800, (15,7)) → 최적화 (2000, (15,10))
- Iterations: 2 → 1 (수식과 텍스트 분리 유지)

### 2. 수식 OCR (LaTeX 추출)

#### 2.1 pix2tex 통합
**라이브러리**: `pix2tex>=0.1.2`

**구현**:
```python
def get_latex_model():
    """Lazy load LaTeX OCR model."""
    global _latex_model
    if _latex_model is None:
        from pix2tex.cli import LatexOCR
        _latex_model = LatexOCR()
    return _latex_model
```

- Lazy loading으로 시작 시간 최적화
- Math 블록에만 LaTeX OCR 실행
- 실패 시 graceful fallback (빈 문자열)

#### 2.2 Rust 모델 업데이트
**파일**: `src/core/model.rs`, `src/ocr/bridge.rs`

```rust
pub struct OcrToken {
    pub text: String,
    pub bbox: [f32; 4],
    pub block_type: String,
    pub latex: Option<String>,  // 추가
}

pub enum Block {
    MathBlock {
        bbox: BBox,
        confidence: f32,
        source: Provenance,
        latex: Option<String>,  // 추가
        debug: Option<BlockDebug>,
    },
}
```

### 3. 출력 포맷 구현

#### 3.1 텍스트 출력 (.txt)
**파일**: `src/export/text_export.rs`

**구조**:
```
=== Page 1 ===

[텍스트 내용]

[TABLE at x:617 y:334 w:177 h:47]

[FIGURE at x:454 y:334 w:153 h:47]

[MATH at x:... y:... w:... h:...]
```

- 텍스트: 원본 그대로 출력
- 비텍스트 블록: 위치와 크기 정보로 표시

#### 3.2 마크다운 출력 (.md)
**파일**: `src/export/markdown_export.rs`

**기능**:
1. **이미지 크롭 및 저장**
   ```rust
   fn crop_block_image(&self, page_image_path, bbox, page_idx, block_idx, block_type)
       -> Result<String>
   ```
   - 각 블록을 페이지 이미지에서 크롭
   - `figures/page_XXX_TYPE__NN.png` 형식으로 저장
   - 상대 경로 반환

2. **블록별 포맷팅**
   - **TextBlock**: 원본 텍스트
   - **TableBlock**: `![Table](figures/page_001_table__02.png)`
   - **FigureBlock**: `![Figure](figures/page_001_figure__05.png)`
   - **MathBlock**: 
     - LaTeX 있으면: `$$\n{latex}\n$$`
     - 없으면: `![Math](figures/page_001_math__12.png)`

3. **출력 파일**
   - `document.md`: 전체 문서
   - `page_NNN.md`: 페이지별
   - `figures/`: 추출된 이미지들

#### 3.3 HTML 디버그 뷰어 개선
**파일**: `src/export/html_debug_export.rs`

**추가 기능**:
- 블록 타입별 색상 구분:
  - Text: 연한 파랑 (`rgba(100,100,255,0.1)`)
  - Table: 주황색 + 점선 테두리 (`rgba(255,165,0,0.15)`)
  - Figure: 보라색 (`rgba(128,0,128,0.1)`)
  - Math: 청록색 (`rgba(0,200,200,0.15)`)
- 범례(Legend) 추가
- `data-type` 속성으로 블록 타입 표시

### 4. 파이프라인 통합

#### 4.1 Export 순서
**파일**: `src/pipeline.rs`

```rust
pub fn export_document(document: &DocumentFinal, output: &Path) -> Result<()> {
    // 1. JSON (구조화된 데이터)
    let json_exporter = JsonExporter::new(output.to_path_buf());
    json_exporter.export(document)?;
    
    // 2. HTML Debug (시각화)
    let html_exporter = HtmlDebugExporter::new(output.join("debug"));
    html_exporter.export(document)?;
    
    // 3. Text (단순 텍스트)
    let text_exporter = TextExporter::new(output.to_path_buf());
    text_exporter.export(document)?;
    
    // 4. Markdown (구조 보존 + 이미지)
    let markdown_exporter = MarkdownExporter::new(output.to_path_buf());
    markdown_exporter.export(document)?;
    
    Ok(())
}
```

---

## 📊 성능 및 결과

### 테스트 문서 (test_document.pdf)
- **원본**: LaTeX로 작성 (1 table, 1 TikZ figure, 여러 수식)
- **페이지**: 3페이지
- **DPI**: 200

### 검출 결과

#### Before 최적화:
- Total blocks: 86개
- Text: 28개
- Figure: 42개 (대부분 텍스트 오분류)
- Table: 14개
- Math: 2개

#### After 최적화:
- **Total blocks**: 153개
- **TextBlock**: 151개 (✅ +438%)
- **FigureBlock**: 1개 (✅ -98%, TikZ 그래프만 정확히 감지)
- **TableBlock**: 0개 (표가 텍스트로 병합됨)
- **MathBlock**: 1개 (✅ LaTeX 추출 성공)

### 출력 파일
```
test_rust_output/
├── document.json      (257KB) - 구조화된 데이터
├── document.md        (5.5KB) - 마크다운
├── document.txt       (5.4KB) - 플레인 텍스트
├── page_001.md        (2.0KB)
├── page_002.md        (1.7KB)
├── page_003.md        (1.7KB)
├── figures/
│   └── page_002_figure__34.png (TikZ 그래프)
└── debug/
    ├── page_001.html
    ├── page_002.html
    └── page_003.html
```

---

## 🔧 주요 기술 결정

### 1. 블록 분류 우선순위
**Math → Figure → Table → Text**

**이유**:
- Math: 특수 기호가 있어도 텍스트로 오인되기 쉬움 → 최우선 검사
- Figure: edge density가 높아 table로 오인될 수 있음 → table보다 먼저
- Table: 명확한 그리드 구조만 table로 분류
- Text: 나머지 모두 (기본값)

### 2. 파라미터 튜닝 접근
**원본 TEX 구조와 비교하며 반복 최적화**

- 초기: 너무 많은 작은 블록 생성 (105개 이미지)
- 조정: min_area 증가, merge_kernel 축소
- 결과: 실제 그래픽 1개만 정확히 추출

### 3. LaTeX OCR 통합 방식
**Lazy loading + Optional fallback**

**장점**:
- 모델 로딩이 느려도 시작 시간에 영향 없음
- Math 블록이 없으면 모델 로딩 안함
- LaTeX 변환 실패해도 전체 파이프라인 중단 안됨

**한계**:
- pix2tex 정확도 한계 (복잡한 수식은 부정확)
- 향후 다른 모델로 교체 가능하도록 인터페이스 설계

### 4. 마크다운 출력 설계
**이미지 크롭 방식 선택**

**대안**:
1. OCR 텍스트만 사용 → 정보 손실
2. 전체 페이지 이미지 삽입 → 파일 크기 큰
3. **블록별 크롭 (채택)** → 정확성 + 파일 크기 최적

**이점**:
- 표/그림을 원본 그대로 보존
- 마크다운 뷰어에서 즉시 확인 가능
- 텍스트 검색 + 시각적 정확성 양립

---

## 🚀 향후 개선 방향

### 1. 블록 분류 정확도
- [ ] 표 검출 개선 (현재 텍스트로 병합됨)
- [ ] 수식 영역 분리 (현재 큰 블록에 포함됨)
- [ ] 다단 레이아웃 지원
- [ ] 제목/본문 구분

### 2. LaTeX OCR 품질
- [ ] 다른 모델 평가 (Mathpix, Nougat 등)
- [ ] 앙상블 방식 고려
- [ ] 후처리 규칙 추가

### 3. 출력 포맷
- [ ] DOCX 출력
- [ ] HTML 출력 (스타일 포함)
- [ ] PDF 재생성

### 4. 최적화
- [ ] 블록 병합 알고리즘 개선
- [ ] 읽기 순서 최적화
- [ ] 병렬 처리 (페이지별)

---

## 📝 관련 파일

### 핵심 구현
- `src/core/model.rs` - Block enum 정의
- `ocr/bridge/ocr_bridge.py` - 블록 분류 로직
- `src/ocr/layout_builder.rs` - OCR 결과를 Block으로 변환
- `src/export/markdown_export.rs` - 마크다운 출력
- `src/export/text_export.rs` - 텍스트 출력
- `src/export/html_debug_export.rs` - HTML 디버그 뷰어

### 의존성
- `requirements.txt` - pix2tex 추가
- `Cargo.toml` - serde, image 등

### 테스트
- `test/test_document.tex` - 테스트 문서 원본
- `test/test_document.pdf` - 생성된 PDF
- `tests/integration.rs` - 통합 테스트

---

## ✅ 검증

### 단위 테스트
```bash
cargo test
# 5 passed (core)
# 3 passed (integration)
```

### 통합 테스트
```bash
target/release/docstruct test/test_document.pdf --out test_rust_output --dpi 200
```

**결과 검증**:
- ✅ JSON 생성: 257KB, 153 blocks
- ✅ Markdown 생성: 텍스트 + 1개 figure 이미지
- ✅ HTML 디버그 뷰어: 블록 타입별 색상 구분
- ✅ TikZ 그래프 정확히 추출
- ✅ Math 블록 LaTeX 변환 (부분 성공)

---

## 📚 참고

### 블록 검출 알고리즘
- OpenCV morphological operations
- Canny edge detection
- Contour analysis

### OCR
- Tesseract (일반 텍스트)
- pix2tex (수식 → LaTeX)

### 출력 포맷
- JSON (구조화 데이터)
- Markdown (문서 + 이미지)
- TXT (플레인 텍스트)
- HTML (디버그 시각화)
