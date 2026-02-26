use crate::core::geometry::BBox;
use crate::parser::hangul::combine_hangul;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct GlyphRun {
    pub text: String,
    pub bbox: BBox,
}

#[derive(Debug, Clone, Copy)]
enum PdfToTextMode {
    Default,
    Raw,
    Layout,
}

fn run_pdftotext(pdf_path: &Path, page_number: usize, mode: PdfToTextMode) -> Option<String> {
    let mut cmd = Command::new("pdftotext");
    cmd.arg("-f")
        .arg(page_number.to_string())
        .arg("-l")
        .arg(page_number.to_string())
        .arg("-nopgbrk");

    match mode {
        PdfToTextMode::Default => {}
        PdfToTextMode::Raw => {
            cmd.arg("-raw");
        }
        PdfToTextMode::Layout => {
            cmd.arg("-layout");
        }
    }

    let output = cmd.arg(pdf_path).arg("-").output().ok()?;
    if !output.status.success() {
        return None;
    }

    let text = String::from_utf8_lossy(&output.stdout)
        .replace('\u{000C}', "")
        .trim()
        .to_string();
    if text.is_empty() {
        return None;
    }

    Some(text)
}

fn hangul_quality_score(text: &str) -> i64 {
    let mut syllables = 0_i64;
    let mut jamos = 0_i64;

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

    (syllables * 3) - (jamos * 2)
}

fn korean_counts(text: &str) -> (i64, i64) {
    let mut syllables = 0_i64;
    let mut jamos = 0_i64;

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

fn has_korean_chars(text: &str) -> bool {
    text.chars().any(|c| {
        let code = c as u32;
        (0xAC00..=0xD7A3).contains(&code)
            || (0x1100..=0x11FF).contains(&code)
            || (0x3130..=0x318F).contains(&code)
            || (0xA960..=0xA97F).contains(&code)
            || (0xD7B0..=0xD7FF).contains(&code)
    })
}

fn is_degraded_korean_text(text: &str) -> bool {
    let (syllables, jamos) = korean_counts(text);
    let total = syllables + jamos;
    total >= 8 && jamos > syllables * 2
}

fn clean_decomposed_korean_lines(text: &str) -> String {
    let mut kept = Vec::new();

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            kept.push(String::new());
            continue;
        }

        let (syllables, jamos) = korean_counts(trimmed);
        let has_korean = syllables + jamos > 0;
        let mostly_jamo = jamos >= 2 && syllables == 0;
        let heavy_jamo_noise = has_korean && jamos >= syllables * 3 && jamos >= 4;

        if mostly_jamo || heavy_jamo_noise {
            continue;
        }

        kept.push(trimmed.to_string());
    }

    // Collapse long blank runs to at most one empty line.
    let mut out = String::new();
    let mut prev_blank = false;
    for line in kept {
        let is_blank = line.is_empty();
        if is_blank && prev_blank {
            continue;
        }
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str(&line);
        prev_blank = is_blank;
    }
    out
}

pub fn extract_glyph_runs(pdf_path: &Path, page_idx: usize) -> Vec<GlyphRun> {
    // Use pdftotext (from poppler-utils) to extract plain text for a single page.
    // This is a coarse approximation: we treat all text on the page as one run
    // and assign it a page-wide bounding box.

    let page_number = page_idx + 1; // pdftotext is 1-based
    let modes = [
        PdfToTextMode::Raw,
        PdfToTextMode::Default,
        PdfToTextMode::Layout,
    ];
    let best_text = modes
        .iter()
        .filter_map(|mode| run_pdftotext(pdf_path, page_number, *mode))
        .map(|raw| {
            // Combine separated Hangul jamos into complete syllables while preserving normal spacing.
            let combined = combine_hangul(&raw);
            let score = hangul_quality_score(&combined);
            (score, combined)
        })
        .max_by_key(|(score, _)| *score)
        .map(|(_, text)| text);

    let text = match best_text {
        Some(text) => text,
        None => {
            eprintln!(
                "failed to extract text via pdftotext for page {}",
                page_number
            );
            return Vec::new();
        }
    };

    if text.is_empty() {
        return Vec::new();
    }

    let text = clean_decomposed_korean_lines(&text);
    if text.is_empty() {
        return Vec::new();
    }

    // If Korean text remains heavily decomposed after normalization/composition,
    // parser output is likely unreliable for this page. Let OCR dominate instead.
    if has_korean_chars(&text)
        && (hangul_quality_score(&text) < -10 || is_degraded_korean_text(&text))
    {
        eprintln!(
            "parser text quality is too low for Korean on page {}. falling back to OCR track",
            page_number
        );
        return Vec::new();
    }

    // For now, approximate the page as a fixed-size box; this matches the
    // default dimensions used elsewhere in the parser pipeline.
    let bbox = BBox::new(0.0, 0.0, 1000.0, 1400.0);

    vec![GlyphRun { text, bbox }]
}
