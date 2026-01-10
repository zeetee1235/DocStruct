use std::fs;
use std::path::PathBuf;

use anyhow::Result;

use crate::core::model::{Block, BlockDebug, DocumentFinal, Provenance};
use crate::export::Exporter;

#[derive(Debug, Clone)]
pub struct HtmlDebugExporter {
    out_dir: PathBuf,
}

impl HtmlDebugExporter {
    pub fn new(out_dir: PathBuf) -> Self {
        Self { out_dir }
    }

    fn block_to_div(block: &Block, layer: &str) -> String {
        let bbox = block.bbox();
        let text = block.text_content().unwrap_or_default();
        let provenance = block.provenance();
        let debug = extract_debug(block);
        format!(
            r#"<div class='bbox {layer}' style='left:{x0}px; top:{y0}px; width:{w}px; height:{h}px;' data-text='{text}' data-provenance='{prov}' data-confidence='{conf}' data-parser-text='{parser_text}' data-ocr-text='{ocr_text}' data-final-text='{final_text}' data-similarity='{similarity}'></div>"#,
            x0 = bbox.x0,
            y0 = bbox.y0,
            w = bbox.width(),
            h = bbox.height(),
            layer = layer,
            text = html_escape::encode_text(&text),
            prov = provenance_label(provenance),
            conf = block.confidence(),
            parser_text = html_escape::encode_text(debug.parser_text.as_deref().unwrap_or("")),
            ocr_text = html_escape::encode_text(debug.ocr_text.as_deref().unwrap_or("")),
            final_text = html_escape::encode_text(debug.final_text.as_deref().unwrap_or("")),
            similarity = debug
                .similarity
                .map(|value| format!("{value:.3}"))
                .unwrap_or_default(),
        )
    }
}

fn extract_debug(block: &Block) -> BlockDebug {
    match block {
        Block::TextBlock { debug, .. }
        | Block::TableBlock { debug, .. }
        | Block::FigureBlock { debug, .. }
        | Block::MathBlock { debug, .. } => debug.clone().unwrap_or(BlockDebug {
            parser_text: None,
            ocr_text: None,
            final_text: None,
            similarity: None,
        }),
    }
}

fn provenance_label(prov: Provenance) -> &'static str {
    match prov {
        Provenance::Parser => "parser",
        Provenance::Ocr => "ocr",
        Provenance::Fused => "fused",
    }
}

impl Exporter for HtmlDebugExporter {
    fn export(&self, document: &DocumentFinal) -> Result<()> {
        fs::create_dir_all(&self.out_dir)?;
        for page in &document.pages {
            let image_path = format!("page_{:03}.png", page.page_idx + 1);
            let mut blocks_html = String::new();
            if let Some(debug) = &page.debug {
                for block in &debug.parser_blocks {
                    blocks_html.push_str(&HtmlDebugExporter::block_to_div(block, "parser"));
                }
                for block in &debug.ocr_blocks {
                    blocks_html.push_str(&HtmlDebugExporter::block_to_div(block, "ocr"));
                }
            }
            for block in &page.blocks {
                blocks_html.push_str(&HtmlDebugExporter::block_to_div(block, "fused"));
            }

            let html = format!(
                r#"<!DOCTYPE html>
<html>
<head>
<meta charset='utf-8'>
<title>DocStruct Debug Page {page_idx}</title>
<style>
body {{ margin: 0; font-family: Arial, sans-serif; }}
#canvas {{ position: relative; }}
#canvas img {{ display: block; }}
.bbox {{ position: absolute; border: 2px solid rgba(0,0,255,0.4); box-sizing: border-box; }}
.bbox.parser {{ border-color: rgba(0,0,255,0.6); }}
.bbox.ocr {{ border-color: rgba(255,0,0,0.6); }}
.bbox.fused {{ border-color: rgba(0,128,0,0.6); }}
#info {{ position: fixed; right: 10px; top: 10px; background: #fff; padding: 10px; border: 1px solid #ddd; max-width: 300px; }}
</style>
</head>
<body>
<div id='info'>Click a block to inspect.</div>
<div id='canvas'>
<img src='{image}' />
{blocks}
</div>
<script>
const info = document.getElementById('info');
for (const el of document.querySelectorAll('.bbox')) {{
  el.addEventListener('click', () => {{
    info.innerHTML = `provenance: ${{el.dataset.provenance}}<br/>confidence: ${{el.dataset.confidence}}<br/>similarity: ${{el.dataset.similarity}}<br/>parser_text: ${{el.dataset.parserText}}<br/>ocr_text: ${{el.dataset.ocrText}}<br/>final_text: ${{el.dataset.finalText}}`;
  }});
}}
</script>
</body>
</html>"#,
                page_idx = page.page_idx + 1,
                image = image_path,
                blocks = blocks_html
            );
            let path = self.out_dir.join(format!("page_{:03}.html", page.page_idx + 1));
            fs::write(path, html)?;
        }
        Ok(())
    }
}
