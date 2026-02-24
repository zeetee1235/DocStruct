use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};

use docstruct::pipeline::{build_document, export_document, PipelineConfig};

#[derive(Parser, Debug)]
#[command(name = "docstruct")]
#[command(version, about = "PDF document structure recovery using parser-OCR cross-validation", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Convert a PDF file to structured format
    Convert {
        /// Input PDF file path
        input: PathBuf,

        /// Output directory (default: ./<input_name>_output)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Output format(s) to generate
        #[arg(short, long, value_enum, default_values_t = vec![Format::Markdown, Format::Json])]
        format: Vec<Format>,

        /// Rendering DPI for OCR track
        #[arg(long, default_value_t = 200)]
        dpi: u32,

        /// Enable debug outputs (HTML viewer, rendered images)
        #[arg(short, long)]
        debug: bool,

        /// Disable progress bar
        #[arg(short, long)]
        quiet: bool,
    },

    /// Convert multiple PDF files
    Batch {
        /// Input PDF files or glob pattern
        inputs: Vec<PathBuf>,

        /// Output directory for all results
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Output format(s) to generate
        #[arg(short, long, value_enum, default_values_t = vec![Format::Markdown, Format::Json])]
        format: Vec<Format>,

        /// Rendering DPI for OCR track
        #[arg(long, default_value_t = 200)]
        dpi: u32,

        /// Enable debug outputs
        #[arg(short, long)]
        debug: bool,
    },

    /// Show information about a PDF file
    Info {
        /// Input PDF file path
        input: PathBuf,
    },
}

#[derive(ValueEnum, Clone, Debug, PartialEq)]
enum Format {
    Json,
    Markdown,
    Text,
    Html,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Convert {
            input,
            output,
            format,
            dpi,
            debug,
            quiet,
        } => convert_single(input, output, format, dpi, debug, quiet),
        Commands::Batch {
            inputs,
            output,
            format,
            dpi,
            debug,
        } => convert_batch(inputs, output, format, dpi, debug),
        Commands::Info { input } => show_info(input),
    }
}

fn convert_single(
    input: PathBuf,
    output: Option<PathBuf>,
    _formats: Vec<Format>,
    dpi: u32,
    _debug: bool,
    quiet: bool,
) -> Result<()> {
    // Validate input
    if !input.exists() {
        anyhow::bail!("Input file does not exist: {}", input.display());
    }
    if !input.is_file() {
        anyhow::bail!("Input is not a file: {}", input.display());
    }

    // Determine output directory
    let output_dir = output.unwrap_or_else(|| {
        let stem = input.file_stem().unwrap().to_string_lossy();
        PathBuf::from(format!("{}_output", stem))
    });

    if !quiet {
        println!("[*] Processing: {}", input.display());
        println!("[*] Output: {}", output_dir.display());
        println!("[*] DPI: {}", dpi);
    }

    let config = PipelineConfig::new(input.clone(), output_dir.clone(), dpi);

    if !quiet {
        println!("\n[+] Building document...");
    }

    let document = build_document(&config)
        .with_context(|| format!("Failed to process PDF: {}", input.display()))?;

    if !quiet {
        println!("[+] Exporting results...");
    }

    export_document(&document, &config.output)
        .with_context(|| format!("Failed to export to: {}", output_dir.display()))?;

    if !quiet {
        println!("\n[✓] Done! Results saved to: {}", output_dir.display());
    }

    Ok(())
}

fn convert_batch(
    inputs: Vec<PathBuf>,
    output: Option<PathBuf>,
    formats: Vec<Format>,
    dpi: u32,
    debug: bool,
) -> Result<()> {
    if inputs.is_empty() {
        anyhow::bail!("No input files specified");
    }

    let base_output = output.unwrap_or_else(|| PathBuf::from("batch_output"));

    println!("[*] Batch processing {} file(s)", inputs.len());
    println!("[*] Base output: {}\n", base_output.display());

    let mut success = 0;
    let mut failed = 0;

    for (i, input) in inputs.iter().enumerate() {
        println!("[{}/{}] Processing: {}", i + 1, inputs.len(), input.display());

        if !input.exists() {
            eprintln!("  [!] Skipped: file does not exist");
            failed += 1;
            continue;
        }

        let stem = input.file_stem().unwrap().to_string_lossy();
        let output_dir = base_output.join(&*stem);

        match convert_single(input.clone(), Some(output_dir), formats.clone(), dpi, debug, true) {
            Ok(_) => {
                println!("  [✓] Success");
                success += 1;
            }
            Err(e) => {
                eprintln!("  [✗] Failed: {}", e);
                failed += 1;
            }
        }
        println!();
    }

    println!("\n[*] Summary: {} succeeded, {} failed", success, failed);

    if failed > 0 {
        anyhow::bail!("{} file(s) failed to process", failed);
    }

    Ok(())
}

fn show_info(input: PathBuf) -> Result<()> {
    use docstruct::parser::pdf_reader::PdfReader;

    if !input.exists() {
        anyhow::bail!("Input file does not exist: {}", input.display());
    }

    let reader = PdfReader::new(input.clone())
        .with_context(|| format!("Failed to open PDF: {}", input.display()))?;

    let page_count = reader.page_count()?;

    println!("PDF Information");
    println!("===============");
    println!("File: {}", input.display());
    println!("Pages: {}", page_count);

    Ok(())
}
