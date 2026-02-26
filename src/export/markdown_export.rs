use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use image::{GenericImageView, ImageReader};

use crate::core::model::{Block, DocumentFinal, Provenance};
use crate::export::Exporter;

#[derive(Debug, Clone)]
pub struct MarkdownExporter {
    out_dir: PathBuf,
    image_dir: PathBuf,
}

impl MarkdownExporter {
    pub fn new(out_dir: PathBuf) -> Self {
        let image_dir = out_dir.join("figures");
        Self { out_dir, image_dir }
    }

    fn crop_block_image(
        &self,
        page_image_path: &PathBuf,
        bbox: &crate::core::geometry::BBox,
        page_idx: usize,
        block_idx: usize,
        block_type: &str,
    ) -> Result<String> {
        // Load the page image
        let img = ImageReader::open(page_image_path)?.decode()?;

        // Ensure coordinates are within image bounds
        let (img_width, img_height) = img.dimensions();
        let x0 = bbox.x0.max(0.0) as u32;
        let y0 = bbox.y0.max(0.0) as u32;
        let x1 = (bbox.x1.min(img_width as f32)) as u32;
        let y1 = (bbox.y1.min(img_height as f32)) as u32;

        if x1 <= x0 || y1 <= y0 {
            return Ok(String::new());
        }

        let width = x1 - x0;
        let height = y1 - y0;

        // Crop the image
        let cropped = img.crop_imm(x0, y0, width, height);

        // Save the cropped image
        fs::create_dir_all(&self.image_dir)?;
        let filename = format!(
            "page_{:03}_{}__{:02}.png",
            page_idx + 1,
            block_type,
            block_idx
        );
        let output_path = self.image_dir.join(&filename);
        cropped.save(&output_path)?;

        // Return relative path for markdown
        Ok(format!("figures/{}", filename))
    }

