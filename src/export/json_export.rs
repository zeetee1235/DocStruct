use std::fs;
use std::path::PathBuf;

use anyhow::Result;

use crate::core::model::DocumentFinal;
use crate::export::Exporter;

#[derive(Debug, Clone)]
pub struct JsonExporter {
    out_dir: PathBuf,
}

impl JsonExporter {
    pub fn new(out_dir: PathBuf) -> Self {
        Self { out_dir }
    }
}

impl Exporter for JsonExporter {
    fn export(&self, document: &DocumentFinal) -> Result<()> {
        fs::create_dir_all(&self.out_dir)?;
        let path = self.out_dir.join("document.json");
        let data = serde_json::to_string_pretty(document)?;
        fs::write(path, data)?;
        Ok(())
    }
}
