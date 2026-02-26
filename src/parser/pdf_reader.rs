use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone)]
pub struct PdfReader {
    path: PathBuf,
}

impl PdfReader {
    pub fn new(path: PathBuf) -> Result<Self> {
        Ok(Self { path })
    }

    pub fn page_count(&self) -> Result<usize> {
        get_page_count(&self.path)
    }
}

fn get_page_count(pdf_path: &Path) -> Result<usize> {
    let output = Command::new("pdfinfo")
        .arg(pdf_path)
        .output()
        .with_context(|| format!("failed to invoke pdfinfo on {}", pdf_path.display()))?;

    if !output.status.success() {
        anyhow::bail!("pdfinfo failed with status: {}", output.status);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if let Some(rest) = line.strip_prefix("Pages:") {
            let num_str = rest.trim();
            let pages: usize = num_str.parse().with_context(|| {
                format!("failed to parse page count from 'Pages:' line: {num_str}")
            })?;
            return Ok(pages);
        }
    }

    anyhow::bail!(
        "pdfinfo output did not contain a 'Pages:' line for {}",
        pdf_path.display()
    );
}
