<p align="left">
  <img src="./docs/assets/docstruct_logo.png" alt="DocStruct logo" width="220" />
</p>

# DocStruct

![Last Commit](https://img.shields.io/github/last-commit/zeetee1235/DocStruct?style=for-the-badge)
![License: MIT](https://img.shields.io/badge/License-MIT-1f2937?style=for-the-badge)

DocStruct is currently a PDF-first document structure recovery tool that combines parser extraction, OCR extraction, and a fusion layer to produce reliable structured outputs.

Current scope: PDF only.  
Planned next formats: DOCX and PPT/PPTX.

Korean documentation: [docs/README.ko.md](./docs/README.ko.md)

## Overview

| Track | Role |
| --- | --- |
| Parser | Extract text and layout from PDF internals |
| OCR | Render pages and detect blocks/text from images |
| Fusion | Align parser/OCR outputs with confidence and provenance |

## Snapshot

```bash
./target/debug/docstruct convert tests/fixtures/test_document.pdf -o output_en --debug
```

<table>
  <tr>
    <th width="33%">PDF Page 1</th>
    <th width="33%">PDF Page 2</th>
    <th width="33%">PDF Page 3</th>
  </tr>
  <tr>
    <td><img src="./docs/assets/readme-input-page1.png" alt="Input PDF page 1" width="100%" /></td>
    <td><img src="./docs/assets/readme-input-page2.png" alt="Input PDF page 2" width="100%" /></td>
    <td><img src="./docs/assets/readme-input-page3.png" alt="Input PDF page 3" width="100%" /></td>
  </tr>
</table>

### Output Artifacts

- `document.json`
- `document.md`
- `document.txt`
- `page_XXX.md` / `page_XXX.txt`
- `figures/*.png`
- `debug/*.html`

## Pipeline

```mermaid
flowchart LR
    A[Input PDF] --> B[PDF Reader]

    B --> C[Parser Track]
    B --> D[OCR Track]

    C --> C1[pdftotext extraction]
    C1 --> C2[Normalization and quality checks]
    C2 --> C3[Parser hypothesis]

    D --> D1[Page render]
    D1 --> D2[Block detect and classify]
    D2 --> D3[Tesseract and post-processing]
    D3 --> D4[OCR hypothesis]

    C3 --> E[Fusion]
    D4 --> E

    E --> F[Resolved page model]
    F --> G1[JSON]
    F --> G2[Markdown]
    F --> G3[Text]
    F --> G4[Debug HTML]
```

Detailed design: [docs/ARCHITECTURE.md](./docs/ARCHITECTURE.md)

## Setup

Requirements:

- Rust toolchain
- Python 3.12+
- `poppler-utils` (`pdftotext`, `pdftoppm`, `pdfinfo`)
- `tesseract` with required language data

Nix Flakes:

```bash
nix develop
cargo build
```

Legacy nix-shell:

```bash
nix-shell
cargo build
```

Optional math OCR (pix2tex):

```bash
pip install --user 'pix2tex[gui]>=0.1.2'
```

## Usage

| Command | Purpose |
| --- | --- |
| `./target/debug/docstruct convert input.pdf -o output_dir --debug` | Convert one PDF |
| `./target/debug/docstruct batch file1.pdf file2.pdf -o output_dir --debug` | Convert multiple PDFs |
| `./target/debug/docstruct info input.pdf` | Inspect PDF metadata |

Useful flags:

- `--dpi <int>`: render DPI for OCR (default: 200)
- `--debug`: write debug assets
- `--quiet`: reduce console logs

## Output Layout

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

## Development

```bash
cargo build
cargo test
cargo test parser::hangul
```

Contributing guide: [CONTRIBUTING.md](./CONTRIBUTING.md)
