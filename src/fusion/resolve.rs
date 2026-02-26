use crate::core::confidence::score_confidence;
use crate::core::model::{Block, BlockDebug, Line, PageClass, Provenance};
use crate::fusion::align::{AlignmentResult, MatchedPair};
use crate::fusion::compare::text_similarity;

pub fn resolve_blocks(alignment: &AlignmentResult, page_class: PageClass) -> Vec<Block> {
    let mut blocks = Vec::new();

    for pair in &alignment.matched {
        blocks.push(resolve_pair(pair, page_class));
    }

    for block in &alignment.unmatched_a {
        blocks.push(promote_single(
            block.clone(),
            Provenance::Parser,
            page_class,
        ));
    }

    for block in &alignment.unmatched_b {
        blocks.push(promote_single(block.clone(), Provenance::Ocr, page_class));
    }

    let blocks = filter_degraded_parser_blocks(blocks);
    match page_class {
        PageClass::Digital => {
            let blocks = filter_redundant_ocr_text_blocks(blocks);
            let blocks = filter_low_quality_ocr_text_blocks(blocks, true);
            let blocks = filter_korean_ocr_when_parser_reliable(blocks, true);
            filter_ocr_text_when_parser_reliable(blocks, page_class)
        }
        PageClass::Hybrid => {
            let blocks = filter_redundant_ocr_text_blocks(blocks);
            let blocks = filter_low_quality_ocr_text_blocks(blocks, true);
            let blocks = filter_korean_ocr_when_parser_reliable(blocks, false);
            filter_ocr_text_when_parser_reliable(blocks, page_class)
        }
        PageClass::Scanned => filter_low_quality_ocr_text_blocks(blocks, false),
    }
}

fn promote_single(block: Block, provenance: Provenance, page_class: PageClass) -> Block {
    match block {
        Block::TextBlock { bbox, lines, .. } => {
            let mut confidence = score_confidence(
                provenance == Provenance::Parser,
                provenance == Provenance::Ocr,
                None,
                true,
            );
            let final_text = text_from_lines(&lines);
            if provenance == Provenance::Parser
                && final_text
                    .as_deref()
                    .map(is_korean_parser_degraded)
                    .unwrap_or(false)
            {
                confidence = (confidence - 0.2).clamp(0.0, 1.0);
            }
            if page_class == PageClass::Scanned {
                if provenance == Provenance::Ocr {
                    confidence = (confidence + 0.08).clamp(0.0, 1.0);
                } else if provenance == Provenance::Parser {
                    confidence = (confidence - 0.1).clamp(0.0, 1.0);
                }
            }
            Block::TextBlock {
                bbox,
                lines,
                confidence,
                source: provenance,
                debug: Some(BlockDebug {
                    parser_text: if provenance == Provenance::Parser {
                        final_text.clone()
                    } else {
                        None
                    },
                    ocr_text: if provenance == Provenance::Ocr {
                        final_text.clone()
                    } else {
                        None
                    },
                    final_text,
                    similarity: None,
                }),
            }
        }
        Block::TableBlock { bbox, .. } => Block::TableBlock {
            bbox,
            confidence: score_confidence(
                provenance == Provenance::Parser,
                provenance == Provenance::Ocr,
                None,
                true,
            ),
            source: provenance,
            debug: None,
        },
        Block::FigureBlock { bbox, .. } => Block::FigureBlock {
            bbox,
            confidence: score_confidence(
                provenance == Provenance::Parser,
                provenance == Provenance::Ocr,
                None,
                true,
            ),
            source: provenance,
            debug: None,
        },
        Block::MathBlock { bbox, latex, .. } => Block::MathBlock {
            bbox,
            confidence: score_confidence(
                provenance == Provenance::Parser,
                provenance == Provenance::Ocr,
                None,
                true,
            ),
            source: provenance,
            latex: latex.clone(),
            debug: None,
        },
    }
}

