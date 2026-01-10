use crate::core::model::PageClass;

#[derive(Debug, Clone, Copy)]
pub struct PageSignals {
    pub parser_glyphs: usize,
    pub image_coverage: f32,
    pub ocr_text_density: f32,
}

pub fn classify_page(signals: PageSignals) -> PageClass {
    let parser_score = signals.parser_glyphs as f32;
    let ocr_score = signals.ocr_text_density;

    if parser_score > 200.0 && signals.image_coverage < 0.3 {
        PageClass::Digital
    } else if parser_score < 50.0 && ocr_score > 0.5 {
        PageClass::Scanned
    } else {
        PageClass::Hybrid
    }
}
