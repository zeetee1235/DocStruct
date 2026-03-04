#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;
use std::{env, ffi::OsStr};

use docstruct::core::model::{Block, DocumentFinal, Provenance};
use docstruct::pipeline::{build_document, export_document, PipelineConfig};
use rfd::FileDialog;
use serde::Serialize;

#[derive(Serialize)]
struct ConversionItem {
    input_path: String,
    success: bool,
    output_dir: Option<String>,
    elapsed_ms: u128,
    text: String,
    error: Option<String>,
}

#[derive(Serialize)]
struct BatchConvertResult {
    items: Vec<ConversionItem>,
    success_count: usize,
    failed_count: usize,
    elapsed_ms: u128,
    combined_text: String,
}

#[tauri::command]
async fn pick_input_files() -> Result<Option<Vec<String>>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        Ok(FileDialog::new()
            .add_filter("Documents", &["pdf", "docx", "ppt", "pptx"])
            .pick_files()
            .map(|paths| {
                paths
                    .into_iter()
                    .map(|path| path.to_string_lossy().to_string())
                    .collect::<Vec<_>>()
            }))
    })
    .await
    .map_err(|e| format!("Failed to open input file dialog: {e}"))?
}

#[tauri::command]
async fn pick_output_dir() -> Result<Option<String>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        Ok(FileDialog::new()
            .pick_folder()
            .map(|path| path.to_string_lossy().to_string()))
    })
    .await
    .map_err(|e| format!("Failed to open output folder dialog: {e}"))?
}

#[tauri::command]
fn suggest_output_dir(input_path: String) -> Result<String, String> {
    let input = PathBuf::from(&input_path);
    let stem = input
        .file_stem()
        .ok_or_else(|| "Could not derive output directory from input filename".to_string())?
        .to_string_lossy()
        .to_string();

    let parent = input
        .parent()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    Ok(parent
        .join(format!("{}_output", stem))
        .to_string_lossy()
        .to_string())
}

fn ensure_command_available(command: &str) -> Result<(), String> {
    Command::new(command)
        .arg("--help")
        .output()
        .map(|_| ())
        .map_err(|e| format!("`{command}` is not available: {e}"))
}

fn get_python_command() -> String {
    env::var("DOCSTRUCT_PYTHON").unwrap_or_else(|_| "python3".to_string())
}

fn command_name_for_message(cmd: &str) -> &str {
    Path::new(cmd)
        .file_name()
        .and_then(OsStr::to_str)
        .unwrap_or(cmd)
}

fn ensure_python_modules() -> Result<(), String> {
    let python_cmd = get_python_command();
    let probe = Command::new(&python_cmd)
        .arg("-c")
        .arg("import cv2, pytesseract; from PIL import Image")
        .output()
        .map_err(|e| format!("Failed to run `{python_cmd}`: {e}"))?;

    if probe.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&probe.stderr).trim().to_string();
    if stderr.is_empty() {
        return Err("Python dependency check failed with empty stderr.".to_string());
    }

    Err(stderr)
}

fn check_runtime_dependencies() -> Result<(), String> {
    let mut missing = Vec::new();
    let python_cmd = get_python_command();

    let python_name = command_name_for_message(&python_cmd).to_string();
    if let Err(err) = ensure_command_available(&python_cmd) {
        missing.push(err);
    }

    for command in ["tesseract", "pdftotext", "pdftoppm", "pdfinfo"] {
        if let Err(err) = ensure_command_available(command) {
            missing.push(err);
        }
    }

    if !missing.is_empty() {
        return Err(format!(
            "Missing required system dependencies:\n- {}\n\nInstall the missing tools and try again.",
            missing.join("\n- ")
        ));
    }

    if let Err(err) = ensure_python_modules() {
        return Err(format!(
            "Python dependencies are missing or broken:\n{err}\n\nInstall with:\n{python_name} -m pip install -r requirements.txt"
        ));
    }

    Ok(())
}

fn should_skip_degraded_parser_text(source: Provenance, text: &str) -> bool {
    if source != Provenance::Parser {
        return false;
    }
    let (syllables, jamos) = korean_counts(text);
    let has_korean = syllables + jamos > 0;
    has_korean && jamos >= syllables * 2 && jamos >= 8
}

fn should_skip_noisy_ocr_text(source: Provenance, text: &str) -> bool {
    if source != Provenance::Ocr {
        return false;
    }
    let compact = text.split_whitespace().collect::<String>();
    if compact.chars().count() <= 2 {
        return true;
    }

    let alnum_or_hangul = compact
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || ('가'..='힣').contains(c))
        .count();
    let total = compact.chars().count();
    if total > 0 && alnum_or_hangul * 2 < total {
        return true;
    }

    let symbol_heavy = text
        .chars()
        .filter(|c| !c.is_alphanumeric() && !c.is_whitespace())
        .count();
    symbol_heavy >= 8 && symbol_heavy > alnum_or_hangul
}

fn korean_counts(text: &str) -> (usize, usize) {
    let mut syllables = 0usize;
    let mut jamos = 0usize;
    for c in text.chars() {
        let code = c as u32;
        if (0xAC00..=0xD7A3).contains(&code) {
            syllables += 1;
        } else if (0x1100..=0x11FF).contains(&code)
            || (0x3130..=0x318F).contains(&code)
            || (0xA960..=0xA97F).contains(&code)
            || (0xD7B0..=0xD7FF).contains(&code)
        {
            jamos += 1;
        }
    }
    (syllables, jamos)
}

