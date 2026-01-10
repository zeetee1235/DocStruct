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

    blocks
}

fn promote_single(block: Block, provenance: Provenance) -> Block {
    match block {
        Block::TextBlock { bbox, lines, .. } => {
            let confidence = score_confidence(
                provenance == Provenance::Parser,
                provenance == Provenance::Ocr,
                None,
                true,
            );
            let final_text = text_from_lines(&lines);
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
        Block::MathBlock { bbox, .. } => Block::MathBlock {
            bbox,
            confidence: score_confidence(provenance == Provenance::Parser, provenance == Provenance::Ocr, None, true),
            source: provenance,
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

    let confidence = score_confidence(true, true, similarity, geometry_good);

    match (&pair.a, &pair.b) {
        (Block::TextBlock { bbox, lines, .. }, Block::TextBlock { .. }) => {
            let (final_lines, provenance) = if similarity.unwrap_or(0.0) >= 0.7 {
                (lines.clone(), Provenance::Fused)
            } else {
                (lines.clone(), Provenance::Parser)
            };
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
