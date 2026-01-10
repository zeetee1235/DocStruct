use anyhow::Result;
use std::path::Path;

use crate::core::model::{Block, Line, PageHypothesis, Provenance, Span};
use crate::parser::text_extractor::extract_glyph_runs;
use crate::parser::ParserTrack;

#[derive(Debug, Default)]
pub struct ParserLayoutBuilder;

impl ParserLayoutBuilder {
    pub fn new() -> Self {
        Self
    }
}

impl ParserTrack for ParserLayoutBuilder {
    fn analyze_page(&self, pdf_path: &Path, page_idx: usize) -> Result<PageHypothesis> {
        let glyph_runs = extract_glyph_runs(pdf_path, page_idx);
        let mut blocks = Vec::new();

        if !glyph_runs.is_empty() {
            let mut spans = Vec::new();
            let mut bbox = glyph_runs[0].bbox;
            for run in glyph_runs {
                bbox = bbox.union(&run.bbox);
                spans.push(Span {
                    text: run.text,
                    bbox: run.bbox,
                    source: Provenance::Parser,
                    style: None,
                });
            }
            let line = Line { spans };
            blocks.push(Block::TextBlock {
                bbox,
                lines: vec![line],
                confidence: 0.6,
                source: Provenance::Parser,
                debug: None,
            });
        }

        Ok(PageHypothesis {
            page_idx,
            blocks,
            width: 1000,
            height: 1400,
        })
    }
}
