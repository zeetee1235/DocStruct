use anyhow::{Context, Result};
use std::path::PathBuf;

use crate::parser::docx_parser::DocxParser;
use crate::parser::pdf_parser::PdfParser;
use crate::parser::pptx_parser::PptxParser;
use crate::parser::ParserTrack;

pub struct ParserLayoutBuilder {
    parser: Box<dyn ParserTrack>,
}

impl ParserLayoutBuilder {
    pub fn new(path: PathBuf) -> Result<Self> {
        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();

        let parser: Box<dyn ParserTrack> = match ext.as_str() {
            "pdf" => Box::new(PdfParser::new(path)),
            "docx" => Box::new(DocxParser::new(path)?),
            "pptx" => Box::new(PptxParser::new(path)?),
            "ppt" => Box::new(PptxParser::from_ppt(path)?),
            _ => {
                anyhow::bail!("unsupported input format: .{ext}. supported: pdf, docx, ppt, pptx")
            }
        };

        Ok(Self { parser })
    }
}

impl ParserTrack for ParserLayoutBuilder {
    fn page_count(&self) -> Result<usize> {
        self.parser.page_count().context("failed to count pages")
    }

    fn analyze_page(&self, page_idx: usize) -> Result<crate::core::model::PageHypothesis> {
        self.parser
            .analyze_page(page_idx)
            .with_context(|| format!("parser failed on page {}", page_idx + 1))
    }

    fn supports_ocr_rendering(&self) -> bool {
        self.parser.supports_ocr_rendering()
    }

    fn rendering_source_path(&self) -> Option<&std::path::Path> {
        self.parser.rendering_source_path()
    }
}
