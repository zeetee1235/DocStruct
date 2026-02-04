use serde::{Deserialize, Serialize};

use crate::core::geometry::BBox;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Provenance {
    Parser,
    Ocr,
    Fused,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PageClass {
    Digital,
    Scanned,
    Hybrid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentFinal {
    pub pages: Vec<PageFinal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageFinal {
    pub page_idx: usize,
    pub class: PageClass,
    pub blocks: Vec<Block>,
    pub width: u32,
    pub height: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debug: Option<PageDebug>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageHypothesis {
    pub page_idx: usize,
    pub blocks: Vec<Block>,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageDebug {
    pub parser_blocks: Vec<Block>,
    pub ocr_blocks: Vec<Block>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Block {
    TextBlock {
        bbox: BBox,
        lines: Vec<Line>,
        confidence: f32,
        source: Provenance,
        debug: Option<BlockDebug>,
    },
    TableBlock {
        bbox: BBox,
        confidence: f32,
        source: Provenance,
        debug: Option<BlockDebug>,
    },
    FigureBlock {
        bbox: BBox,
        confidence: f32,
        source: Provenance,
        debug: Option<BlockDebug>,
    },
    MathBlock {
        bbox: BBox,
        confidence: f32,
        source: Provenance,
        #[serde(skip_serializing_if = "Option::is_none")]
        latex: Option<String>,
        debug: Option<BlockDebug>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Line {
    pub spans: Vec<Span>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Span {
    pub text: String,
    pub bbox: BBox,
    pub source: Provenance,
    pub style: Option<TextStyle>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextStyle {
    pub font: Option<String>,
    pub size: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockDebug {
    pub parser_text: Option<String>,
    pub ocr_text: Option<String>,
    pub final_text: Option<String>,
    pub similarity: Option<f32>,
}

impl Block {
    pub fn bbox(&self) -> BBox {
        match self {
            Block::TextBlock { bbox, .. }
            | Block::TableBlock { bbox, .. }
            | Block::FigureBlock { bbox, .. }
            | Block::MathBlock { bbox, .. } => *bbox,
        }
    }

    pub fn provenance(&self) -> Provenance {
        match self {
            Block::TextBlock { source, .. }
            | Block::TableBlock { source, .. }
            | Block::FigureBlock { source, .. }
            | Block::MathBlock { source, .. } => *source,
        }
    }

    pub fn confidence(&self) -> f32 {
        match self {
            Block::TextBlock { confidence, .. }
            | Block::TableBlock { confidence, .. }
            | Block::FigureBlock { confidence, .. }
            | Block::MathBlock { confidence, .. } => *confidence,
        }
    }

    pub fn text_content(&self) -> Option<String> {
        match self {
            Block::TextBlock { lines, .. } => {
                let text = lines
                    .iter()
                    .flat_map(|line| line.spans.iter())
                    .map(|span| span.text.clone())
                    .collect::<Vec<_>>()
                    .join(" ");
                Some(text)
            }
            _ => None,
        }
    }

    pub fn kind(&self) -> &'static str {
        match self {
            Block::TextBlock { .. } => "text",
            Block::TableBlock { .. } => "table",
            Block::FigureBlock { .. } => "figure",
            Block::MathBlock { .. } => "math",
        }
    }
}
