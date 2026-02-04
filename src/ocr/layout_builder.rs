use anyhow::Result;
use std::path::Path;

use crate::core::geometry::BBox;
use crate::core::model::{Block, Line, PageHypothesis, Provenance, Span};
use crate::ocr::bridge::{OcrBridge, OcrToken};
use crate::ocr::OcrTrack;

#[derive(Debug, Clone)]
pub struct OcrLayoutBuilder {
    bridge: OcrBridge,
}

impl OcrLayoutBuilder {
    pub fn new(bridge: OcrBridge) -> Self {
        Self { bridge }
    }

    fn token_to_block(&self, token: OcrToken) -> Block {
        let bbox = BBox::new(token.bbox[0], token.bbox[1], token.bbox[2], token.bbox[3]);
        let confidence = 0.5;
        let source = Provenance::Ocr;
        
        match token.block_type.as_str() {
            "table" => Block::TableBlock {
                bbox,
                confidence,
                source,
                debug: None,
            },
            "figure" => Block::FigureBlock {
                bbox,
                confidence,
                source,
                debug: None,
            },
            "math" => Block::MathBlock {
                bbox,
                confidence,
                source,
                latex: token.latex.filter(|s| !s.is_empty()),
                debug: None,
            },
            _ => {
                // Text block with spans
                let span = Span {
                    text: token.text,
                    bbox,
                    source: Provenance::Ocr,
                    style: None,
                };
                let line = Line { spans: vec![span] };
                Block::TextBlock {
                    bbox,
                    lines: vec![line],
                    confidence,
                    source,
                    debug: None,
                }
            }
        }
    }
}

impl OcrTrack for OcrLayoutBuilder {
    fn analyze_page(&self, rendered_image: &Path, page_idx: usize) -> Result<PageHypothesis> {
        let tokens = self.bridge.run(rendered_image)?;
        let blocks: Vec<Block> = tokens.into_iter()
            .map(|token| self.token_to_block(token))
            .collect();
        
        Ok(PageHypothesis {
            page_idx,
            blocks,
            width: 1000,
            height: 1400,
        })
    }
}
