use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::core::model::{DocumentFinal, PageDebug, PageFinal, PageHypothesis};
use crate::export::html_debug_export::HtmlDebugExporter;
use crate::export::json_export::JsonExporter;
use crate::export::markdown_export::MarkdownExporter;
use crate::export::text_export::TextExporter;
use crate::export::Exporter;
use crate::fusion::{FusionEngine, SimpleFusionEngine};
use crate::ocr::{
    bridge::OcrBridge, layout_builder::OcrLayoutBuilder, renderer::PageRenderer, OcrTrack,
};
use crate::parser::{layout_builder::ParserLayoutBuilder, pdf_reader::PdfReader, ParserTrack};

#[derive(Debug, Clone)]
pub struct PipelineConfig {
    pub input: PathBuf,
    pub output: PathBuf,
    pub dpi: u32,
}

impl PipelineConfig {
    pub fn new(input: PathBuf, output: PathBuf, dpi: u32) -> Self {
        Self { input, output, dpi }
    }
}

pub fn build_document(config: &PipelineConfig) -> Result<DocumentFinal> {
    let pdf_reader = PdfReader::new(config.input.clone())?;
    let page_count = pdf_reader.page_count()?;

    let renderer = PageRenderer::new(config.output.join("debug"), config.dpi);
    let parser_track = ParserLayoutBuilder::new();
    let bridge = OcrBridge::new(config.output.join("ocr"));
    let ocr_track = OcrLayoutBuilder::new(bridge);
    let fusion = SimpleFusionEngine::new();

    let mut pages: Vec<PageFinal> = Vec::with_capacity(page_count);

    for page_idx in 0..page_count {
        let rendered = renderer.render_page(&config.input, page_idx)?;
        let parser_hypo = parser_track.analyze_page(&config.input, page_idx)?;
        let ocr_hypo = ocr_track.analyze_page(&rendered.path, page_idx)?;
        let mut fused = fusion.fuse(&parser_hypo, &ocr_hypo)?;
        attach_debug_info(&mut fused, &parser_hypo, &ocr_hypo);
        pages.push(fused);
    }

    Ok(DocumentFinal { pages })
}

pub fn export_document(document: &DocumentFinal, output: &Path) -> Result<()> {
    let json_exporter = JsonExporter::new(output.to_path_buf());
    json_exporter.export(document)?;

    let html_exporter = HtmlDebugExporter::new(output.join("debug"));
    html_exporter.export(document)?;

    let text_exporter = TextExporter::new(output.to_path_buf());
    text_exporter.export(document)?;

    let markdown_exporter = MarkdownExporter::new(output.to_path_buf());
    markdown_exporter.export(document)?;

    Ok(())
}

fn attach_debug_info(fused: &mut PageFinal, parser: &PageHypothesis, ocr: &PageHypothesis) {
    fused.debug = Some(PageDebug {
        parser_blocks: parser.blocks.clone(),
        ocr_blocks: ocr.blocks.clone(),
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::core::geometry::BBox;
    use crate::core::model::{Block, Line, PageClass, PageHypothesis, Provenance, Span};

    fn temp_output_dir(prefix: &str) -> PathBuf {
        let mut out = std::env::temp_dir();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let pid = std::process::id();
        out.push(format!("{prefix}-{pid}-{now}"));
        out
    }

    fn text_block(text: &str, source: Provenance) -> Block {
        let bbox = BBox::new(0.0, 0.0, 10.0, 10.0);
        Block::TextBlock {
            bbox,
            lines: vec![Line {
                spans: vec![Span {
                    text: text.to_string(),
                    bbox,
                    source,
                    style: None,
                }],
            }],
            confidence: 0.5,
            source,
            debug: None,
        }
    }

    #[test]
    fn attaches_debug_blocks() {
        let parser = PageHypothesis {
            page_idx: 0,
            blocks: vec![text_block("parser", Provenance::Parser)],
            width: 100,
            height: 100,
        };
        let ocr = PageHypothesis {
            page_idx: 0,
            blocks: vec![text_block("ocr", Provenance::Ocr)],
            width: 100,
            height: 100,
        };
        let mut fused = PageFinal {
            page_idx: 0,
            class: PageClass::Hybrid,
            blocks: vec![],
            width: 100,
            height: 100,
            debug: None,
        };

        attach_debug_info(&mut fused, &parser, &ocr);

        let debug = fused.debug.expect("debug info should be set");
        assert_eq!(debug.parser_blocks.len(), 1);
        assert_eq!(debug.ocr_blocks.len(), 1);
    }

    #[test]
    fn export_document_writes_outputs() -> Result<()> {
        let output = temp_output_dir("docstruct-pipeline");
        fs::create_dir_all(&output)?;

        let document = DocumentFinal {
            pages: vec![PageFinal {
                page_idx: 0,
                class: PageClass::Digital,
                blocks: vec![],
                width: 100,
                height: 100,
                debug: None,
            }],
        };

        export_document(&document, &output)?;

        assert!(output.join("document.json").exists());
        assert!(output.join("debug/page_001.html").exists());

        let _ = fs::remove_dir_all(&output);
        Ok(())
    }
}
