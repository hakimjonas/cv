mod config;
mod cv_data;
mod git_config;
mod github;
mod html_generator;
mod typst_generator;

use anyhow::{Context, Result};
use im::Vector;
use std::env;

// Extension trait to enable method chaining with pipe
trait Pipe: Sized {
    fn pipe<F, R>(self, f: F) -> R
    where
        F: FnOnce(Self) -> R,
    {
        f(self)
    }
}

// Implement Pipe for Config to enable method chaining
impl Pipe for config::Config {}

/// Prints a message about missing GitHub token and how to set it
fn print_token_missing_message() {
    println!("No GitHub API token found. API requests will be subject to lower rate limits.");
    println!(
        "To avoid rate limiting, set the token in git config with: git config --global {} <your-token>",
        git_config::GITHUB_TOKEN_KEY
    );
    println!("Or set the GITHUB_TOKEN environment variable.");
}

/// Attempts to get a GitHub token from environment variables
///
/// # Returns
///
/// An Option containing the token if found
fn get_token_from_env() -> Option<String> {
    env::var("GITHUB_TOKEN").ok().inspect(|_token| {
        println!("Using GitHub API token from environment variable for authentication");
    })
}

/// Attempts to get a GitHub token from git config
///
/// # Returns
///
/// A Result containing an Option with the token if found
fn get_token_from_git_config() -> Result<Option<String>> {
    git_config::read_github_token().map(|token_opt| {
        token_opt.inspect(|_token| {
            println!("Using GitHub API token from git config for authentication");
        })
    })
}

/// Gets a GitHub token from available sources with priority:
/// 1. GitHub Actions environment (if running in GitHub Actions)
/// 2. Git config
/// 3. Environment variable
///
/// # Returns
///
/// A Config with the token set if found
fn get_github_token(config: config::Config) -> config::Config {
    // Check if running in GitHub Actions
    match env::var("GITHUB_ACTIONS") {
        Ok(actions) if actions == "true" => {
            // We're running in GitHub Actions, check for GITHUB_TOKEN
            match env::var("GITHUB_TOKEN") {
                Ok(token) => {
                    println!("Using GitHub API token from GitHub Actions for authentication");
                    config.with_option(config::GITHUB_TOKEN_KEY, &token)
                }
                Err(_) => {
                    println!("No GitHub API token found in GitHub Actions environment.");
                    config
                }
            }
        }
        _ => {
            // Not running in GitHub Actions, check git config
            match get_token_from_git_config() {
                Ok(Some(token)) => config.with_option(config::GITHUB_TOKEN_KEY, &token),
                Ok(None) => {
                    // Token not found in git config, try environment variable
                    match get_token_from_env() {
                        Some(token) => config.with_option(config::GITHUB_TOKEN_KEY, &token),
                        None => {
                            print_token_missing_message();
                            config
                        }
                    }
                }
                Err(e) => {
                    println!(
                        "Warning: Failed to read GitHub token from git config: {}",
                        e
                    );
                    // Fall back to environment variable
                    match get_token_from_env() {
                        Some(token) => config.with_option(config::GITHUB_TOKEN_KEY, &token),
                        None => {
                            print_token_missing_message();
                            config
                        }
                    }
                }
            }
        }
    }
}

/// Main entry point for the CV generator application
///
/// This function initializes the configuration, loads the CV data,
/// fetches GitHub projects, and generates both HTML and PDF versions of the CV.
///
/// Command-line arguments:
/// - `--set-token <token>`: Set the GitHub API token in git config
/// - `--remove-token`: Remove the GitHub API token from git config
/// - `--cache-path <path>`: Set a custom path for the GitHub cache file
fn main() -> Result<()> {
    // Parse command-line arguments
    let args: Vector<String> = env::args().collect();

    // Check for --set-token argument
    if args.len() >= 3 && args[1] == "--set-token" {
        let token = &args[2];
        println!("Setting GitHub API token in git config...");
        return match git_config::write_github_token(token) {
            Ok(_) => {
                println!("GitHub API token set successfully.");
                Ok(())
            }
            Err(e) => {
                println!("Error setting GitHub API token: {}", e);
                Err(e)
            }
        };
    }

    // Check for --remove-token argument
    if args.len() >= 2 && args[1] == "--remove-token" {
        println!("Removing GitHub API token from git config...");
        return match git_config::remove_github_token() {
            Ok(_) => {
                println!("GitHub API token removed successfully.");
                Ok(())
            }
            Err(e) => {
                println!("Error removing GitHub API token: {}", e);
                Err(e)
            }
        };
    }

    // Create configuration with custom cache path if specified
    let config = config::Config::default().pipe(|cfg| {
        // Process arguments to find --cache-path
        args.iter().enumerate().fold(cfg, |acc, (i, arg)| {
            if arg == "--cache-path" && i + 1 < args.len() {
                let cache_path = std::path::PathBuf::from(&args[i + 1]);
                println!("Using custom GitHub cache path: {}", cache_path.display());
                acc.with_option(config::GITHUB_CACHE_KEY, &args[i + 1])
            } else {
                acc
            }
        })
    });

    // Load CV data
    println!("Loading CV data from {}", config.data_path.display());
    let mut cv =
        cv_data::Cv::from_json(&config.data_path_str()?).context("Failed to load CV data")?;

    // Get GitHub token from available sources
    let config_with_token = get_github_token(config);

    // Fetch GitHub projects from sources defined in CV data
    println!("Fetching GitHub projects from sources defined in CV data");
    match github::fetch_projects_from_sources_sync(
        &cv.github_sources,
        config_with_token.github_token(),
    ) {
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
    html_generator::generate_html(&cv, &config_with_token.html_output_str()?)
        .context("Failed to generate HTML files")?;

    // Copy static assets (excluding index.html which we generate)
    println!("Copying static assets");
    html_generator::copy_static_assets_except(
        &config_with_token.static_dir_str()?,
        &config_with_token.output_dir_str()?,
        &["index.html"],
    )
    .context("Failed to copy static assets")?;

    // Generate PDF CV
    println!("Generating PDF CV");
    typst_generator::generate_pdf(
        &cv,
        &config_with_token.typst_temp_str()?,
        &config_with_token.pdf_output_str()?,
    )
    .context("Failed to generate PDF CV")?;

    // Print summary
    println!("Done! Output files:");
    println!("  - HTML CV: {}", config_with_token.html_output.display());
    println!("  - PDF CV: {}", config_with_token.pdf_output.display());
    println!(
        "  - Static assets: {}",
        config_with_token.output_dir.display()
    );

    Ok(())
}
