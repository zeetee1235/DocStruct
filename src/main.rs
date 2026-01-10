use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

use docstruct::core::model::{DocumentFinal, PageDebug, PageFinal};
use docstruct::export::{html_debug_export::HtmlDebugExporter, json_export::JsonExporter, Exporter};
use docstruct::fusion::{FusionEngine, SimpleFusionEngine};
use docstruct::ocr::{bridge::OcrBridge, layout_builder::OcrLayoutBuilder, renderer::PageRenderer, OcrTrack};
use docstruct::parser::{layout_builder::ParserLayoutBuilder, pdf_reader::PdfReader, ParserTrack};

#[derive(Parser, Debug)]
#[command(name = "docstruct")]
#[command(about = "Parser ↔ OCR cross-checking document structure reconstruction")]
struct Cli {
    input: PathBuf,
    #[arg(long)]
    out: PathBuf,
    #[arg(long, default_value_t = 200)]
    dpi: u32,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let pdf_reader = PdfReader::new(cli.input.clone())?;
    let page_count = pdf_reader.page_count();

    let renderer = PageRenderer::new(cli.out.join("debug"), cli.dpi);
    let parser_track = ParserLayoutBuilder::new();
    let bridge = OcrBridge::new(cli.out.join("ocr"));
    let ocr_track = OcrLayoutBuilder::new(bridge);
    let fusion = SimpleFusionEngine::new();

    let mut pages: Vec<PageFinal> = Vec::new();

    for page_idx in 0..page_count {
        let rendered = renderer.render_page(&cli.input, page_idx)?;
        let parser_hypo = parser_track.analyze_page(&cli.input, page_idx)?;
        let ocr_hypo = ocr_track.analyze_page(&rendered.path, page_idx)?;
        let mut fused = fusion.fuse(&parser_hypo, &ocr_hypo)?;
        fused.debug = Some(PageDebug {
            parser_blocks: parser_hypo.blocks.clone(),
            ocr_blocks: ocr_hypo.blocks.clone(),
        });
        pages.push(fused);
    }

    let document = DocumentFinal { pages };
    let json_exporter = JsonExporter::new(cli.out.clone());
    json_exporter.export(&document)?;

    let html_exporter = HtmlDebugExporter::new(cli.out.join("debug"));
    html_exporter.export(&document)?;

    Ok(())
}
