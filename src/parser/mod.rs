pub mod docx_parser;
pub mod hangul;
pub mod layout_builder;
pub mod pdf_parser;
pub mod pdf_reader;
pub mod pptx_parser;
pub mod text_extractor;

pub use layout_builder::ParserLayoutBuilder;

use anyhow::Result;
use std::path::Path;

use crate::core::model::PageHypothesis;

pub trait ParserTrack {
    fn page_count(&self) -> Result<usize>;
    fn analyze_page(&self, page_idx: usize) -> Result<PageHypothesis>;
    fn supports_ocr_rendering(&self) -> bool;
    fn rendering_source_path(&self) -> Option<&Path>;
}
