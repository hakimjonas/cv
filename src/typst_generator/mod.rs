/// Typst generator module for creating PDF CVs
///
/// This module provides functionality to generate Typst markup from CV data
/// and compile it to PDF using the Typst CLI.
mod markup;
mod sections;
mod utils;

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::cv_data::Cv;
use markup::generate_typst_markup;

/// Generate a PDF from CV data using Typst
///
/// # Arguments
///
/// * `cv` - The CV data to generate a PDF from
/// * `temp_path` - Path to a temporary file for Typst markup
/// * `output_path` - Path where the PDF will be written
///
/// # Returns
///
/// A Result indicating success or failure
pub fn generate_pdf(cv: &Cv, temp_path: &str, output_path: &str) -> Result<()> {
    // Generate Typst markup
    let typst_markup = generate_typst_markup(cv);

    // Ensure the output directory exists
    if let Some(parent) = Path::new(output_path).parent() {
        fs::create_dir_all(parent).context("Failed to create output directory")?;
    }

    // Write Typst markup to temporary file
    fs::write(temp_path, &typst_markup)
        .with_context(|| format!("Failed to write Typst markup to {temp_path}"))?;

    // Debug: Print template structure
    if cfg!(debug_assertions) {
        let lines: Vec<&str> = typst_markup.lines().collect();
        println!("Generated Typst template (first 20 lines):");
        for (i, line) in lines.iter().take(20).enumerate() {
            println!("{:2}: {}", i + 1, line);
        }
        println!("\n... (template continues) ...\n");
        println!("Generated Typst template (last 10 lines):");
        for (i, line) in lines.iter().rev().take(10).rev().enumerate() {
            println!("{:2}: {}", lines.len() - 9 + i, line);
        }
    }

    // Compile Typst to PDF
    let status = Command::new("typst")
        .arg("compile")
        .arg(temp_path)
        .arg(output_path)
        .status()
        .context("Failed to execute Typst CLI. Make sure it's installed and in your PATH")?;

    if !status.success() {
        return Err(anyhow::anyhow!(
            "Typst compilation failed with status: {}",
            status
        ));
    }

    // Optionally, clean up the temporary file
    fs::remove_file(temp_path)
        .with_context(|| format!("Failed to remove temporary file: {temp_path}"))?;

    Ok(())
}
