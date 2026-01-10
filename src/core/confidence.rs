pub fn score_confidence(
    has_parser: bool,
    has_ocr: bool,
    similarity: Option<f32>,
    geometry_good: bool,
) -> f32 {
    let mut score = 0.0;
    if has_parser {
        score += 0.4;
    }
    if has_ocr {
        score += 0.3;
    }

    if let Some(sim) = similarity {
        if sim >= 0.9 {
            score += 0.3;
        } else if sim >= 0.7 {
            score += 0.15;
        } else {
            score -= 0.2;
        }
    }

    if geometry_good {
        score += 0.1;
    } else {
        score -= 0.1;
    }

    score.clamp(0.0, 1.0)
}
