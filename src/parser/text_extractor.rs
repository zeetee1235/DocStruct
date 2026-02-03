use crate::core::geometry::BBox;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct GlyphRun {
    pub text: String,
    pub bbox: BBox,
}

pub fn extract_glyph_runs(pdf_path: &Path, page_idx: usize) -> Vec<GlyphRun> {
    // Use pdftotext (from poppler-utils) to extract plain text for a single page.
    // This is a coarse approximation: we treat all text on the page as one run
    // and assign it a page-wide bounding box.

    let page_number = page_idx + 1; // pdftotext is 1-based
    let output = Command::new("pdftotext")
        .arg("-f")
        .arg(page_number.to_string())
        .arg("-l")
        .arg(page_number.to_string())
        .arg(pdf_path)
        .arg("-") // write to stdout
        .output();

    let output = match output {
        Ok(out) => out,
        Err(err) => {
            eprintln!("failed to invoke pdftotext: {err}");
            return Vec::new();
        }
    };

    if !output.status.success() {
        eprintln!("pdftotext exited with status: {}", output.status);
        return Vec::new();
    }

    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if text.is_empty() {
        return Vec::new();
    }

    // For now, approximate the page as a fixed-size box; this matches the
    // default dimensions used elsewhere in the parser pipeline.
    let bbox = BBox::new(0.0, 0.0, 1000.0, 1400.0);

    vec![GlyphRun { text, bbox }]
}