fn resolve_pair(pair: &MatchedPair, page_class: PageClass) -> Block {
    let geometry_good = pair.iou > 0.3 || pair.center_distance < 50.0;
    let a_text = pair.a.text_content();
    let b_text = pair.b.text_content();

    let similarity = match (&a_text, &b_text) {
        (Some(a), Some(b)) => Some(text_similarity(a, b)),
        _ => None,
    };

    let mut confidence = score_confidence(true, true, similarity, geometry_good);

    match (&pair.a, &pair.b) {
        (
            Block::TextBlock {
                bbox,
                lines: parser_lines,
                ..
            },
            Block::TextBlock {
                lines: ocr_lines, ..
            },
        ) => {
            let parser_text = a_text.as_deref().unwrap_or_default();
            let ocr_text = b_text.as_deref().unwrap_or_default();
            let parser_quality = korean_text_quality(parser_text);
            let ocr_quality = korean_text_quality(ocr_text);
            let korean_present = has_korean_chars(parser_text) || has_korean_chars(ocr_text);
            let sim = similarity.unwrap_or(0.0);
            let parser_len = parser_text.chars().count();
            let ocr_len = ocr_text.chars().count();

            let (final_lines, provenance) = if sim >= 0.72 {
                if page_class == PageClass::Scanned {
                    (ocr_lines.clone(), Provenance::Fused)
                } else {
                    (parser_lines.clone(), Provenance::Fused)
                }
            } else {
                match page_class {
                    PageClass::Digital => {
                        if korean_present
                            && sim < 0.30
                            && ocr_quality > parser_quality + 5
                            && ocr_len > parser_len + 40
                        {
                            (ocr_lines.clone(), Provenance::Ocr)
                        } else {
                            (parser_lines.clone(), Provenance::Parser)
                        }
                    }
                    PageClass::Hybrid => {
                        if (korean_present
                            && sim < 0.30
                            && ocr_quality > parser_quality + 4
                            && ocr_len > parser_len + 50)
                            || (sim < 0.35
                                && ocr_len > parser_len + 80
                                && !is_noisy_ocr_text(ocr_text))
                        {
                            (ocr_lines.clone(), Provenance::Ocr)
                        } else {
                            (parser_lines.clone(), Provenance::Parser)
                        }
                    }
                    PageClass::Scanned => {
                        if !ocr_text.trim().is_empty() && !is_noisy_ocr_text(ocr_text) {
                            (ocr_lines.clone(), Provenance::Ocr)
                        } else {
                            (parser_lines.clone(), Provenance::Parser)
                        }
                    }
                }
            };

            if korean_present && provenance == Provenance::Parser && parser_quality < -2 {
                confidence = (confidence - 0.2).clamp(0.0, 1.0);
            }
            if korean_present && provenance == Provenance::Ocr && ocr_quality > 0 {
                confidence = (confidence + 0.05).clamp(0.0, 1.0);
            }
            if page_class == PageClass::Scanned && provenance == Provenance::Ocr {
                confidence = (confidence + 0.08).clamp(0.0, 1.0);
            }

            let final_text = text_from_lines(&final_lines);
            Block::TextBlock {
                bbox: *bbox,
                lines: final_lines,
                confidence,
                source: provenance,
                debug: Some(BlockDebug {
                    parser_text: a_text.clone(),
                    ocr_text: b_text.clone(),
                    final_text,
                    similarity,
                }),
            }
        }
        _ => Block::FigureBlock {
            bbox: pair.a.bbox().union(&pair.b.bbox()),
            confidence,
            source: Provenance::Fused,
            debug: None,
        },
    }
}

