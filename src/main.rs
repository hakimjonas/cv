mod config;
mod cv_data;
mod github;
mod html_generator;
mod typst_generator;

use anyhow::{Context, Result};
use im::Vector;

/// Main entry point for the CV generator application
///
/// This function initializes the configuration, loads the CV data,
/// fetches GitHub projects, and generates both HTML and PDF versions of the CV.
fn main() -> Result<()> {
    // Create default configuration
    let config = config::Config::default();

    // Load CV data
    println!("Loading CV data from {}", config.data_path.display());
    let mut cv =
        cv_data::Cv::from_json(&config.data_path_str()?).context("Failed to load CV data")?;

    // Fetch GitHub projects from both personal account and fungal-lang organization
    println!("Fetching GitHub projects for hakimjonas and fungal-lang organization");
    match github::fetch_all_github_projects_sync("hakimjonas", "fungal-lang") {
        Ok(github_projects) => {
            println!("Found {} GitHub projects", github_projects.len());

            // Keep any existing projects that are not from GitHub (identified by not having a repository URL)
            let local_projects = cv
                .projects
                .iter()
                .filter(|p| p.repository.is_none())
                .cloned()
                .collect::<Vector<_>>();

            // Combine local projects with GitHub projects
            cv.projects = local_projects
                .iter()
                .chain(github_projects.iter())
                .cloned()
                .collect();

            println!("Updated CV with {} total projects", cv.projects.len());
        }
        Err(e) => {
            println!("Warning: Failed to fetch GitHub projects: {}", e);
            println!("Continuing with existing projects data");
        }
    }

    // Generate HTML CV and index
    println!("Generating HTML files");
    html_generator::generate_html(&cv, &config.html_output_str()?)
        .context("Failed to generate HTML files")?;

    // Copy static assets (excluding index.html which we generate)
    println!("Copying static assets");
    html_generator::copy_static_assets_except(
        &config.static_dir_str()?,
        &config.output_dir_str()?,
        &["index.html"],
    )
    .context("Failed to copy static assets")?;

    // Generate PDF CV
    println!("Generating PDF CV");
    typst_generator::generate_pdf(&cv, &config.typst_temp_str()?, &config.pdf_output_str()?)
        .context("Failed to generate PDF CV")?;

    // Print summary
    println!("Done! Output files:");
    println!("  - HTML CV: {}", config.html_output.display());
    println!("  - PDF CV: {}", config.pdf_output.display());
    println!("  - Static assets: {}", config.output_dir.display());

    Ok(())
}
