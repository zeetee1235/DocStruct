pub mod layout_builder;
pub mod pdf_reader;
pub mod text_extractor;
pub mod hangul;

pub use pdf_reader::PdfReader;

use anyhow::Result;
use std::path::Path;

use crate::core::model::PageHypothesis;

pub trait ParserTrack {
    fn analyze_page(&self, pdf_path: &Path, page_idx: usize) -> Result<PageHypothesis>;
}