fn text_from_lines(lines: &[Line]) -> Option<String> {
    let text = lines
        .iter()
        .flat_map(|line| line.spans.iter())
        .map(|span| span.text.clone())
        .collect::<Vec<_>>()
        .join(" ");
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

fn has_korean_chars(text: &str) -> bool {
    text.chars().any(|c| {
        let code = c as u32;
        (0xAC00..=0xD7A3).contains(&code)
            || (0x1100..=0x11FF).contains(&code)
            || (0x3130..=0x318F).contains(&code)
            || (0xA960..=0xA97F).contains(&code)
            || (0xD7B0..=0xD7FF).contains(&code)
    })
}

fn hanja_count(text: &str) -> usize {
    text.chars()
        .filter(|c| {
            let code = *c as u32;
            (0x3400..=0x4DBF).contains(&code)
                || (0x4E00..=0x9FFF).contains(&code)
                || (0xF900..=0xFAFF).contains(&code)
        })
        .count()
}

fn korean_text_quality(text: &str) -> i32 {
    let mut syllables = 0_i32;
    let mut jamos = 0_i32;
    for c in text.chars() {
        let code = c as u32;
        if (0xAC00..=0xD7A3).contains(&code) {
            syllables += 1;
        } else if (0x1100..=0x11FF).contains(&code)
            || (0x3130..=0x318F).contains(&code)
            || (0xA960..=0xA97F).contains(&code)
            || (0xD7B0..=0xD7FF).contains(&code)
        {
            jamos += 1;
        }
    }
    (syllables * 2) - (jamos * 3)
}

fn is_korean_parser_degraded(text: &str) -> bool {
    has_korean_chars(text) && korean_text_quality(text) < -2
}

fn filter_degraded_parser_blocks(blocks: Vec<Block>) -> Vec<Block> {
    let ocr_texts: Vec<String> = blocks
        .iter()
        .filter_map(|block| match block {
            Block::TextBlock {
                source: Provenance::Ocr,
                ..
            } => block.text_content(),
            _ => None,
        })
        .collect();

    if ocr_texts.is_empty() {
        return blocks;
    }

    let best_ocr_quality = ocr_texts
        .iter()
        .map(|text| korean_text_quality(text))
        .max()
        .unwrap_or(i32::MIN);

    blocks
        .into_iter()
        .filter(|block| match block {
            Block::TextBlock {
                source: Provenance::Parser,
                ..
            } => {
                let text = block.text_content().unwrap_or_default();
                let parser_quality = korean_text_quality(&text);

                // If parser Korean is heavily decomposed while OCR text exists,
                // suppress parser block to avoid duplicated, corrupted output.
                !(has_korean_chars(&text)
                    && parser_quality < -10
                    && best_ocr_quality > parser_quality)
            }
            _ => true,
        })
        .collect()
}

fn filter_redundant_ocr_text_blocks(blocks: Vec<Block>) -> Vec<Block> {
    let parser_texts_with_area: Vec<(String, f32)> = blocks
        .iter()
        .filter_map(|block| match block {
            Block::TextBlock {
                bbox,
                source: Provenance::Parser | Provenance::Fused,
                ..
            } => block.text_content().map(|text| (text, bbox.area())),
            _ => None,
        })
        .collect();

    if parser_texts_with_area.is_empty() {
        return blocks;
    }

    let parser_total_len = parser_texts_with_area
        .iter()
        .map(|(t, _)| t.chars().count())
        .sum::<usize>();
    let parser_quality = parser_texts_with_area
        .iter()
        .map(|(t, _)| korean_text_quality(t))
        .max()
        .unwrap_or(i32::MIN);
    let parser_max_area = parser_texts_with_area
        .iter()
        .map(|(_, area)| *area)
        .fold(0.0_f32, f32::max);

    // Parser-dominant digital page: remove noisy duplicated OCR text fragments.
    // Thresholds are intentionally relaxed so pages like "mixed text + short sections"
    // still drop OCR duplicates when parser provides a broad, readable text block.
    let parser_dominant =
        parser_total_len >= 120 && parser_quality >= -2 && parser_max_area >= 300_000.0;
    if !parser_dominant {
        return blocks;
    }

    let parser_text_blocks: Vec<(String, crate::core::geometry::BBox)> = blocks
        .iter()
        .filter_map(|block| match block {
            Block::TextBlock {
                bbox,
                source: Provenance::Parser | Provenance::Fused,
                ..
            } => block.text_content().map(|text| (text, *bbox)),
            _ => None,
        })
        .collect();

    blocks
        .into_iter()
        .filter(|block| match block {
            Block::TextBlock {
                source: Provenance::Ocr,
                bbox,
                ..
            } => {
                let ocr_text = block.text_content().unwrap_or_default();
                !is_duplicate_ocr_text_block(&ocr_text, bbox, &parser_text_blocks)
            }
            _ => true,
        })
        .collect()
}

fn filter_low_quality_ocr_text_blocks(
    blocks: Vec<Block>,
    aggressive_short_filter: bool,
) -> Vec<Block> {
    blocks
        .into_iter()
        .filter(|block| match block {
            Block::TextBlock {
                source: Provenance::Ocr,
                ..
            } => {
                let text = block.text_content().unwrap_or_default();
                if text.trim().is_empty() {
                    return false;
                }
                if is_noisy_ocr_text(&text) {
                    return false;
                }
                let compact_len = normalize_text_for_compare(&text).chars().count();
                !(aggressive_short_filter && compact_len <= 3)
            }
            _ => true,
        })
        .collect()
}

fn filter_korean_ocr_when_parser_reliable(blocks: Vec<Block>, strict: bool) -> Vec<Block> {
    let parser_texts: Vec<(String, crate::core::geometry::BBox)> = blocks
        .iter()
        .filter_map(|block| match block {
            Block::TextBlock {
                bbox,
                source: Provenance::Parser | Provenance::Fused,
                ..
            } => block.text_content().map(|t| (t, *bbox)),
            _ => None,
        })
        .collect();

    if parser_texts.is_empty() {
        return blocks;
    }

    let parser_korean_chars: usize = parser_texts
        .iter()
        .map(|(t, _)| t.chars().filter(|c| ('가'..='힣').contains(c)).count())
        .sum();
    let parser_best_quality = parser_texts
        .iter()
        .map(|(t, _)| korean_text_quality(t))
        .max()
        .unwrap_or(i32::MIN);

    let parser_reliable = parser_korean_chars >= 18 && parser_best_quality >= -1;
    if !parser_reliable {
        return blocks;
    }

    blocks
        .into_iter()
        .filter(|block| match block {
            Block::TextBlock {
                source: Provenance::Ocr,
                ..
            } => {
                let ocr_text = block.text_content().unwrap_or_default();
                if !has_korean_chars(&ocr_text) {
                    return true;
                }
                let _ = strict;
                // Accuracy-first mode: if parser Korean is reliable, suppress OCR Korean text.
                false
            }
            _ => true,
        })
        .collect()
}

fn parser_reliable_for_accuracy(blocks: &[Block]) -> bool {
    let parser_texts: Vec<String> = blocks
        .iter()
        .filter_map(|block| match block {
            Block::TextBlock {
                source: Provenance::Parser | Provenance::Fused,
                ..
            } => block.text_content(),
            _ => None,
        })
        .collect();

    if parser_texts.is_empty() {
        return false;
    }

    let parser_chars = parser_texts
        .iter()
        .map(|t| t.chars().count())
        .sum::<usize>();
    let parser_text_blocks = parser_texts.len();
    let parser_korean_quality = parser_texts
        .iter()
        .map(|t| korean_text_quality(t))
        .max()
        .unwrap_or(i32::MIN);

    let ocr_chars = blocks
        .iter()
        .filter_map(|block| match block {
            Block::TextBlock {
                source: Provenance::Ocr,
                ..
            } => block.text_content().map(|t| t.chars().count()),
            _ => None,
        })
        .sum::<usize>();

    parser_chars >= 220
        && parser_text_blocks >= 1
        && (ocr_chars == 0 || parser_chars.saturating_mul(10) >= ocr_chars.saturating_mul(7))
        && parser_korean_quality >= -2
}

fn filter_ocr_text_when_parser_reliable(blocks: Vec<Block>, page_class: PageClass) -> Vec<Block> {
    if page_class == PageClass::Scanned || !parser_reliable_for_accuracy(&blocks) {
        return blocks;
    }

    blocks
        .into_iter()
        .filter(|block| {
            !matches!(
                block,
                Block::TextBlock {
                    source: Provenance::Ocr,
                    ..
                }
            )
        })
        .collect()
}

fn is_noisy_ocr_text(text: &str) -> bool {
    let norm = normalize_text_for_compare(text);
    if norm.chars().count() <= 1 {
        return true;
    }

    let alnum_or_korean = norm
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || ('가'..='힣').contains(c))
        .count();
    if alnum_or_korean <= 1 {
        return true;
    }

    let unique = norm.chars().collect::<std::collections::HashSet<_>>().len();
    if norm.chars().count() >= 6 && unique <= 2 {
        return true;
    }

    let hangul_present = has_korean_chars(&norm);
    let hanja = hanja_count(&norm);
    if hangul_present && hanja >= 2 {
        return true;
    }

    has_korean_chars(&norm) && korean_text_quality(&norm) < -6
}

