# DocStruct Architecture

## 1. Goals and Constraints

DocStruct reconstructs structured document outputs from PDF inputs with a dual-source strategy:

- Parser source: PDF-native text extraction (`pdftotext`)
- OCR source: rendered-page recognition (`tesseract` via Python bridge)

Primary goals:

- maximize text/block recovery on mixed-content PDFs
- retain provenance and confidence for downstream debugging
- support multilingual workflows, with Korean-specific normalization paths

Operational constraints:

- external runtime dependencies (`poppler-utils`, `tesseract`, Python)
- different error characteristics between parser and OCR tracks
- per-page processing with deterministic export formats

## 2. Top-Level Data Flow

1. `pipeline::build_document` opens input PDF and iterates pages.
2. Per page:
   - render page image (`ocr::renderer::PageRenderer`)
   - run parser track (`parser::layout_builder::ParserLayoutBuilder`)
   - run OCR track (`ocr::layout_builder::OcrLayoutBuilder`)
   - run fusion (`fusion::SimpleFusionEngine`)
3. Attach debug hypotheses (`parser_blocks`, `ocr_blocks`) to final page model.
4. Export final document into JSON/Markdown/Text/Debug HTML.

## 3. Module Breakdown

### 3.1 `src/core`

Responsibilities:

- geometry primitives (`BBox`): intersection/union/area/center distance
- confidence scoring (`confidence.rs`)
- domain model (`model.rs`):
  - `DocumentFinal`, `PageFinal`, `PageHypothesis`
  - `Block` variants (`TextBlock`, `TableBlock`, `FigureBlock`, `MathBlock`)
  - provenance enum (`Parser`, `Ocr`, `Fused`)
- page-level classification heuristics (`page_classifier.rs`)

Design note:

- model types are serialization-ready (`serde`) and used by all layers.

### 3.2 `src/parser`

Responsibilities:

- read PDF metadata/page count (`pdf_reader.rs`)
- extract parser text (`text_extractor.rs`)
- Korean normalization (`hangul.rs`): combine decomposed jamo into syllables
- parser hypothesis construction (`layout_builder.rs`)

Current behavior:

- parser text is coarse-grained at block level in many cases
- quality gates suppress severely degraded Korean parser outputs

Failure profile:

- parser can miss rendered-only text/figures
- parser may emit decomposed/noisy Unicode depending on PDF internals

### 3.3 `src/ocr` + `ocr/bridge`

Responsibilities:

- render PDF page images (`renderer.rs`)
- call Python OCR bridge (`bridge.rs`)
- map OCR tokens to Rust block model (`layout_builder.rs`)

Python bridge (`ocr/bridge/ocr_bridge.py`) pipeline:

1. image preprocessing and block detection (OpenCV morphology)
2. block OCR (`pytesseract`) with language config
3. block type classification (`text/table/figure/math`)
4. post-processing:
   - Hangul normalization
   - CJK/Hanja noise suppression
   - token deduplication
   - adjacent text-block merge
   - short Korean split-ending fixes
5. optional fallback full-page OCR if recall is too low

Failure profile:

- OCR can hallucinate symbols/characters under dense math or low contrast
- segmentation may fragment lines or over-merge neighboring content

### 3.4 `src/fusion`

Submodules:

- `align.rs`: geometric matching between parser and OCR blocks
- `compare.rs`: text similarity scoring
- `resolve.rs`: conflict resolution and filtering
- `finalize.rs`: page class decision (`digital/scanned/hybrid`)

Fusion process:

1. Align parser/OCR blocks by IoU/center distance.
2. Resolve matched pairs:
   - compare text similarity
   - choose parser/ocr/fused lines based on class and quality heuristics
3. Promote unmatched blocks with source-aware confidence.
4. Apply filters:
   - remove degraded parser Korean when OCR is clearly better
   - remove redundant OCR text under parser-dominant pages
   - remove low-quality OCR noise
   - suppress Korean OCR text when parser Korean is reliable (accuracy-first)

Design intent:

- parser is generally trusted for clean digital text
- OCR is used for coverage gaps and scanned content
- provenance is preserved for auditability

### 3.5 `src/export`

Exporters:

- `json_export.rs`
- `markdown_export.rs`
- `text_export.rs`
- `html_debug_export.rs`

Debug HTML includes per-block metadata:

- block type
- provenance
- confidence
- parser/ocr/final text and similarity (when available)

## 4. Runtime Entry Points

CLI entry points (`src/main.rs`):

- `convert`: one PDF
- `batch`: multiple PDFs
- `info`: PDF metadata only

Primary runtime path:

- `convert` -> `pipeline::build_document` -> `pipeline::export_document`

## 5. Key Heuristics

### 5.1 Page Classification

Inputs:

- parser glyph count
- OCR glyph count
- OCR density and coverage proxy

Output:

- `Digital`, `Scanned`, or `Hybrid`

This class controls fusion aggressiveness and source preference.

### 5.2 Korean Accuracy Controls

Implemented across parser/OCR/fusion:

- Hangul composition normalization
- decomposed-jamo degradation scoring
- Hanja/CJK removal in OCR normalization path
- strict OCR Korean suppression when parser Korean is reliable

Tradeoff:

- this can reduce OCR-only Korean recall in ambiguous regions
- but significantly improves character-level precision on noisy pages

### 5.3 Redundancy Control

When parser text is strong and broad:

- duplicated OCR snippets are removed by similarity + overlap checks

Goal:

- prevent repeated content in final exports
- keep non-overlapping OCR additions when they contain distinct content

## 6. Testing Strategy

Current tests cover:

- geometry and similarity primitives
- Hangul normalization behaviors
- fusion filtering/selection cases
- parser/OCR integration smoke tests with fixtures

Recommended additions:

- fixture-based regression tests for per-language precision/recall
- page-class regression checks (`digital/scanned/hybrid`)
- OCR bridge snapshot tests for post-processing outputs

## 7. Known Limitations

- Formula-heavy regions can still produce OCR symbol noise.
- Coverage metric is heuristic, not semantic segmentation.
- Parser track may collapse layout details for some PDFs.
- OCR fallback can still introduce short low-confidence fragments on edge cases.

## 8. Extension Points

Common extension paths:

- replace OCR bridge model stack while keeping JSON token contract
- add stronger table/math-specific structural modeling
- add confidence calibration per block type/source
- add benchmark harness for fixture-level metric tracking

## 9. File Map

```text
src/
  core/
  parser/
  ocr/
  fusion/
  export/
ocr/bridge/
  ocr_bridge.py
tests/
  fixtures/
docs/
  ARCHITECTURE.md
```
