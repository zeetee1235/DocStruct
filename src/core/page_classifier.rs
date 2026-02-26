use crate::core::model::PageClass;

#[derive(Debug, Clone, Copy)]
pub struct PageSignals {
    pub parser_glyphs: usize,
    pub ocr_glyphs: usize,
    pub image_coverage: f32,
    pub ocr_text_density: f32,
}

pub fn classify_page(signals: PageSignals) -> PageClass {
    let parser_score = signals.parser_glyphs as f32;
    let ocr_score = signals.ocr_text_density;
    let coverage = signals.image_coverage;
    let ocr_glyphs = signals.ocr_glyphs;

    if signals.parser_glyphs >= 120 && signals.parser_glyphs >= ocr_glyphs.saturating_mul(2) {
        PageClass::Digital
    } else if ocr_glyphs >= signals.parser_glyphs.saturating_mul(2)
        && (ocr_score > 0.35 || coverage > 0.3)
    {
        PageClass::Scanned
    } else if parser_score > 220.0 && ocr_score < 0.25 && coverage < 0.25 {
        PageClass::Digital
    } else {
        PageClass::Hybrid
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_digital_page() {
        let signals = PageSignals {
            parser_glyphs: 420,
            ocr_glyphs: 100,
            image_coverage: 0.2,
            ocr_text_density: 0.18,
        };
        assert_eq!(classify_page(signals), PageClass::Digital);
    }

    #[test]
    fn classifies_scanned_page() {
        let signals = PageSignals {
            parser_glyphs: 20,
            ocr_glyphs: 300,
            image_coverage: 0.62,
            ocr_text_density: 0.56,
        };
        assert_eq!(classify_page(signals), PageClass::Scanned);
    }
}
