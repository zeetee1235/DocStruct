use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::Command;

use crate::core::geometry::BBox;
use crate::core::model::{Block, Line, PageHypothesis, Provenance, Span};
use crate::parser::ParserTrack;

#[derive(Debug, Clone)]
pub struct DocxParser {
    text: String,
}

impl DocxParser {
    pub fn new(path: PathBuf) -> Result<Self> {
        let script = r#"
import sys, zipfile, xml.etree.ElementTree as ET
path = sys.argv[1]
with zipfile.ZipFile(path) as z:
    data = z.read('word/document.xml')
root = ET.fromstring(data)
ns = {'w':'http://schemas.openxmlformats.org/wordprocessingml/2006/main'}
paras = []
for p in root.findall('.//w:p', ns):
    chunks = [t.text for t in p.findall('.//w:t', ns) if t.text]
    txt = ''.join(chunks).strip()
    if txt:
        paras.append(txt)
print('\n'.join(paras))
"#;
        let output = Command::new("python3")
            .arg("-c")
            .arg(script)
            .arg(&path)
            .output()
            .with_context(|| "failed to invoke python3 for DOCX parsing")?;

        if !output.status.success() {
            anyhow::bail!(
                "DOCX parsing failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        Ok(Self {
            text: String::from_utf8_lossy(&output.stdout).trim().to_string(),
        })
    }
}

impl ParserTrack for DocxParser {
    fn page_count(&self) -> Result<usize> {
        Ok(1)
    }

    fn analyze_page(&self, page_idx: usize) -> Result<PageHypothesis> {
        let blocks = if page_idx == 0 && !self.text.is_empty() {
            let bbox = BBox::new(0.0, 0.0, 1000.0, 1400.0);
            vec![Block::TextBlock {
                bbox,
                lines: vec![Line {
                    spans: vec![Span {
                        text: self.text.clone(),
                        bbox,
                        source: Provenance::Parser,
                        style: None,
                    }],
                }],
                confidence: 0.6,
                source: Provenance::Parser,
                debug: None,
            }]
        } else {
            vec![]
        };

        Ok(PageHypothesis {
            page_idx,
            blocks,
            width: 1000,
            height: 1400,
        })
    }

    fn supports_ocr_rendering(&self) -> bool {
        false
    }

    fn rendering_source_path(&self) -> Option<&std::path::Path> {
        None
    }
}