    fn format_block(
        &self,
        block: &Block,
        page_idx: usize,
        block_idx: usize,
        page_image_path: &PathBuf,
    ) -> Result<String> {
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
                    return Ok(String::new());
                }
                if Self::should_skip_noisy_ocr_text(*source, &text) {
                    return Ok(String::new());
                }
                Ok(text)
            }
            Block::TableBlock { bbox, .. } => {
                // Crop table image
                let img_path =
                    self.crop_block_image(page_image_path, bbox, page_idx, block_idx, "table")?;

                if img_path.is_empty() {
                    return Ok(String::new());
                }

                Ok(format!(
                    "\n**Table {}:**\n\n![Table]({})\n",
                    block_idx + 1,
                    img_path
                ))
            }
            Block::FigureBlock { bbox, .. } => {
                // Crop figure image
                let img_path =
                    self.crop_block_image(page_image_path, bbox, page_idx, block_idx, "figure")?;

                if img_path.is_empty() {
                    return Ok(String::new());
                }

                Ok(format!(
                    "\n**Figure {}:**\n\n![Figure]({})\n",
                    block_idx + 1,
                    img_path
                ))
            }
            Block::MathBlock { bbox, latex, .. } => {
                // If we have LaTeX, use it; otherwise crop image
                if let Some(latex_str) = latex {
                    if !latex_str.is_empty() {
                        return Ok(format!(
                            "\n**Math Equation {}:**\n\n$$\n{}\n$$\n",
                            block_idx + 1,
                            latex_str
                        ));
                    }
                }

                // Fallback: crop math image
                let img_path =
                    self.crop_block_image(page_image_path, bbox, page_idx, block_idx, "math")?;

                if img_path.is_empty() {
                    return Ok(String::new());
                }

                Ok(format!(
                    "\n**Math Equation {}:**\n\n![Math]({})\n",
                    block_idx + 1,
                    img_path
                ))
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

    fn should_skip_noisy_ocr_text(source: Provenance, text: &str) -> bool {
        if source != Provenance::Ocr {
            return false;
        }
        let compact = text.split_whitespace().collect::<String>();
        if compact.chars().count() <= 2 {
            return true;
        }

        let alnum_or_hangul = compact
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || ('가'..='힣').contains(c))
            .count();
        let total = compact.chars().count();
        if total > 0 && alnum_or_hangul * 2 < total {
            return true;
        }

        let symbol_heavy = text
            .chars()
            .filter(|c| !c.is_alphanumeric() && !c.is_whitespace())
            .count();
        symbol_heavy >= 8 && symbol_heavy > alnum_or_hangul
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

impl Exporter for MarkdownExporter {
    fn export(&self, document: &DocumentFinal) -> Result<()> {
        fs::create_dir_all(&self.out_dir)?;
        fs::create_dir_all(&self.image_dir)?;

        // Export full document as markdown
        let mut markdown = String::new();
        markdown.push_str("# Document\n\n");

        for page in &document.pages {
            markdown.push_str(&format!("---\n\n## Page {}\n\n", page.page_idx + 1));

            // Find the rendered page image
            let page_image_path = self.out_dir.join(format!(
                "debug/page_{:03}-{}.png",
                page.page_idx + 1,
                page.page_idx + 1
            ));

            // If debug image doesn't exist, skip block image cropping
            let has_debug_image = page_image_path.exists();

            for (block_idx, block) in page.blocks.iter().enumerate() {
                let block_text = if has_debug_image {
                    self.format_block(block, page.page_idx, block_idx, &page_image_path)?
                } else {
                    // Fallback to simple text representation
                    match block {
                        Block::TextBlock { lines, source, .. } => {
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
                                String::new()
                            } else if Self::should_skip_noisy_ocr_text(*source, &text) {
                                String::new()
                            } else {
                                text
                            }
                        }
                        Block::TableBlock { bbox, .. } => {
                            format!(
                                "\n[TABLE: {:.0}x{:.0} at ({:.0}, {:.0})]\n",
                                bbox.width(),
                                bbox.height(),
                                bbox.x0,
                                bbox.y0
                            )
                        }
                        Block::FigureBlock { bbox, .. } => {
                            format!(
                                "\n[FIGURE: {:.0}x{:.0} at ({:.0}, {:.0})]\n",
                                bbox.width(),
                                bbox.height(),
                                bbox.x0,
                                bbox.y0
                            )
                        }
                        Block::MathBlock { bbox, latex, .. } => {
                            if let Some(latex_str) = latex {
                                if !latex_str.is_empty() {
                                    format!("\n$$\n{}\n$$\n", latex_str)
                                } else {
                                    format!(
                                        "\n[MATH: {:.0}x{:.0} at ({:.0}, {:.0})]\n",
                                        bbox.width(),
                                        bbox.height(),
                                        bbox.x0,
                                        bbox.y0
                                    )
                                }
                            } else {
                                format!(
                                    "\n[MATH: {:.0}x{:.0} at ({:.0}, {:.0})]\n",
                                    bbox.width(),
                                    bbox.height(),
                                    bbox.x0,
                                    bbox.y0
                                )
                            }
                        }
                    }
                };

                if !block_text.is_empty() {
                    markdown.push_str(&block_text);
                    markdown.push_str("\n\n");
                }
            }
        }

        let output_path = self.out_dir.join("document.md");
        fs::write(output_path, markdown)?;

        // Export per-page markdown files
        for page in &document.pages {
            let mut page_markdown = String::new();
            page_markdown.push_str(&format!("# Page {}\n\n", page.page_idx + 1));

            let page_image_path = self.out_dir.join(format!(
                "debug/page_{:03}-{}.png",
                page.page_idx + 1,
                page.page_idx + 1
            ));
            let has_debug_image = page_image_path.exists();

            for (block_idx, block) in page.blocks.iter().enumerate() {
                let block_text = if has_debug_image {
                    self.format_block(block, page.page_idx, block_idx, &page_image_path)?
                } else {
                    match block {
                        Block::TextBlock { lines, source, .. } => {
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
                                String::new()
                            } else if Self::should_skip_noisy_ocr_text(*source, &text) {
                                String::new()
                            } else {
                                text
                            }
                        }
                        Block::TableBlock { bbox, .. } => {
                            format!("\n[TABLE: {:.0}x{:.0}]\n", bbox.width(), bbox.height())
                        }
                        Block::FigureBlock { bbox, .. } => {
                            format!("\n[FIGURE: {:.0}x{:.0}]\n", bbox.width(), bbox.height())
                        }
                        Block::MathBlock { bbox, latex, .. } => {
                            if let Some(latex_str) = latex {
                                if !latex_str.is_empty() {
                                    format!("\n$$\n{}\n$$\n", latex_str)
                                } else {
                                    format!("\n[MATH: {:.0}x{:.0}]\n", bbox.width(), bbox.height())
                                }
                            } else {
                                format!("\n[MATH: {:.0}x{:.0}]\n", bbox.width(), bbox.height())
                            }
                        }
                    }
                };

                if !block_text.is_empty() {
                    page_markdown.push_str(&block_text);
                    page_markdown.push_str("\n\n");
                }
            }

            let page_output_path = self
                .out_dir
                .join(format!("page_{:03}.md", page.page_idx + 1));
            fs::write(page_output_path, page_markdown)?;
        }

        Ok(())
    }
}
