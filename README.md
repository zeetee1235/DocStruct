# DocStruct

Parser ↔ OCR 크로스체킹 기반 문서 구조 복원 시스템 (MVP).

## 목표

- 입력: PDF
- 출력: `DocumentFinal` (페이지별 블록 + 텍스트/표/수식/그림 + confidence + provenance)
- provenance: `parser | ocr | fused`
- confidence: 0~1
- 좌표계: page pixel 좌표 (렌더 DPI 기준)

## 아키텍처

- Parser Track과 OCR Track이 각각 독립적으로 Layout Hypothesis를 생성
- Fusion Engine이 정렬/비교/충돌 해결/신뢰도 스코어링으로 최종 구조 생성

## 폴더 구조

```
core/
  geometry/        # BBox, IoU, 좌표 변환
  model/           # Document/Page/Block/Line/Span
  confidence/      # scoring
parser/
  pdf_reader/
  text_extractor/  # glyph/run + bbox
  layout_builder/  # run → line → block (hypothesis A)
ocr/
  renderer/        # page → image
  bridge/          # python 호출/IPC
  layout_builder/  # OCR tokens → block (hypothesis B)
fusion/
  align/
  compare/
  resolve/
  finalize/
export/
  json_export/
  html_debug_export/
```

## 실행 방법 (MVP)

```bash
cargo run -- <input.pdf> --out <dir> --dpi 200
```

산출물:

- `<dir>/document.json`
- `<dir>/debug/page_001.html` + 페이지 이미지 + 오버레이

## Document JSON 스키마 예시

```json
{
  "pages": [
    {
      "page_idx": 0,
      "class": "digital",
      "width": 1000,
      "height": 1400,
      "blocks": [
        {
          "type": "TextBlock",
          "bbox": { "x0": 10.0, "y0": 20.0, "x1": 400.0, "y1": 80.0 },
          "lines": [
            {
              "spans": [
                {
                  "text": "Hello world",
                  "bbox": { "x0": 10.0, "y0": 20.0, "x1": 200.0, "y1": 40.0 },
                  "source": "parser",
                  "style": null
                }
              ]
            }
          ],
          "confidence": 0.85,
          "source": "fused"
        }
      ]
    }
  ]
}
```

## Python OCR 브리지

- 스크립트: `ocr/bridge/ocr_bridge.py`
- 입력: 이미지 경로
- 출력: `[{"text": "...", "bbox": [x0,y0,x1,y1]}]` JSON
- OCR 엔진은 플러그 가능하도록 설계 (tesseract/paddleocr/easyocr 등 교체 가능)

## 디버그 뷰어

- HTML 페이지에서 parser/ocr/fused 블록을 각각 색상 레이어로 표시
- 블록 클릭 시 parser_text / ocr_text / final_text / confidence / similarity 노출
