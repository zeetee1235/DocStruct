use anyhow::Result;
use image::{ImageBuffer, Rgb};
use std::fs;
use std::path::{Path, PathBuf};

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

    pub fn render_page(&self, _pdf_path: &Path, page_idx: usize) -> Result<RenderedPage> {
        fs::create_dir_all(&self.out_dir)?;
        let width = (1000.0 * self.dpi as f32 / 200.0) as u32;
        let height = (1400.0 * self.dpi as f32 / 200.0) as u32;
        let mut image = ImageBuffer::from_pixel(width, height, Rgb([255, 255, 255]));
        let filename = format!("page_{:03}.png", page_idx + 1);
        let path = self.out_dir.join(filename);
        image.save(&path)?;
        Ok(RenderedPage { path, width, height })
    }
}
