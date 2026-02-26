use crate::core::model::{PageClass, PageHypothesis};
use crate::core::page_classifier::{classify_page as classify_page_internal, PageSignals};

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
    let page_area = (parser.width.max(ocr.width) as f32) * (parser.height.max(ocr.height) as f32);
    let ocr_coverage = if page_area <= 0.0 {
        0.0
    } else {
        let max_block_area = ocr
            .blocks
            .iter()
            .map(|block| block.bbox().area())
            .fold(0.0_f32, f32::max);
        (max_block_area / page_area).clamp(0.0, 1.0)
    };

    let signals = PageSignals {
        parser_glyphs,
        ocr_glyphs,
        image_coverage: ocr_coverage,
        ocr_text_density: (ocr_glyphs as f32 / 1000.0).min(1.0),
    };
    classify_page_internal(signals)
}
