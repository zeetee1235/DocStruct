use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

use docstruct::pipeline::{build_document, export_document, PipelineConfig};

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
    let config = PipelineConfig::new(cli.input, cli.out, cli.dpi);

    let document = build_document(&config)?;
    export_document(&document, &config.output)?;

    Ok(())
}
