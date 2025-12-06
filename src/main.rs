use anyhow::{Context, Result};
use cv_generator::{
    cv_data::Cv,
    github,
    github_cache::GitHubCache,
    html_generator,
    language_icons::LanguageIcons,
    performance::BuildProfiler,
    site_config::SiteConfig,
    typst_generator,
    unified_config::{self, AppConfig},
};
use im::Vector;
use std::env;
use std::fs;
use std::path::Path;
use tracing::{debug, info, warn};

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

/// Download and save an image from URL, returning the actual path used
/// (may differ from requested path if format detection changes extension)
async fn download_and_save_image(url: &str, path: &str) -> Result<String> {
    // Ensure the parent directory exists
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent)?;
    }

    // Build client with proper headers for GitHub
    let client = reqwest::Client::builder()
        .user_agent("cv-generator")
        .build()?;

    // Download the image
    debug!("Downloading image from: {}", url);
    let response = client.get(url).send().await?;

    // Check for successful response
    let status = response.status();
    debug!("Response status: {}", status);

    if !status.is_success() {
        anyhow::bail!("Failed to download image: HTTP {}", status);
    }

    // Verify content type is an image
    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    debug!("Content-Type: {}", content_type);

    if !content_type.starts_with("image/") {
        anyhow::bail!(
            "Downloaded content is not an image (content-type: {})",
            content_type
        );
    }

    let bytes = response.bytes().await?;
    debug!("Downloaded {} bytes", bytes.len());

    // Detect actual format from magic bytes
    let is_png = bytes.len() >= 8 && &bytes[0..8] == b"\x89PNG\r\n\x1a\n";
    let is_jpeg = bytes.len() >= 2 && &bytes[0..2] == b"\xff\xd8";

    if !is_png && !is_jpeg {
        anyhow::bail!(
            "Downloaded file is not a valid PNG or JPEG image (first bytes: {:02x?})",
            &bytes[0..bytes.len().min(8)]
        );
    }

    // Determine correct extension based on actual format
    let actual_path = if is_jpeg && path.ends_with(".png") {
        path.replace(".png", ".jpg")
    } else if is_png && path.ends_with(".jpg") {
        path.replace(".jpg", ".png")
    } else {
        path.to_string()
    };

    debug!(
        "Image format: {}, saving to: {}",
        if is_png { "PNG" } else { "JPEG" },
        actual_path
    );

    // Save to file
    fs::write(&actual_path, bytes)?;
    Ok(actual_path)
}

