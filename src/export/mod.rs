pub mod html_debug_export;
pub mod json_export;
pub mod markdown_export;
pub mod text_export;

use anyhow::Result;

use crate::core::model::DocumentFinal;

pub use html_debug_export::HtmlDebugExporter;
pub use json_export::JsonExporter;
pub use markdown_export::MarkdownExporter;
pub use text_export::TextExporter;

pub trait Exporter {
    fn export(&self, document: &DocumentFinal) -> Result<()>;
}
