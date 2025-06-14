#[allow(dead_code)]
mod asset_processor;
mod config;
mod credentials;
mod cv_data;
mod db;
#[allow(dead_code)]
mod git_config;
mod github;
mod html_generator;
mod migrate;
#[allow(dead_code)]
mod runtime;
mod typst_generator;

use anyhow::{Context, Result};
use im::Vector;
use std::env;
use tracing::{info, warn, error, debug};
use cv::logging;

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

/// Initialize logging with tracing
fn init_logging() {
    // Set up a logging configuration for the CV application
    let config = logging::LoggingConfig {
        app_name: "cv".to_string(),
        level: tracing::Level::INFO,
        log_spans: true,
        ..Default::default()
    };

    // Initialize logging with the configuration
    let _guard = logging::init_logging(config);
    info!("Logging initialized with tracing");
}

/// Prints a message about missing GitHub token and how to set it
fn print_token_missing_message() {
    warn!("No GitHub API token found. API requests will be subject to lower rate limits.");
    info!("To avoid rate limiting, set the token with: cv --set-token <your-token>");
    info!("Or set the GITHUB_TOKEN environment variable.");
}

/// Attempts to get a GitHub token from environment variables
///
/// # Returns
///
/// An Option containing the token if found
fn get_token_from_env() -> Option<String> {
    env::var("GITHUB_TOKEN").ok().inspect(|_token| {
        info!("Using GitHub API token from environment variable for authentication");
    })
}