fn is_duplicate_ocr_text_block(
    ocr_text: &str,
    ocr_bbox: &crate::core::geometry::BBox,
    parser_text_blocks: &[(String, crate::core::geometry::BBox)],
) -> bool {
    let ocr_norm = normalize_text_for_compare(ocr_text);
    if ocr_norm.len() < 4 {
        return true;
    }

    parser_text_blocks.iter().any(|(parser_text, parser_bbox)| {
        let parser_norm = normalize_text_for_compare(parser_text);
        if parser_norm.is_empty() {
            return false;
        }

        // Strong textual containment means OCR is likely just a duplicate snippet.
        if ocr_norm.len() >= 8 && parser_norm.contains(&ocr_norm) {
            return true;
        }

        let similarity = text_similarity(&parser_norm, &ocr_norm);
        similarity >= 0.82 || (ocr_bbox.iou(parser_bbox) >= 0.55 && similarity >= 0.55)
    })
}

fn normalize_text_for_compare(text: &str) -> String {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::geometry::BBox;
    use crate::core::model::{Line, Span};

    fn text_block(text: &str, source: Provenance, bbox: BBox) -> Block {
        Block::TextBlock {
            bbox,
            lines: vec![Line {
                spans: vec![Span {
                    text: text.to_string(),
                    bbox,
                    source,
                    style: None,
                }],
            }],
            confidence: 0.5,
            source,
            debug: None,
        }
    }

    #[test]
    fn parser_dominant_filter_drops_duplicate_ocr_snippet() {
        let parser_text = "This is a long parser block with enough content to be parser dominant \
            and should suppress duplicate OCR snippets on the same region of the page.";
        let parser = text_block(
            parser_text,
            Provenance::Parser,
            BBox::new(0.0, 0.0, 900.0, 700.0),
        );
        let ocr_dup = text_block(
            "duplicate OCR snippets on the same region",
            Provenance::Ocr,
            BBox::new(50.0, 60.0, 300.0, 120.0),
        );

        let filtered = filter_redundant_ocr_text_blocks(vec![parser, ocr_dup]);

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].provenance(), Provenance::Parser);
    }

    #[test]
    fn parser_dominant_filter_keeps_unique_ocr_text() {
        let parser_text = "This is a long parser block with enough content to be parser dominant \
            and should only remove OCR blocks that are clearly duplicated.";
        let parser = text_block(
            parser_text,
            Provenance::Parser,
            BBox::new(0.0, 0.0, 900.0, 700.0),
        );
        let ocr_unique = text_block(
            "Appendix C checksum 9A7F",
            Provenance::Ocr,
            BBox::new(50.0, 720.0, 320.0, 760.0),
        );

        let filtered = filter_redundant_ocr_text_blocks(vec![parser, ocr_unique]);

        assert_eq!(filtered.len(), 2);
        assert!(filtered
            .iter()
            .any(|block| block.provenance() == Provenance::Ocr));
    }

    #[test]
    fn filters_noisy_ocr_text_block() {
        let parser = text_block(
            "정상 파서 텍스트 블록입니다.",
            Provenance::Parser,
            BBox::new(0.0, 0.0, 800.0, 300.0),
        );
        let ocr_noise = text_block(
            "ㅁ ㅁ ㅁ ㅁ ㅁ",
            Provenance::Ocr,
            BBox::new(10.0, 320.0, 180.0, 350.0),
        );

        let filtered = filter_low_quality_ocr_text_blocks(vec![parser, ocr_noise], false);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].provenance(), Provenance::Parser);
    }

    #[test]
    fn filters_hanja_mixed_korean_noise() {
        let parser = text_block(
            "이것은 OCR 테스트 문서입니다.",
            Provenance::Parser,
            BBox::new(0.0, 0.0, 800.0, 300.0),
        );
        let ocr_noise = text_block(
            "이것은 哲 豆 吳 테스트 문서",
            Provenance::Ocr,
            BBox::new(10.0, 320.0, 280.0, 360.0),
        );

        let filtered = filter_low_quality_ocr_text_blocks(vec![parser, ocr_noise], false);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].provenance(), Provenance::Parser);
    }

    #[test]
    fn drops_korean_ocr_when_parser_is_reliable() {
        let parser = text_block(
            "이것은 OCR 테스트 문서입니다. 영어와 한글을 동시에 처리할 수 있는지 확인합니다.",
            Provenance::Parser,
            BBox::new(0.0, 0.0, 900.0, 500.0),
        );
        let ocr_korean = text_block(
            "이것은 OCR 테스트 문서 입니다",
            Provenance::Ocr,
            BBox::new(10.0, 10.0, 420.0, 60.0),
        );
        let ocr_english = text_block(
            "English heading",
            Provenance::Ocr,
            BBox::new(20.0, 520.0, 260.0, 560.0),
        );

        let filtered =
            filter_korean_ocr_when_parser_reliable(vec![parser, ocr_korean, ocr_english], true);
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().any(|b| matches!(
            b,
            Block::TextBlock {
                source: Provenance::Ocr,
                ..
            }
        )));
        assert!(!filtered.iter().any(|b| {
            matches!(
                b,
                Block::TextBlock {
                    source: Provenance::Ocr,
                    ..
                }
            ) && b.text_content().unwrap_or_default().contains("문서")
        }));
    }

    #[test]
    fn drops_all_ocr_text_when_parser_is_reliable_non_scanned() {
        let parser = text_block(
            "DocStruct Stress Test Comprehensive PDF Feature Test mixed content equations tables graphics symbols \
            section one narrative text with multiple clauses and punctuation for parser dominance \
            section two equations and inline notation with additional wording to increase reliable parser coverage \
            section three tables lists diagrams hyperlinks and code fragments to represent broad page coverage",
            Provenance::Parser,
            BBox::new(0.0, 0.0, 900.0, 700.0),
        );
        let ocr_noise = text_block(
            "VxE=——, ot' V-B=0",
            Provenance::Ocr,
            BBox::new(40.0, 710.0, 420.0, 760.0),
        );

        let filtered =
            filter_ocr_text_when_parser_reliable(vec![parser, ocr_noise], PageClass::Hybrid);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].provenance(), Provenance::Parser);
    }
}
