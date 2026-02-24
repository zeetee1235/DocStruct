use std::fs;
use std::path::PathBuf;

use anyhow::Result;

use crate::core::model::{Block, DocumentFinal, Provenance};
use crate::export::Exporter;

#[derive(Debug, Clone)]
pub struct TextExporter {
    out_dir: PathBuf,
}

impl TextExporter {
    pub fn new(out_dir: PathBuf) -> Self {
        Self { out_dir }
    }

    fn format_block(block: &Block) -> String {
        match block {
            Block::TextBlock { lines, source, .. } => {
                // Extract text from all spans in all lines
                let text = lines
                    .iter()
                    .map(|line| {
                        line.spans
                            .iter()
                            .map(|span| span.text.as_str())
                            .collect::<Vec<_>>()
                            .join("")
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                if Self::should_skip_degraded_parser_text(*source, &text) {
                    return String::new();
                }
                text
            }
            Block::TableBlock { bbox, .. } => {
                format!("[TABLE at x:{:.0} y:{:.0} w:{:.0} h:{:.0}]", 
                    bbox.x0, bbox.y0, bbox.width(), bbox.height())
            }
            Block::FigureBlock { bbox, .. } => {
                format!("[FIGURE at x:{:.0} y:{:.0} w:{:.0} h:{:.0}]", 
                    bbox.x0, bbox.y0, bbox.width(), bbox.height())
            }
            Block::MathBlock { bbox, .. } => {
                format!("[MATH at x:{:.0} y:{:.0} w:{:.0} h:{:.0}]", 
                    bbox.x0, bbox.y0, bbox.width(), bbox.height())
            }
        }
    }

    fn should_skip_degraded_parser_text(source: Provenance, text: &str) -> bool {
        if source != Provenance::Parser {
            return false;
        }
        let (syllables, jamos) = Self::korean_counts(text);
        let has_korean = syllables + jamos > 0;
        has_korean && jamos >= syllables * 2 && jamos >= 8
    }

    fn korean_counts(text: &str) -> (usize, usize) {
        let mut syllables = 0usize;
        let mut jamos = 0usize;
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
        (syllables, jamos)
    }
}

impl Exporter for TextExporter {
    fn export(&self, document: &DocumentFinal) -> Result<()> {
        fs::create_dir_all(&self.out_dir)?;
        
        // Export full document
        let mut full_text = String::new();
        for page in &document.pages {
            full_text.push_str(&format!("=== Page {} ===\n\n", page.page_idx + 1));
            for block in &page.blocks {
                let block_text = Self::format_block(block);
                if !block_text.is_empty() {
                    full_text.push_str(&block_text);
                    full_text.push_str("\n\n");
                }
            }
            full_text.push_str("\n");
        }
        
        let full_path = self.out_dir.join("document.txt");
        fs::write(full_path, full_text)?;
        
        // Export per-page text files
        for page in &document.pages {
            let mut page_text = String::new();
            for block in &page.blocks {
                let block_text = Self::format_block(block);
                if !block_text.is_empty() {
                    page_text.push_str(&block_text);
                    page_text.push_str("\n\n");
                }
            }
            
            let page_path = self.out_dir.join(format!("page_{:03}.txt", page.page_idx + 1));
            fs::write(page_path, page_text)?;
        }
        
        Ok(())
    }
}
