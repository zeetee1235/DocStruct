use crate::core::confidence::score_confidence;
use crate::core::model::{Block, BlockDebug, Line, Provenance};
use crate::fusion::align::{AlignmentResult, MatchedPair};
use crate::fusion::compare::text_similarity;

pub fn resolve_blocks(alignment: &AlignmentResult) -> Vec<Block> {
    let mut blocks = Vec::new();

    for pair in &alignment.matched {
        blocks.push(resolve_pair(pair));
    }

    for block in &alignment.unmatched_a {
        blocks.push(promote_single(block.clone(), Provenance::Parser));
    }

    for block in &alignment.unmatched_b {
        blocks.push(promote_single(block.clone(), Provenance::Ocr));
    }

    let blocks = filter_degraded_parser_blocks(blocks);
    filter_redundant_ocr_text_blocks(blocks)
}

fn promote_single(block: Block, provenance: Provenance) -> Block {
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
            confidence: score_confidence(provenance == Provenance::Parser, provenance == Provenance::Ocr, None, true),
            source: provenance,
            debug: None,
        },
        Block::FigureBlock { bbox, .. } => Block::FigureBlock {
            bbox,
            confidence: score_confidence(provenance == Provenance::Parser, provenance == Provenance::Ocr, None, true),
            source: provenance,
            debug: None,
        },
        Block::MathBlock { bbox, latex, .. } => Block::MathBlock {
            bbox,
            confidence: score_confidence(provenance == Provenance::Parser, provenance == Provenance::Ocr, None, true),
            source: provenance,
            latex: latex.clone(),
            debug: None,
        },
    }
}

fn resolve_pair(pair: &MatchedPair) -> Block {
    let geometry_good = pair.iou > 0.3 || pair.center_distance < 50.0;
    let a_text = pair.a.text_content();
    let b_text = pair.b.text_content();

    let similarity = match (&a_text, &b_text) {
        (Some(a), Some(b)) => Some(text_similarity(a, b)),
        _ => None,
    };

    let mut confidence = score_confidence(true, true, similarity, geometry_good);

    match (&pair.a, &pair.b) {
        (Block::TextBlock { bbox, lines: parser_lines, .. }, Block::TextBlock { lines: ocr_lines, .. }) => {
            let parser_text = a_text.as_deref().unwrap_or_default();
            let ocr_text = b_text.as_deref().unwrap_or_default();
            let parser_quality = korean_text_quality(parser_text);
            let ocr_quality = korean_text_quality(ocr_text);
            let korean_present = has_korean_chars(parser_text) || has_korean_chars(ocr_text);

            let (final_lines, provenance) = if similarity.unwrap_or(0.0) >= 0.7 {
                (parser_lines.clone(), Provenance::Fused)
            } else if korean_present && ocr_quality > parser_quality + 2 {
                (ocr_lines.clone(), Provenance::Ocr)
            } else {
                (parser_lines.clone(), Provenance::Parser)
            };

            if korean_present && provenance == Provenance::Parser && parser_quality < -2 {
                confidence = (confidence - 0.2).clamp(0.0, 1.0);
            }
            if korean_present && provenance == Provenance::Ocr && ocr_quality > 0 {
                confidence = (confidence + 0.05).clamp(0.0, 1.0);
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
            Block::TextBlock { source: Provenance::Ocr, .. } => block.text_content(),
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
            Block::TextBlock { source: Provenance::Parser, .. } => {
                let text = block.text_content().unwrap_or_default();
                let parser_quality = korean_text_quality(&text);

                // If parser Korean is heavily decomposed while OCR text exists,
                // suppress parser block to avoid duplicated, corrupted output.
                !(has_korean_chars(&text) && parser_quality < -10 && best_ocr_quality > parser_quality)
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

    blocks
        .into_iter()
        .filter(|block| !matches!(block, Block::TextBlock { source: Provenance::Ocr, .. }))
        .collect()
}
