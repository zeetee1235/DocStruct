pub mod html_debug_export;
pub mod json_export;

use anyhow::Result;

use crate::core::model::DocumentFinal;

pub use html_debug_export::HtmlDebugExporter;
pub use json_export::JsonExporter;

pub trait Exporter {
    fn export(&self, document: &DocumentFinal) -> Result<()>;
}
