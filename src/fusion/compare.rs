use strsim::normalized_levenshtein;

pub fn text_similarity(a: &str, b: &str) -> f32 {
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }

    let base = normalized_levenshtein(a, b) as f32;
    let token_overlap = token_overlap(a, b);
    let mut score = (base + token_overlap) / 2.0;

    if numeric_mismatch(a, b) {
        score -= 0.1;
    }

    score.clamp(0.0, 1.0)
}

fn token_overlap(a: &str, b: &str) -> f32 {
    let a_tokens: std::collections::HashSet<_> = a.split_whitespace().collect();
    let b_tokens: std::collections::HashSet<_> = b.split_whitespace().collect();
    if a_tokens.is_empty() || b_tokens.is_empty() {
        return 0.0;
    }
    let intersection = a_tokens.intersection(&b_tokens).count() as f32;
    let union = a_tokens.union(&b_tokens).count() as f32;
    intersection / union
}

fn numeric_mismatch(a: &str, b: &str) -> bool {
    let digits_a: String = a.chars().filter(|c| c.is_ascii_digit()).collect();
    let digits_b: String = b.chars().filter(|c| c.is_ascii_digit()).collect();
    !digits_a.is_empty() && !digits_b.is_empty() && digits_a != digits_b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn similarity_penalizes_numeric_mismatch() {
        let score = text_similarity("2024 report", "2023 report");
        assert!(score < 0.9);
    }
}
