use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;

use docstruct::core::geometry::BBox;
use docstruct::core::model::{Block, DocumentFinal, Line, PageDebug, PageFinal, PageHypothesis, Span, Provenance};
use docstruct::export::{JsonExporter, HtmlDebugExporter, Exporter};
use docstruct::fusion::{SimpleFusionEngine, FusionEngine};
use docstruct::ocr::{OcrTrack, PageRenderer};
use docstruct::ocr::layout_builder::OcrLayoutBuilder;
use docstruct::ocr::bridge::OcrBridge;
use docstruct::parser::{ParserTrack, PdfReader};
use docstruct::parser::layout_builder::ParserLayoutBuilder;

/// Unit test: Verify basic fusion logic with synthetic data
#[test]
fn test_fusion_with_synthetic_data() -> Result<()> {
    // Build a minimal parser hypothesis with one text block
    let p_bbox = BBox::new(0.0, 0.0, 50.0, 50.0);
    let parser_block = Block::TextBlock {
        bbox: p_bbox,
        lines: vec![Line {
            spans: vec![Span {
                text: "Hello".to_string(),
                bbox: p_bbox,
                source: Provenance::Parser,
                style: None,
            }],
        }],
        confidence: 0.6,
        source: Provenance::Parser,
        debug: None,
    };

    let parser = PageHypothesis {
        page_idx: 0,
        blocks: vec![parser_block],
        width: 1000,
        height: 1400,
    };

    // Build a minimal OCR hypothesis that overlaps and has the same text
    let o_bbox = BBox::new(10.0, 10.0, 60.0, 60.0);
    let ocr_block = Block::TextBlock {
        bbox: o_bbox,
        lines: vec![Line {
            spans: vec![Span {
                text: "Hello".to_string(),
                bbox: o_bbox,
                source: Provenance::Ocr,
                style: None,
            }],
        }],
        confidence: 0.5,
        source: Provenance::Ocr,
        debug: None,
    };

    let ocr = PageHypothesis {
        page_idx: 0,
        blocks: vec![ocr_block],
        width: 1000,
        height: 1400,
    };

    // Fuse
    let fusion = SimpleFusionEngine::new();
    let fused = fusion.fuse(&parser, &ocr)?;

    // Basic sanity checks on fused page
    assert_eq!(fused.page_idx, 0);
    assert!(!fused.blocks.is_empty());
    let found_text = fused
        .blocks
        .iter()
        .filter_map(|b| b.text_content())
        .collect::<Vec<_>>()
        .join(" ");
    assert!(found_text.contains("Hello"));

    // Export to a temporary directory
    let mut out = std::env::temp_dir();
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
    let pid = std::process::id();
    out.push(format!("docstruct-test-{}-{}", pid, now));
    let exporter = JsonExporter::new(out.clone());
    exporter.export(&DocumentFinal { pages: vec![fused] })?;

    // Verify exported JSON contains the text
    let file = out.join("document.json");
    let contents = fs::read_to_string(&file)?;
    assert!(contents.contains("Hello"));

    // Cleanup
    let _ = fs::remove_file(&file);
    let _ = fs::remove_dir(&out);

    Ok(())
}

/// Integration test: Process real test_document.pdf with parser-only pipeline
#[test]
fn test_parser_pipeline_with_test_document() -> Result<()> {
    let test_pdf = PathBuf::from("test/test_document.pdf");
    
    // Skip if test PDF doesn't exist (CI environment)
    if !test_pdf.exists() {
        eprintln!("Skipping test: test/test_document.pdf not found");
        return Ok(());
    }

    let pdf_reader = PdfReader::new(test_pdf.clone())?;
    let page_count = pdf_reader.page_count();
    
    // Should have at least 1 page
    assert!(page_count > 0, "test_document.pdf should have at least one page");

    let parser_track = ParserLayoutBuilder::new();
    
    // Test first page parsing
    let page_hypo = parser_track.analyze_page(&test_pdf, 0)?;
    
    assert_eq!(page_hypo.page_idx, 0);
    assert!(page_hypo.width > 0);
    assert!(page_hypo.height > 0);
    
    // test_document.pdf should extract some text blocks
    assert!(!page_hypo.blocks.is_empty(), "Should extract at least one block from test_document.pdf");
    
    // Check that we can extract text content
    let text_blocks: Vec<_> = page_hypo.blocks.iter()
        .filter_map(|b| b.text_content())
        .collect();
    
    assert!(!text_blocks.is_empty(), "Should have text content in blocks");
    
    Ok(())
}

/// Integration test: Full pipeline with test_document.pdf (parser + OCR + fusion + export)
#[test]
#[ignore] // Ignored by default because it requires OCR setup
fn test_full_pipeline_with_test_document() -> Result<()> {
    let test_pdf = PathBuf::from("test/test_document.pdf");
    
    if !test_pdf.exists() {
        eprintln!("Skipping test: test/test_document.pdf not found");
        return Ok(());
    }

    // Setup output directory
    let mut out = std::env::temp_dir();
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
    out.push(format!("docstruct-integration-{}", now));
    fs::create_dir_all(&out)?;

    let pdf_reader = PdfReader::new(test_pdf.clone())?;
    let page_count = pdf_reader.page_count();

    let renderer = PageRenderer::new(out.join("debug"), 200);
    let parser_track = ParserLayoutBuilder::new();
    let bridge = OcrBridge::new(out.join("ocr"));
    let ocr_track = OcrLayoutBuilder::new(bridge);
    let fusion = SimpleFusionEngine::new();

    let mut pages: Vec<PageFinal> = Vec::new();

    // Process first page only for fast testing
    let page_idx = 0;
    let rendered = renderer.render_page(&test_pdf, page_idx)?;
    let parser_hypo = parser_track.analyze_page(&test_pdf, page_idx)?;
    let ocr_hypo = ocr_track.analyze_page(&rendered.path, page_idx)?;
    
    let mut fused = fusion.fuse(&parser_hypo, &ocr_hypo)?;
    fused.debug = Some(PageDebug {
        parser_blocks: parser_hypo.blocks.clone(),
        ocr_blocks: ocr_hypo.blocks.clone(),
    });
    pages.push(fused);

    let document = DocumentFinal { pages };
    
    // Export JSON
    let json_exporter = JsonExporter::new(out.clone());
    json_exporter.export(&document)?;

    // Export HTML debug
    let html_exporter = HtmlDebugExporter::new(out.join("debug"));
    html_exporter.export(&document)?;

    // Verify outputs exist
    assert!(out.join("document.json").exists(), "JSON output should exist");
    
    let json_content = fs::read_to_string(out.join("document.json"))?;
    assert!(json_content.contains("pages"), "JSON should contain pages array");
    assert!(json_content.contains("blocks"), "JSON should contain blocks");

    // Cleanup
    let _ = fs::remove_dir_all(&out);

    Ok(())
}

/// Integration test: Verify korean_test.pdf can be opened and has pages
#[test]
fn test_korean_pdf_opens() -> Result<()> {
    let korean_pdf = PathBuf::from("test/korean_test.pdf");
    
    if !korean_pdf.exists() {
        eprintln!("Skipping test: test/korean_test.pdf not found");
        return Ok(());
    }

    let pdf_reader = PdfReader::new(korean_pdf.clone())?;
    let page_count = pdf_reader.page_count();
    
    assert!(page_count > 0, "korean_test.pdf should have at least one page");

    let parser_track = ParserLayoutBuilder::new();
    let page_hypo = parser_track.analyze_page(&korean_pdf, 0)?;
    
    assert_eq!(page_hypo.page_idx, 0);
    
    Ok(())
}
