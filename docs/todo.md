#### PdfReader (`pdf_reader.rs`)
```rust
pub fn page_count(&self) -> usize
```
- PDF 파일을 열고 페이지 수를 반환
- *현재는 stub 구현으로 항상 1 반환 (추후 실제 PDF 라이브러리 통합 필요)*

#### TextExtractor (`text_extractor.rs`)
```rust
pub fn extract_glyph_runs(pdf_path: &Path, page_idx: usize) -> Vec<GlyphRun>
```
- PDF에서 glyph(문자 단위) 데이터와 bounding box 추출
- *현재는 stub으로 더미 데이터 반환*

### OCR Track (`src/ocr/`)

**역할**: 렌더링된 페이지 이미지에서 텍스트를 광학적으로 인식

#### 3.1 PageRenderer (`renderer.rs`)
```rust
pub fn render_page(&self, pdf_path: &Path, page_idx: usize) -> Result<RenderedPage>
```
- PDF 페이지를 PNG 이미지로 변환
- DPI 설정에 따라 해상도 조정
- *현재는 빈 흰색 이미지 생성 (추후 실제 렌더링 엔진 통합 필요)*

**Python 브리지 스크립트** (`ocr/bridge/ocr_bridge.py`):
- 현재는 빈 배열 반환 (placeholder)
- 추후 tesseract/paddleocr/easyocr 등 실제 OCR 엔진 통합 예정