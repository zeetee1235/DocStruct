use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::core::geometry::BBox;
use crate::core::model::{Block, Line, PageHypothesis, Provenance, Span};
use crate::parser::ParserTrack;

#[derive(Debug, Clone, Deserialize)]
struct SlideRun {
    text: String,
    bbox: [f32; 4],
}

#[derive(Debug)]
pub struct PptxParser {
    slides: Vec<Vec<SlideRun>>,
    render_pdf_path: PathBuf,
    _temp_dir: Option<PathBuf>,
}

impl PptxParser {
    pub fn new(path: PathBuf) -> Result<Self> {
        Self::from_path(path, None)
    }

    pub fn from_ppt(path: PathBuf) -> Result<Self> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
        let temp_dir = std::env::temp_dir().join(format!("docstruct-ppt-{now}"));
        fs::create_dir_all(&temp_dir)?;

        let status = Command::new("soffice")
            .arg("--headless")
            .arg("--convert-to")
            .arg("pptx")
            .arg(&path)
            .arg("--outdir")
            .arg(&temp_dir)
            .status()
            .with_context(|| "failed to invoke soffice for PPT conversion")?;

        if !status.success() {
            anyhow::bail!("soffice failed to convert PPT to PPTX with status: {status}");
        }

        let stem = path
            .file_stem()
            .ok_or_else(|| anyhow::anyhow!("invalid PPT file name"))?
            .to_string_lossy();
        let converted = temp_dir.join(format!("{stem}.pptx"));
        if !converted.exists() {
            anyhow::bail!("converted PPTX file not found: {}", converted.display());
        }

        Self::from_path(converted, Some(temp_dir))
    }

    fn from_path(path: PathBuf, temp_dir: Option<PathBuf>) -> Result<Self> {
        let working_dir = if let Some(dir) = temp_dir {
            dir
        } else {
            let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
            let dir = std::env::temp_dir().join(format!("docstruct-pptx-{now}"));
            fs::create_dir_all(&dir)?;
            dir
        };

        let pdf_status = Command::new("soffice")
            .arg("--headless")
            .arg("--convert-to")
            .arg("pdf")
            .arg(&path)
            .arg("--outdir")
            .arg(&working_dir)
            .status()
            .with_context(|| "failed to invoke soffice for PPTX->PDF conversion")?;

        if !pdf_status.success() {
            anyhow::bail!("soffice failed to convert PPTX to PDF with status: {pdf_status}");
        }

        let stem = path
            .file_stem()
            .ok_or_else(|| anyhow::anyhow!("invalid PPTX file name"))?
            .to_string_lossy();
        let render_pdf_path = working_dir.join(format!("{stem}.pdf"));
        if !render_pdf_path.exists() {
            anyhow::bail!(
                "converted PPTX PDF file not found: {}",
                render_pdf_path.display()
            );
        }

        let script = r#"
import json, re, sys, zipfile, xml.etree.ElementTree as ET
path = sys.argv[1]
ns = {
 'p':'http://schemas.openxmlformats.org/presentationml/2006/main',
 'a':'http://schemas.openxmlformats.org/drawingml/2006/main',
}
with zipfile.ZipFile(path) as z:
    pres = ET.fromstring(z.read('ppt/presentation.xml'))
    sld = pres.find('.//p:sldSz', ns)
    sw = float(sld.attrib.get('cx', '9144000')) if sld is not None else 9144000.0
    sh = float(sld.attrib.get('cy', '6858000')) if sld is not None else 6858000.0
    sx = 1000.0 / max(sw, 1.0)
    sy = 1400.0 / max(sh, 1.0)
    slides = sorted([n for n in z.namelist() if n.startswith('ppt/slides/slide') and n.endswith('.xml')], key=lambda n:int(re.search(r'slide(\d+)\.xml$', n).group(1)))
    out=[]
    for sn in slides:
        root = ET.fromstring(z.read(sn))
        runs=[]
        for sp in root.findall('.//p:sp', ns):
            text = ' '.join([t.text for t in sp.findall('.//a:t', ns) if t.text]).strip()
            if not text:
                continue
            off = sp.find('.//a:off', ns)
            ext = sp.find('.//a:ext', ns)
            x = float(off.attrib.get('x', '0')) if off is not None else 0.0
            y = float(off.attrib.get('y', '0')) if off is not None else 0.0
            w = float(ext.attrib.get('cx', str(sw))) if ext is not None else sw
            h = float(ext.attrib.get('cy', str(sh))) if ext is not None else sh
            runs.append({'text':text, 'bbox':[x*sx, y*sy, (x+w)*sx, (y+h)*sy]})
        out.append(runs)
print(json.dumps(out, ensure_ascii=False))
"#;

        let output = Command::new("python3")
            .arg("-c")
            .arg(script)
            .arg(&path)
            .output()
            .with_context(|| "failed to invoke python3 for PPTX parsing")?;

        if !output.status.success() {
            anyhow::bail!(
                "PPTX parsing failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        let slides: Vec<Vec<SlideRun>> =
            serde_json::from_slice(&output.stdout).context("invalid PPTX parser output")?;

        Ok(Self {
            slides,
            render_pdf_path,
            _temp_dir: Some(working_dir),
        })
    }
}

impl ParserTrack for PptxParser {
    fn page_count(&self) -> Result<usize> {
        Ok(self.slides.len())
    }

    fn analyze_page(&self, page_idx: usize) -> Result<PageHypothesis> {
        let runs = self.slides.get(page_idx).cloned().unwrap_or_default();
        let mut blocks = Vec::new();

        for run in runs {
            let bbox = BBox::new(run.bbox[0], run.bbox[1], run.bbox[2], run.bbox[3]);
            blocks.push(Block::TextBlock {
                bbox,
                lines: vec![Line {
                    spans: vec![Span {
                        text: run.text,
                        bbox,
                        source: Provenance::Parser,
                        style: None,
                    }],
                }],
                confidence: 0.6,
                source: Provenance::Parser,
                debug: None,
            });
        }

        Ok(PageHypothesis {
            page_idx,
            blocks,
            width: 1000,
            height: 1400,
        })
    }

    fn supports_ocr_rendering(&self) -> bool {
        true
    }

    fn rendering_source_path(&self) -> Option<&Path> {
        Some(&self.render_pdf_path)
    }
}
