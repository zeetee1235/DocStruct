pub mod align;
pub mod compare;
pub mod finalize;
pub mod resolve;

use anyhow::Result;

use crate::core::model::{PageFinal, PageHypothesis};

pub trait FusionEngine {
    fn fuse(&self, parser: &PageHypothesis, ocr: &PageHypothesis) -> Result<PageFinal>;
}

#[derive(Debug, Default)]
pub struct SimpleFusionEngine;

impl SimpleFusionEngine {
    pub fn new() -> Self {
        Self
    }
}

impl FusionEngine for SimpleFusionEngine {
    fn fuse(&self, parser: &PageHypothesis, ocr: &PageHypothesis) -> Result<PageFinal> {
        let aligned = align::align_blocks(&parser.blocks, &ocr.blocks);
        let resolved = resolve::resolve_blocks(&aligned);
        let page_class = finalize::classify_page(parser, ocr);
        Ok(PageFinal {
            page_idx: parser.page_idx,
            class: page_class,
            blocks: resolved,
            width: parser.width.max(ocr.width),
            height: parser.height.max(ocr.height),
            debug: None,
        })
    }
}
