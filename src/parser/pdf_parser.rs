use anyhow::Result;
use std::path::PathBuf;

use crate::core::model::{Block, Line, PageHypothesis, Provenance, Span};
use crate::parser::pdf_reader::PdfReader;
use crate::parser::text_extractor::extract_glyph_runs;
use crate::parser::ParserTrack;

#[derive(Debug, Clone)]
pub struct PdfParser {
    path: PathBuf,
}

impl PdfParser {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl ParserTrack for PdfParser {
    fn page_count(&self) -> Result<usize> {
        PdfReader::new(self.path.clone())?.page_count()
    }

    fn analyze_page(&self, page_idx: usize) -> Result<PageHypothesis> {
        let glyph_runs = extract_glyph_runs(&self.path, page_idx);
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

    fn supports_ocr_rendering(&self) -> bool {
        true
    }

    fn rendering_source_path(&self) -> Option<&std::path::Path> {
        Some(&self.path)
    }
}
