use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone)]
pub struct RenderedPage {
    pub path: PathBuf,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone)]
pub struct PageRenderer {
    out_dir: PathBuf,
    dpi: u32,
}

impl PageRenderer {
    pub fn new(out_dir: PathBuf, dpi: u32) -> Self {
        Self { out_dir, dpi }
    }

    pub fn render_page(&self, pdf_path: &Path, page_idx: usize) -> Result<RenderedPage> {
        fs::create_dir_all(&self.out_dir)?;

        // pdftoppm uses 1-based page indices
        let page_number = page_idx + 1;
        let prefix = self.out_dir.join(format!("page_{:03}", page_number));
        let prefix_str = prefix
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("non-UTF8 output path not supported"))?;

        let status = Command::new("pdftoppm")
            .arg("-png")
            .arg("-r")
            .arg(self.dpi.to_string())
            .arg("-f")
            .arg(page_number.to_string())
            .arg("-l")
            .arg(page_number.to_string())
            .arg(pdf_path)
            .arg(prefix_str)
            .status()
            .with_context(|| "failed to invoke pdftoppm; is poppler-utils installed?")?;

        if !status.success() {
            anyhow::bail!("pdftoppm failed with status: {status}");
        }

        // pdftoppm will create a file like `<prefix>-1.png` for this page
        let image_path = self
            .out_dir
            .join(format!("page_{:03}-{}.png", page_number, page_number));

        if !image_path.exists() {
            anyhow::bail!(
                "expected rendered image not found: {}",
                image_path.display()
            );
        }

        // We could inspect the image to get exact dimensions in the future.
        // For now, approximate using the default layout used elsewhere.
        let width = (1000.0 * self.dpi as f32 / 200.0) as u32;
        let height = (1400.0 * self.dpi as f32 / 200.0) as u32;

        Ok(RenderedPage {
            path: image_path,
            width,
            height,
        })
    }
}
