use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrToken {
    pub text: String,
    pub bbox: [f32; 4],
    #[serde(default = "default_confidence")]
    pub confidence: f32,
    #[serde(default = "default_block_type")]
    pub block_type: String,
    #[serde(default)]
    pub latex: Option<String>,
}

fn default_block_type() -> String {
    "text".to_string()
}

fn default_confidence() -> f32 {
    0.5
}

#[derive(Debug, Clone)]
pub struct OcrBridge {
    work_dir: PathBuf,
    script_path: PathBuf,
    lang: String,
}

impl OcrBridge {
    pub fn new(work_dir: PathBuf) -> Self {
        let script_path = PathBuf::from("ocr/bridge/ocr_bridge.py");
        Self {
            work_dir,
            script_path,
            lang: "eng+kor".to_string(),
        }
    }

    pub fn with_script(mut self, script_path: PathBuf) -> Self {
        self.script_path = script_path;
        self
    }

    pub fn with_lang(mut self, lang: String) -> Self {
        self.lang = lang;
        self
    }

    pub fn run(&self, image_path: &Path) -> Result<Vec<OcrToken>> {
        fs::create_dir_all(&self.work_dir)?;
        let python_cmd = env::var("DOCSTRUCT_PYTHON").unwrap_or_else(|_| "python3".to_string());
        let output = Command::new(&python_cmd)
            .arg(&self.script_path)
            .arg("--image")
            .arg(image_path)
            .arg("--lang")
            .arg(&self.lang)
            .output()
            .with_context(|| format!("failed to invoke python OCR bridge using `{python_cmd}`"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("OCR bridge failed: {stderr}");
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let json_payload = extract_json_payload(&stdout).ok_or_else(|| {
            anyhow::anyhow!(
                "failed to find OCR JSON payload in stdout. raw stdout:\n{}",
                stdout.trim()
            )
        })?;
        let tokens: Vec<OcrToken> = serde_json::from_str(json_payload)
            .with_context(|| format!("failed to parse OCR JSON response. raw stdout:\n{}", stdout.trim()))?;
        Ok(tokens)
    }
}

fn extract_json_payload(stdout: &str) -> Option<&str> {
    let trimmed = stdout.trim();
    if trimmed.starts_with('[') || trimmed.starts_with('{') {
        return Some(trimmed);
    }

    stdout
        .lines()
        .rev()
        .map(str::trim)
        .find(|line| line.starts_with('[') || line.starts_with('{'))
}
