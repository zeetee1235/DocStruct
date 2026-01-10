use crate::core::geometry::BBox;

#[derive(Debug, Clone)]
pub struct GlyphRun {
    pub text: String,
    pub bbox: BBox,
}

pub fn extract_glyph_runs(_pdf_path: &std::path::Path, _page_idx: usize) -> Vec<GlyphRun> {
    Vec::new()
}
