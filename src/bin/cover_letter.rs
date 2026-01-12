//! Cover Letter PDF Generator
//!
//! A local CLI tool to generate cover letter PDFs that match the CV styling.
//!
//! Usage:
//!     cargo run --bin cover-letter -- letter.md
//!     cargo run --bin cover-letter -- letter.md --data /path/to/cv_data.json
//!
//! Just write your letter as plain text. Personal info and styling come from
//! the CV data and site config (same sources as the main CV generator).

use anyhow::{Context, Result};
use cv_generator::{
    cover_letter::CoverLetter, cv_data::Cv, site_config::SiteConfig,
    typst_generator::generate_cover_letter_pdf, unified_config::AppConfig,
};
use std::env;
use std::path::PathBuf;
use tracing::{info, warn};

struct CliArgs {
    input_path: PathBuf,
    data_path: Option<PathBuf>,
}

fn parse_args() -> Result<CliArgs> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: cover-letter <letter.md> [--data <cv_data.json>]");
        eprintln!();
        eprintln!("Generate a cover letter PDF from a text/markdown file.");
        eprintln!("The letter content is used as-is - just write your letter.");
        eprintln!();
        eprintln!("Options:");
        eprintln!("  --data <path>  Path to cv_data.json (default: data/cv_data.json)");
        eprintln!();
        eprintln!("Personal info and styling come from the same sources as the CV:");
        eprintln!("  - CV data: data/cv_data.json (or --data flag or CV__DATA_PATH env var)");
        eprintln!("  - Site config: config/site.json");
        std::process::exit(1);
    }

    let mut input_path: Option<PathBuf> = None;
    let mut data_path: Option<PathBuf> = None;
    let mut i = 1;

    while i < args.len() {
        if args[i] == "--data" {
            if i + 1 >= args.len() {
                anyhow::bail!("--data requires a path argument");
            }
            data_path = Some(PathBuf::from(&args[i + 1]));
            i += 2;
        } else if args[i].starts_with('-') {
            anyhow::bail!("Unknown option: {}", args[i]);
        } else {
            if input_path.is_some() {
                anyhow::bail!("Multiple input files specified");
            }
            input_path = Some(PathBuf::from(&args[i]));
            i += 1;
        }
    }

    let input_path = input_path.ok_or_else(|| anyhow::anyhow!("No input file specified"))?;

    Ok(CliArgs {
        input_path,
        data_path,
    })
}

fn init_logging() {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_thread_ids(false)
        .with_level(true)
        .with_ansi(true)
        .init();
}

fn main() -> Result<()> {
    init_logging();

    let cli = parse_args()?;

    if !cli.input_path.exists() {
        anyhow::bail!("Input file not found: {}", cli.input_path.display());
    }

    // Load application configuration (same as main CV generator)
    let config = AppConfig::load().context("Failed to load application configuration")?;

    // Use --data flag if provided, otherwise fall back to config
    let data_path = cli.data_path.unwrap_or_else(|| config.data_path.clone());

    info!("Loading cover letter from: {}", cli.input_path.display());
    let letter = CoverLetter::from_file(&cli.input_path)
        .with_context(|| format!("Failed to read cover letter: {}", cli.input_path.display()))?;

    // Load personal info from CV data
    info!("Loading CV data from: {}", data_path.display());
    let cv = Cv::from_json(&data_path.to_string_lossy()).with_context(|| {
        format!(
            "Failed to load CV data from: {}\n\
             Specify path with --data flag or CV__DATA_PATH environment variable.",
            data_path.display()
        )
    })?;

    info!("Loaded personal info for: {}", cv.personal_info.name);

    // Load site config for Typst styling
    info!("Loading site configuration");
    let site_config = SiteConfig::from_json("config/site.json").unwrap_or_else(|e| {
        warn!("Failed to load site config: {}. Using defaults.", e);
        SiteConfig::default()
    });

    let typst_config = site_config
        .get_typst_config()
        .context("Failed to get Typst configuration")?;

    // Generate output path next to input file
    let output_path = cli.input_path.with_extension("pdf");
    let temp_path = config.typst_temp.to_string_lossy().to_string();

    info!("Generating PDF: {}", output_path.display());
    generate_cover_letter_pdf(
        &letter,
        &cv.personal_info,
        &typst_config,
        &temp_path,
        output_path.to_str().unwrap(),
    )
    .context("Failed to generate cover letter PDF")?;

    info!("Done! Cover letter saved to: {}", output_path.display());

    Ok(())
}
