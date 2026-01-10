use crate::core::model::Block;

#[derive(Debug, Clone)]
pub struct MatchedPair {
    pub a: Block,
    pub b: Block,
    pub iou: f32,
    pub center_distance: f32,
}

#[derive(Debug, Clone)]
pub struct AlignmentResult {
    pub matched: Vec<MatchedPair>,
    pub unmatched_a: Vec<Block>,
    pub unmatched_b: Vec<Block>,
}

pub fn align_blocks(a_blocks: &[Block], b_blocks: &[Block]) -> AlignmentResult {
    let mut matched = Vec::new();
    let mut unmatched_a = Vec::new();
    let mut used_b = vec![false; b_blocks.len()];

    for a in a_blocks {
        let mut best_idx = None;
        let mut best_score = 0.0;
        for (idx, b) in b_blocks.iter().enumerate() {
            if used_b[idx] {
                continue;
            }
            let iou = a.bbox().iou(&b.bbox());
            let dist = a.bbox().center_distance(&b.bbox());
            let kind_bonus = if a.kind() == b.kind() { 0.1 } else { 0.0 };
            let score = iou + kind_bonus - dist / 10000.0;
            if score > best_score {
                best_score = score;
                best_idx = Some(idx);
            }
        }

        if let Some(idx) = best_idx {
            let b = b_blocks[idx].clone();
            let iou = a.bbox().iou(&b.bbox());
            let dist = a.bbox().center_distance(&b.bbox());
            if iou > 0.1 || dist < 150.0 {
                used_b[idx] = true;
                matched.push(MatchedPair {
                    a: a.clone(),
                    b,
                    iou,
                    center_distance: dist,
                });
            } else {
                unmatched_a.push(a.clone());
            }
        } else {
            unmatched_a.push(a.clone());
        }
    }

    let mut unmatched_b = Vec::new();
    for (idx, b) in b_blocks.iter().enumerate() {
        if !used_b[idx] {
            unmatched_b.push(b.clone());
        }
    }

    AlignmentResult {
        matched,
        unmatched_a,
        unmatched_b,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::geometry::BBox;
    use crate::core::model::{Block, Line, Provenance, Span};

    fn text_block(bbox: BBox) -> Block {
        Block::TextBlock {
            bbox,
            lines: vec![Line {
                spans: vec![Span {
                    text: "hello".to_string(),
                    bbox,
                    source: Provenance::Parser,
                    style: None,
                }],
            }],
            confidence: 0.5,
            source: Provenance::Parser,
            debug: None,
        }
    }

    #[test]
    fn aligns_overlapping_blocks() {
        let a = text_block(BBox::new(0.0, 0.0, 50.0, 50.0));
        let b = text_block(BBox::new(10.0, 10.0, 60.0, 60.0));
        let result = align_blocks(&[a.clone()], &[b.clone()]);
        assert_eq!(result.matched.len(), 1);
        assert!(result.unmatched_a.is_empty());
        assert!(result.unmatched_b.is_empty());
    }
}