fn format_block_text(block: &Block) -> String {
    match block {
        Block::TextBlock { lines, source, .. } => {
            let text = lines
                .iter()
                .map(|line| {
                    line.spans
                        .iter()
                        .map(|span| span.text.as_str())
                        .collect::<Vec<_>>()
                        .join("")
                })
                .collect::<Vec<_>>()
                .join("\n");

            if should_skip_degraded_parser_text(*source, &text)
                || should_skip_noisy_ocr_text(*source, &text)
            {
                String::new()
            } else {
                text
            }
        }
        Block::TableBlock { bbox, .. } => format!(
            "[TABLE at x:{:.0} y:{:.0} w:{:.0} h:{:.0}]",
            bbox.x0,
            bbox.y0,
            bbox.width(),
            bbox.height()
        ),
        Block::FigureBlock { bbox, .. } => format!(
            "[FIGURE at x:{:.0} y:{:.0} w:{:.0} h:{:.0}]",
            bbox.x0,
            bbox.y0,
            bbox.width(),
            bbox.height()
        ),
        Block::MathBlock { bbox, latex, .. } => {
            if let Some(latex) = latex {
                if !latex.trim().is_empty() {
                    return format!("[MATH] {latex}");
                }
            }
            format!(
                "[MATH at x:{:.0} y:{:.0} w:{:.0} h:{:.0}]",
                bbox.x0,
                bbox.y0,
                bbox.width(),
                bbox.height()
            )
        }
    }
}

fn document_to_text(document: &DocumentFinal) -> String {
    let mut out = String::new();
    for page in &document.pages {
        out.push_str(&format!("=== Page {} ===\n\n", page.page_idx + 1));
        for block in &page.blocks {
            let block_text = format_block_text(block);
            if !block_text.is_empty() {
                out.push_str(&block_text);
                out.push_str("\n\n");
            }
        }
        out.push('\n');
    }
    out
}

fn derive_output_dir(base: &Path, input: &Path, is_multi: bool) -> Result<PathBuf, String> {
    if !is_multi {
        return Ok(base.to_path_buf());
    }

    let stem = input
        .file_stem()
        .ok_or_else(|| format!("Could not derive output directory for {}", input.display()))?
        .to_string_lossy()
        .to_string();

    Ok(base.join(stem))
}

#[tauri::command]
async fn convert_documents(
    input_paths: Vec<String>,
    output_dir: Option<String>,
    dpi: u32,
) -> Result<BatchConvertResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        if input_paths.is_empty() {
            return Err("No input files selected.".to_string());
        }

        check_runtime_dependencies()?;

        let overall_started = Instant::now();
        let mut items = Vec::with_capacity(input_paths.len());
        let mut combined_text = String::new();
        let multi = input_paths.len() > 1;
        let base_output = output_dir.as_ref().map(PathBuf::from);

        for input_path in input_paths {
            let started = Instant::now();
            let input = PathBuf::from(&input_path);

            if !input.exists() {
                items.push(ConversionItem {
                    input_path,
                    success: false,
                    output_dir: None,
                    elapsed_ms: started.elapsed().as_millis(),
                    text: String::new(),
                    error: Some("Input file does not exist.".to_string()),
                });
                continue;
            }

            if !input.is_file() {
                items.push(ConversionItem {
                    input_path,
                    success: false,
                    output_dir: None,
                    elapsed_ms: started.elapsed().as_millis(),
                    text: String::new(),
                    error: Some("Input path is not a file.".to_string()),
                });
                continue;
            }

            let output_for_file = if let Some(base) = &base_output {
                Some(derive_output_dir(base, &input, multi)?)
            } else {
                None
            };

            let temp_output_for_processing = output_for_file
                .clone()
                .unwrap_or_else(|| std::env::temp_dir().join("docstruct-gui-preview"));

            let config = PipelineConfig::new(input.clone(), temp_output_for_processing, dpi);
            let build_result = build_document(&config);

            match build_result {
                Ok(document) => {
                    let text = document_to_text(&document);

                    if let Some(out) = &output_for_file {
                        if let Err(err) = export_document(&document, out) {
                            items.push(ConversionItem {
                                input_path,
                                success: false,
                                output_dir: Some(out.to_string_lossy().to_string()),
                                elapsed_ms: started.elapsed().as_millis(),
                                text,
                                error: Some(format!("Export failed: {err}")),
                            });
                            continue;
                        }
                    }

                    combined_text.push_str(&format!("===== {} =====\n\n", input.display()));
                    combined_text.push_str(&text);
                    combined_text.push('\n');

                    items.push(ConversionItem {
                        input_path,
                        success: true,
                        output_dir: output_for_file
                            .as_ref()
                            .map(|p| p.to_string_lossy().to_string()),
                        elapsed_ms: started.elapsed().as_millis(),
                        text,
                        error: None,
                    });
                }
                Err(err) => {
                    items.push(ConversionItem {
                        input_path,
                        success: false,
                        output_dir: output_for_file
                            .as_ref()
                            .map(|p| p.to_string_lossy().to_string()),
                        elapsed_ms: started.elapsed().as_millis(),
                        text: String::new(),
                        error: Some(format!("Build failed: {err}")),
                    });
                }
            }
        }

        let success_count = items.iter().filter(|item| item.success).count();
        let failed_count = items.len().saturating_sub(success_count);

        Ok(BatchConvertResult {
            items,
            success_count,
            failed_count,
            elapsed_ms: overall_started.elapsed().as_millis(),
            combined_text,
        })
    })
    .await
    .map_err(|e| format!("Task join failed: {e}"))?
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            pick_input_files,
            pick_output_dir,
            suggest_output_dir,
            convert_documents
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
