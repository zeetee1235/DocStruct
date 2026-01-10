use anyhow::Result;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct PdfReader {
    path: PathBuf,
}

impl PdfReader {
    pub fn new(path: PathBuf) -> Result<Self> {
        Ok(Self { path })
    }

    pub fn page_count(&self) -> usize {
        let _ = &self.path;
        1
    }
}
