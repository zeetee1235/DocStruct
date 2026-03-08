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

        // pdftoppm naming varies by version (`-1`, `-01`, etc.). Find the
        // rendered output by prefix rather than assuming one suffix pattern.
        let page_prefix = format!("page_{:03}-", page_number);
        let image_path = fs::read_dir(&self.out_dir)?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.is_file())
            .find(|path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .map(|name| name.starts_with(&page_prefix) && name.ends_with(".png"))
                    .unwrap_or(false)
            })
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "expected rendered image not found for prefix {} in {}",
                    page_prefix,
                    self.out_dir.display()
                )
            })?;

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
