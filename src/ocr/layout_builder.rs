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

    fn tokens_to_block(&self, tokens: Vec<OcrToken>) -> Option<Block> {
        if tokens.is_empty() {
            return None;
        }

        let mut spans = Vec::new();
        let mut bbox = BBox::new(tokens[0].bbox[0], tokens[0].bbox[1], tokens[0].bbox[2], tokens[0].bbox[3]);
        for token in tokens {
            let token_bbox = BBox::new(token.bbox[0], token.bbox[1], token.bbox[2], token.bbox[3]);
            bbox = bbox.union(&token_bbox);
            spans.push(Span {
                text: token.text,
                bbox: token_bbox,
                source: Provenance::Ocr,
                style: None,
            });
        }
        let line = Line { spans };
        Some(Block::TextBlock {
            bbox,
            lines: vec![line],
            confidence: 0.5,
            source: Provenance::Ocr,
            debug: None,
        })
    }
}

impl OcrTrack for OcrLayoutBuilder {
    fn analyze_page(&self, rendered_image: &Path, page_idx: usize) -> Result<PageHypothesis> {
        let tokens = self.bridge.run(rendered_image)?;
        let mut blocks = Vec::new();
        if let Some(block) = self.tokens_to_block(tokens) {
            blocks.push(block);
        }
        Ok(PageHypothesis {
            page_idx,
            blocks,
            width: 1000,
            height: 1400,
        })
    }
}
