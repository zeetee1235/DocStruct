use crate::core::model::{PageClass, PageHypothesis};
use crate::core::page_classifier::{classify_page, PageSignals};

pub fn classify_page(parser: &PageHypothesis, ocr: &PageHypothesis) -> PageClass {
    let parser_glyphs = parser
        .blocks
        .iter()
        .filter_map(|block| block.text_content())
        .map(|text| text.len())
        .sum::<usize>();
    let ocr_glyphs = ocr
        .blocks
        .iter()
        .filter_map(|block| block.text_content())
        .map(|text| text.len())
        .sum::<usize>();
    let signals = PageSignals {
        parser_glyphs,
        image_coverage: if ocr_glyphs > 0 { 0.6 } else { 0.1 },
        ocr_text_density: (ocr_glyphs as f32 / 1000.0).min(1.0),
    };
    classify_page(signals)
}