/// Attempts to get a GitHub token from secure storage
///
/// # Returns
///
/// A Result containing an Option with the token if found
fn get_token_from_secure_storage() -> Result<Option<String>> {
    credentials::get_github_token().map(|token_opt| {
        token_opt.inspect(|_token| {
            info!("Using GitHub API token from secure storage for authentication");
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
                    info!("Using GitHub API token from GitHub Actions for authentication");
                    config.with_option(config::GITHUB_TOKEN_KEY, &token)
                }
                Err(_) => {
                    warn!("No GitHub API token found in GitHub Actions environment.");
                    config
                }
            }
        }
        _ => {
            // Not running in GitHub Actions, check secure storage
            match get_token_from_secure_storage() {
                Ok(Some(token)) => config.with_option(config::GITHUB_TOKEN_KEY, &token),
                Ok(None) => {
                    // Token not found in secure storage, try environment variable
                    match get_token_from_env() {
                        Some(token) => config.with_option(config::GITHUB_TOKEN_KEY, &token),
                        None => {
                            print_token_missing_message();
                            config
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to read GitHub token from secure storage: {}", e);
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
    // Initialize logging
    init_logging();

    // Parse command-line arguments
    let args: Vector<String> = env::args().collect();

    // Check for --set-token argument
    if args.len() >= 3 && args[1] == "--set-token" {
        let token = &args[2];
        info!("Setting GitHub API token in secure storage...");
        return match credentials::store_github_token(token) {
            Ok(_) => {
                info!("GitHub API token set successfully.");
                Ok(())
            }
            Err(e) => {
                error!("Error setting GitHub API token: {}", e);
                Err(e)
            }
        };
    }

    // Check for --remove-token argument
    if args.len() >= 2 && args[1] == "--remove-token" {
        info!("Removing GitHub API token from secure storage...");
        return match credentials::remove_github_token() {
            Ok(_) => {
                info!("GitHub API token removed successfully.");
                Ok(())
            }
            Err(e) => {
                error!("Error removing GitHub API token: {}", e);
                Err(e)
            }
        };
    }

    // Check for --migrate-to-db argument
    if args.len() >= 2 && args[1] == "--migrate-to-db" {
        let config = config::Config::default();
        info!("Migrating CV data from JSON to SQLite database...");
        return match migrate::migrate_json_to_sqlite(
            &config.data_path_str()?,
            &config.db_path_str()?,
        ) {
            Ok(_) => {
                info!(
                    "CV data migrated successfully to database: {}",
                    config.db_path.display()
                );
                Ok(())
            }
            Err(e) => {
                error!("Error migrating CV data to database: {}", e);
                Err(e)
            }
        };
    }

    // Create configuration with custom paths and options
    let config = config::Config::default().pipe(|cfg| {
        // Process arguments to find --cache-path and other options
        args.iter().enumerate().fold(cfg, |acc, (i, arg)| {
            if arg == "--cache-path" && i + 1 < args.len() {
                let cache_path = std::path::PathBuf::from(&args[i + 1]);
                println!("Using custom GitHub cache path: {}", cache_path.display());
                acc.with_option(config::GITHUB_CACHE_KEY, &args[i + 1])
            } else if arg == "--db-path" && i + 1 < args.len() {
                let db_path = std::path::PathBuf::from(&args[i + 1]);
                println!("Using custom database path: {}", db_path.display());
                acc.with_option(config::DB_PATH_KEY, &args[i + 1])
            } else if arg == "--public-data" && i + 1 < args.len() {
                println!("Using custom public data settings: {}", args[i + 1]);
                acc.with_option(config::PUBLIC_DATA_KEY, &args[i + 1])
            } else if arg == "--db-storage" && i + 1 < args.len() {
                println!("Using custom database storage settings: {}", args[i + 1]);
                acc.with_option(config::DB_STORAGE_KEY, &args[i + 1])
            } else {
                acc
            }
        })
    });

    // Check if we should use the database
    let use_db = args.iter().any(|arg| arg == "--use-db");

    // Load CV data
    let mut cv = if use_db {
        info!(
            "Loading CV data from database: {}",
            config.db_path.display()
        );
        migrate::load_cv_from_sqlite(&config.db_path_str()?)
            .context("Failed to load CV data from database")?
    } else {
        info!("Loading CV data from JSON: {}", config.data_path.display());
        cv_data::Cv::from_json(&config.data_path_str()?)
            .context("Failed to load CV data from JSON")?
    };

    // Get GitHub token from available sources
    let config_with_token = get_github_token(config);

    // Fetch GitHub projects from sources defined in CV data
    info!("Fetching GitHub projects from sources defined in CV data");
    match github::fetch_projects_from_sources_sync(
        &cv.github_sources,
        config_with_token.github_token(),
    ) {
        Ok(github_projects) => {
            info!("Found {} GitHub projects", github_projects.len());

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

            info!("Updated CV with {} total projects", cv.projects.len());
        }
        Err(e) => {
            warn!("Failed to fetch GitHub projects: {}", e);
            info!("Continuing with existing projects data");
        }
    }

    // Load language icons and associate them with projects
    info!("Loading language icons");
    let icons_path = config_with_token
        .data_path
        .parent()
        .unwrap()
        .join("language_icons.json");
    match cv::language_icons::LanguageIcons::from_json(icons_path.to_str().unwrap()) {
        Ok(icons) => {
            info!("Found {} language icons", icons.0.len());

            // Associate language icons with projects
            for project in cv.projects.iter_mut() {
                // Extract project name without language suffix
                let mut project_name = project.name.clone();
                if let Some(lang_pos) = project_name.find(" - ") {
                    project_name = project_name[..lang_pos].to_string();
                }

                // Set the display name
                project.display_name = Some(project_name.clone());

                // Detect language from project name or technologies
                if let Some(lang) =
                    icons.detect_language_vector(&project.name, &project.technologies)
                {
                    project.language = Some(lang.clone());
                    project.language_icon = Some(icons.get_icon(&lang).to_string());
                    debug!("Detected language for project {}: {}", project_name, lang);
                }
            }
        }
        Err(e) => {
            warn!("Failed to load language icons: {}", e);
            info!("Continuing without language icons");
        }
    }

    // Filter CV data based on public_data configuration
    info!("Filtering CV data based on public_data configuration");
    let public_data_fields = config_with_token.public_data();
    debug!("Public data fields: {:?}", public_data_fields);

    // Note: In a real implementation, we would create a filtered copy of the CV data
    // based on the public_data configuration. For now, we'll just log the fields
    // that would be included.

    // Generate HTML CV and index
    info!("Generating HTML files");
    html_generator::generate_html(&cv, &config_with_token.html_output_str()?)
        .context("Failed to generate HTML files")?;

    // Copy static assets (excluding index.html which we generate)
    info!("Copying static assets");
    html_generator::copy_static_assets_except(
        &config_with_token.static_dir_str()?,
        &config_with_token.output_dir_str()?,
        &["index.html"],
    )
    .context("Failed to copy static assets")?;

    // Generate PDF CV
    info!("Generating PDF CV");
    typst_generator::generate_pdf(
        &cv,
        &config_with_token.typst_temp_str()?,
        &config_with_token.pdf_output_str()?,
    )
    .context("Failed to generate PDF CV")?;

    // Print summary
    info!("Done! Output files:");
    info!("  - HTML CV: {}", config_with_token.html_output.display());
    info!("  - PDF CV: {}", config_with_token.pdf_output.display());
    info!(
        "  - Static assets: {}",
        config_with_token.output_dir.display()
    );

    Ok(())
}