/// Main entry point for the CV generator application
///
/// This function initializes the configuration, loads the CV data,
/// fetches GitHub projects, and generates both HTML and PDF versions of the CV.
///
/// Command-line arguments:
/// - `--cache-path <path>`: Set a custom path for the GitHub cache file
/// - `--public-data <config>`: Set public data configuration
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    init_logging();

    // Initialize performance profiler
    let mut profiler = BuildProfiler::new();

    // Parse command-line arguments
    let args: Vector<String> = env::args().collect();

    // Load GitHub cache
    let cache_path = "cache/github_cache.json";
    let mut github_cache = profiler.time_operation("Load GitHub cache", || {
        GitHubCache::load_or_default(cache_path)
    });

    // Load configuration from all available sources
    let base_config = AppConfig::load().context("Failed to load configuration")?;

    // Process command-line arguments to override configuration
    let config = args.iter().enumerate().fold(base_config, |cfg, (i, arg)| {
        if i + 1 < args.len() {
            match arg.as_str() {
                "--cache-path" => {
                    let cache_path = std::path::PathBuf::from(&args[i + 1]);
                    info!("Using custom GitHub cache path: {}", cache_path.display());
                    cfg.with_option(unified_config::GITHUB_CACHE_KEY, &args[i + 1])
                }
                "--public-data" => {
                    info!("Using custom public data settings: {}", args[i + 1]);
                    cfg.with_option(unified_config::PUBLIC_DATA_KEY, &args[i + 1])
                }
                _ => cfg,
            }
        } else {
            cfg
        }
    });

    // Load CV data - prioritize local file (which may contain real data from content branch in CI)
    let mut cv = profiler.time_operation("Load CV data", || {
        info!(
            "Loading CV data from local file: {}",
            config.data_path.display()
        );
        Cv::from_json(&config.data_path.to_string_lossy()).context("Failed to load CV data")
    })?;

    // We're using GitHub CLI (gh) which handles authentication automatically
    info!("Fetching GitHub projects from sources defined in CV data using GitHub CLI");
    match profiler.time_operation("Fetch GitHub projects", || {
        github::fetch_projects_from_sources_cached(&cv.github_sources, &mut github_cache)
    }) {
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

    // Check if custom profile image exists, clear it if not
    if let Some(ref profile_path) = cv.personal_info.profile_image {
        // Try multiple locations for the profile image:
        // 1. Absolute path or dist/ prefix (as-is)
        // 2. Relative to static directory (static/img/...)
        // 3. Relative to repo root (img/...)
        let candidates = if profile_path.starts_with("dist/") || Path::new(profile_path).is_absolute() {
            vec![Path::new(profile_path).to_path_buf()]
        } else {
            vec![
                config.static_dir.join(profile_path),
                Path::new(profile_path).to_path_buf(),
            ]
        };

        let resolved_path = candidates.iter().find(|p| p.exists());

        match resolved_path {
            Some(path) => {
                info!("Using custom profile image: {}", path.display());
                // Update the path to the resolved location for downstream use
                cv.personal_info.profile_image = Some(path.to_string_lossy().to_string());
            }
            None => {
                warn!(
                    "Profile image not found (tried: {}), will fall back to GitHub avatar",
                    candidates.iter().map(|p| p.display().to_string()).collect::<Vec<_>>().join(", ")
                );
                cv.personal_info.profile_image = None;
            }
        }
    }

    // Fetch GitHub avatar URL if no valid profile image is available
    if cv.personal_info.profile_image.is_none() {
        let github_username = cv
            .github_sources
            .iter()
            .find_map(|source| source.username.as_ref());

        if let Some(username) = github_username {
            info!("Fetching GitHub avatar for user: {}", username);
            match profiler.time_operation("Fetch GitHub avatar", || {
                github::fetch_github_avatar_cached(username, &mut github_cache)
            }) {
                Ok(avatar_url) => {
                    cv.personal_info.github_avatar_url = Some(avatar_url.clone());
                    info!("Successfully fetched GitHub avatar URL");

                    // Download and save avatar for PDF generation
                    let avatar_path = format!("{}/img/profile.png", config.output_dir.display());
                    match download_and_save_image(&avatar_url, &avatar_path).await {
                        Ok(actual_path) => {
                            // Keep GitHub avatar URL for HTML, and use local file for PDF
                            // The profile_image will be used by Typst, github_avatar_url by HTML
                            cv.personal_info.profile_image = Some(actual_path);
                            info!("Downloaded GitHub avatar for PDF generation");
                        }
                        Err(e) => {
                            warn!("Failed to download GitHub avatar: {}", e);
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to fetch GitHub avatar: {}", e);
                    info!("Will use default placeholder image");
                }
            }
        } else {
            info!("No GitHub username found in sources, using default placeholder image");
        }
    }

    // Load language icons and associate them with projects
    info!("Loading language icons");
    let icons_path = config
        .data_path
        .parent()
        .unwrap()
        .join("language_icons.json");
    match LanguageIcons::from_json(icons_path.to_str().unwrap()) {
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
    let public_data_fields = config.public_data();
    debug!("Public data fields: {:?}", public_data_fields);

    // Note: In a real implementation, we would create a filtered copy of the CV data
    // based on the public_data configuration. For now, we'll just log the fields
    // that would be included.

    // Load site configuration (menu, navigation, etc.)
    info!("Loading site configuration");
    let site_config = SiteConfig::from_json("config/site.json").unwrap_or_else(|e| {
        warn!("Failed to load site config: {}. Using defaults.", e);
        SiteConfig::default()
    });

    // Generate HTML CV and index
    info!("Generating HTML files");
    profiler.time_operation("Generate HTML files", || {
        html_generator::generate_html(&cv, &site_config, &config.html_output_str()?)
            .context("Failed to generate HTML files")
    })?;

    // Copy static assets (excluding generated HTML files)
    info!("Copying static assets");
    profiler.time_operation("Copy static assets", || {
        html_generator::copy_static_assets_except(
            &config.static_dir_str()?,
            &config.output_dir_str()?,
            &["index.html", "cv.html", "projects.html", "blog.html"],
        )
        .context("Failed to copy static assets")
    })?;

    // Optimize CSS and JS files after copying static assets
    info!("Optimizing CSS and JS assets");
    profiler.time_operation("Optimize assets", || {
        html_generator::optimize_assets(Path::new(&config.output_dir_str()?))
            .context("Failed to optimize assets")
    })?;

    // Generate PDF CV
    info!("Generating PDF CV");
    profiler.time_operation("Generate PDF CV", || {
        let typst_config = site_config
            .get_typst_config()
            .context("Failed to get Typst configuration")?;
        typst_generator::generate_pdf(
            &cv,
            &typst_config,
            &config.typst_temp_str()?,
            &config.pdf_output_str()?,
        )
        .context("Failed to generate PDF CV")
    })?;

    // Process and bundle assets (disabled for now)
    info!("Skipping asset processing for now");
    // bundler::process_assets("bundle.toml", &config.static_dir_str()?)
    //     .context("Failed to process and bundle assets")?;

    // Save GitHub cache
    profiler.time_operation("Save GitHub cache", || {
        github_cache.cleanup_expired();
        github_cache.save(cache_path)
    })?;

    // Print performance summary
    profiler.print_summary();

    // Print output summary
    info!("Done! Output files:");
    info!("  - HTML CV: {}", config.html_output.display());
    info!("  - PDF CV: {}", config.pdf_output.display());
    info!("  - Static assets: {}", config.output_dir.display());
    info!(
        "  - Bundled assets: {}/[bundle_name].bundle.[css|js]",
        config.output_dir.display()
    );

    Ok(())
}
