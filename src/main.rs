#[allow(dead_code)]
// mod asset_processor; // Disabled for now
mod bundler;

mod cv_data;
#[allow(dead_code)]
mod git_config;
mod github;
mod html_generator;
mod language_icons;
// #[allow(dead_code)]
// mod runtime; // Disabled for now
mod typst_generator;
mod unified_config;

use anyhow::{Context, Result};
// use cv::logging; // Disabled for now
use im::Vector;
use std::env;
use tracing::{debug, error, info, warn};
use unified_config::AppConfig;

// Extension trait to enable method chaining with pipe
#[allow(dead_code)]
trait Pipe: Sized {
    fn pipe<F, R>(self, f: F) -> R
    where
        F: FnOnce(Self) -> R,
    {
        f(self)
    }
}

/// Initialize simple logging with tracing
fn init_logging() {
    use tracing_subscriber;

    tracing_subscriber::fmt()
        .with_target(false)
        .with_thread_ids(false)
        .with_level(true)
        .with_ansi(true)
        .init();

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
    git_config::read_github_token().map(|token_opt| {
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
/// An AppConfig with the token set if found
fn get_github_token_app(config: AppConfig) -> AppConfig {
    // Check if running in GitHub Actions
    match env::var("GITHUB_ACTIONS") {
        Ok(actions) if actions == "true" => {
            // We're running in GitHub Actions, check for GITHUB_TOKEN
            match env::var("GITHUB_TOKEN") {
                Ok(token) => {
                    info!("Using GitHub API token from GitHub Actions for authentication");
                    let mut updated_config = config.clone();
                    updated_config.github_token = Some(token);
                    updated_config
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
                Ok(Some(token)) => {
                    let mut updated_config = config.clone();
                    updated_config.github_token = Some(token);
                    updated_config
                }
                Ok(None) => {
                    // Token not found in secure storage, try environment variable
                    match get_token_from_env() {
                        Some(token) => {
                            let mut updated_config = config.clone();
                            updated_config.github_token = Some(token);
                            updated_config
                        }
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
                        Some(token) => {
                            let mut updated_config = config.clone();
                            updated_config.github_token = Some(token);
                            updated_config
                        }
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
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    init_logging();

    // Parse command-line arguments
    let args: Vector<String> = env::args().collect();

    // Check for --set-token argument
    if args.len() >= 3 && args[1] == "--set-token" {
        let token = &args[2];
        info!("Setting GitHub API token in secure storage...");
        return match git_config::write_github_token(token) {
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
        return match git_config::remove_github_token() {
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

    // Load configuration from all available sources
    let mut config = AppConfig::load().context("Failed to load configuration")?;

    // Process command-line arguments to override configuration
    for i in 0..args.len() {
        if i + 1 < args.len() {
            match args[i].as_str() {
                "--cache-path" => {
                    let cache_path = std::path::PathBuf::from(&args[i + 1]);
                    info!("Using custom GitHub cache path: {}", cache_path.display());
                    config = config.with_option(unified_config::GITHUB_CACHE_KEY, &args[i + 1]);
                }
                "--public-data" => {
                    info!("Using custom public data settings: {}", args[i + 1]);
                    config = config.with_option(unified_config::PUBLIC_DATA_KEY, &args[i + 1]);
                }
                _ => {}
            }
        }
    }

    // Load CV data - prioritize local file (which may contain real data from content branch in CI)
    let mut cv = {
        info!(
            "Loading CV data from local file: {}",
            config.data_path.display()
        );
        cv_data::Cv::from_json(&config.data_path.to_string_lossy())
            .context("Failed to load CV data")?
    };

    // Get GitHub token from available sources
    let config_with_token = get_github_token_app(config);

    // Fetch GitHub projects from sources defined in CV data
    info!("Fetching GitHub projects from sources defined in CV data using GitHub CLI");
    match github::fetch_projects_from_sources(&cv.github_sources) {
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

    // Fetch GitHub avatar URL if no custom profile image is provided
    if cv.personal_info.profile_image.is_none() {
        // Try to get GitHub username from sources for avatar
        let github_username = cv
            .github_sources
            .iter()
            .find_map(|source| source.username.as_ref());

        if let Some(username) = github_username {
            info!("Fetching GitHub avatar for user: {}", username);
            match github::fetch_github_avatar(username) {
                Ok(avatar_url) => {
                    cv.personal_info.github_avatar_url = Some(avatar_url);
                    info!("Successfully fetched GitHub avatar URL");
                }
                Err(e) => {
                    warn!("Failed to fetch GitHub avatar: {}", e);
                    info!("Will use default placeholder image");
                }
            }
        } else {
            info!("No GitHub username found in sources, using default placeholder image");
        }
    } else {
        info!("Using custom profile image from CV data");
    }

    // Load language icons and associate them with projects
    info!("Loading language icons");
    let icons_path = config_with_token
        .data_path
        .parent()
        .unwrap()
        .join("language_icons.json");
    match language_icons::LanguageIcons::from_json(icons_path.to_str().unwrap()) {
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

    // Copy static assets (excluding generated HTML files)
    info!("Copying static assets");
    html_generator::copy_static_assets_except(
        &config_with_token.static_dir_str()?,
        &config_with_token.output_dir_str()?,
        &["index.html", "cv.html", "projects.html", "blog.html"],
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

    // Process and bundle assets (disabled for now)
    info!("Skipping asset processing for now");
    // bundler::process_assets("bundle.toml", &config_with_token.static_dir_str()?)
    //     .context("Failed to process and bundle assets")?;

    // Print summary
    info!("Done! Output files:");
    info!("  - HTML CV: {}", config_with_token.html_output.display());
    info!("  - PDF CV: {}", config_with_token.pdf_output.display());
    info!(
        "  - Static assets: {}",
        config_with_token.output_dir.display()
    );
    info!(
        "  - Bundled assets: {}/[bundle_name].bundle.[css|js]",
        config_with_token.output_dir.display()
    );

    Ok(())
}
