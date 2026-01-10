pub mod bridge;
pub mod layout_builder;
pub mod renderer;

use anyhow::Result;
use std::path::Path;

use crate::core::model::PageHypothesis;

pub trait OcrTrack {
    fn analyze_page(&self, rendered_image: &Path, page_idx: usize) -> Result<PageHypothesis>;
}
