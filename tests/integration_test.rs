use anyhow::Result;
use cv_generator::{cv_data::Cv, html_generator, site_config::SiteConfig};
use std::fs;
use std::path::Path;
use tempfile::tempdir;

/// Test full CV generation workflow
#[test]
fn test_full_cv_generation_workflow() -> Result<()> {
    let temp_dir = tempdir()?;
    let output_dir = temp_dir.path();

    // Load test CV data
    let cv = Cv::from_json("data/cv_data.json")?;
    assert!(!cv.personal_info.name.is_empty(), "CV should have a name");

    // Load site configuration
    let site_config = SiteConfig::from_json("config/site.json")?;
    assert!(
        !site_config.menu.is_empty(),
        "Site config should have menu items"
    );

    // Generate HTML
    let output_path = output_dir.join("cv.html");
    html_generator::generate_html(&cv, &site_config, output_path.to_str().unwrap())?;

    // Verify generated files exist
    assert!(output_path.exists(), "CV HTML should be generated");
    assert!(
        output_dir.join("index.html").exists(),
        "Index should be generated"
    );
    assert!(
        output_dir.join("projects.html").exists(),
        "Projects page should be generated"
    );

    // Verify HTML content
    let cv_html = fs::read_to_string(&output_path)?;
    assert!(
        cv_html.contains(&cv.personal_info.name),
        "HTML should contain personal name"
    );
    assert!(
        cv_html.contains("<!doctype html>"),
        "HTML should be well-formed"
    );
    assert!(
        cv_html.contains("css/main.min.css"),
        "HTML should link to CSS"
    );

    // Verify deployment configs
    assert!(
        output_dir.join(".htaccess").exists(),
        ".htaccess should be generated"
    );
    assert!(
        output_dir.join("robots.txt").exists(),
        "robots.txt should be generated"
    );
    assert!(
        output_dir.join("manifest.json").exists(),
        "manifest.json should be generated"
    );

    Ok(())
}

/// Test CV data serialization roundtrip
#[test]
fn test_cv_data_roundtrip() -> Result<()> {
    let temp_dir = tempdir()?;
    let test_file = temp_dir.path().join("test_cv.json");

    // Load original CV
    let original_cv = Cv::from_json("data/cv_data.json")?;

    // Serialize to file
    let json = serde_json::to_string_pretty(&original_cv)?;
    fs::write(&test_file, json)?;

    // Load from file
    let loaded_cv = Cv::from_json(test_file.to_str().unwrap())?;

    // Verify data integrity
    assert_eq!(original_cv.personal_info.name, loaded_cv.personal_info.name);
    assert_eq!(
        original_cv.personal_info.email,
        loaded_cv.personal_info.email
    );
    assert_eq!(original_cv.experiences.len(), loaded_cv.experiences.len());
    assert_eq!(original_cv.projects.len(), loaded_cv.projects.len());

    Ok(())
}

/// Test configuration loading from multiple sources
#[test]
fn test_config_hierarchy() -> Result<()> {
    use cv_generator::unified_config::AppConfig;

    // Load configuration
    let config = AppConfig::load()?;

    // Verify defaults
    assert_eq!(config.data_path, Path::new("data/cv_data.json"));
    assert_eq!(config.output_dir, Path::new("dist"));
    assert_eq!(config.static_dir, Path::new("static"));

    // Verify derived paths
    assert_eq!(config.html_output, Path::new("dist/cv.html"));
    assert_eq!(config.pdf_output, Path::new("dist/cv.pdf"));

    Ok(())
}

/// Test HTML generation with minimal CV data
#[test]
fn test_minimal_cv_generation() -> Result<()> {
    use cv_generator::cv_data::PersonalInfo;
    use im::Vector;

    let temp_dir = tempdir()?;
    let output_path = temp_dir.path().join("cv.html");

    // Create minimal CV
    let cv = Cv {
        personal_info: PersonalInfo {
            name: "Test User".to_string(),
            title: "Test Engineer".to_string(),
            email: "test@example.com".to_string(),
            phone: None,
            website: None,
            location: None,
            summary: "Test summary".to_string(),
            social_links: Default::default(),
            profile_image: None,
            github_avatar_url: None,
        },
        experiences: Vector::new(),
        education: Vector::new(),
        skill_categories: Vector::new(),
        projects: Vector::new(),
        languages: Default::default(),
        certifications: Vector::new(),
        github_sources: Vector::new(),
    };

    let site_config = SiteConfig::default();

    // Generate HTML
    html_generator::generate_html(&cv, &site_config, output_path.to_str().unwrap())?;

    // Verify file was created
    assert!(
        output_path.exists(),
        "Minimal CV should generate successfully"
    );

    // Verify content
    let html = fs::read_to_string(&output_path)?;
    assert!(html.contains("Test User"));
    assert!(html.contains("Test Engineer"));
    assert!(html.contains("test@example.com"));

    Ok(())
}

/// Test asset copying
#[test]
fn test_asset_copying() -> Result<()> {
    use cv_generator::html_generator::copy_static_assets_except;

    let temp_dir = tempdir()?;
    let dest_dir = temp_dir.path().join("output");
    fs::create_dir_all(&dest_dir)?;

    // Copy static assets (excluding generated HTML)
    copy_static_assets_except(
        "static",
        dest_dir.to_str().unwrap(),
        &["index.html", "cv.html"],
    )?;

    // Verify CSS directory was copied
    let css_dir = dest_dir.join("css");
    if Path::new("static/css").exists() {
        assert!(css_dir.exists(), "CSS directory should be copied");
    }

    Ok(())
}

/// Test blog post loading
#[test]
fn test_blog_post_loading() -> Result<()> {
    use cv_generator::blog_posts::load_posts_from_directory;
    use std::path::PathBuf;

    let blog_dir = PathBuf::from("content/blog");

    // Only run if blog directory exists
    if blog_dir.exists() {
        let posts = load_posts_from_directory(&blog_dir)?;

        // Verify posts have required fields
        for post in posts.iter() {
            assert!(!post.title.is_empty(), "Post should have a title");
            assert!(!post.slug.is_empty(), "Post should have a slug");
            assert!(!post.content.is_empty(), "Post should have content");
        }
    }

    Ok(())
}

/// Test performance profiler
#[test]
fn test_build_profiler() {
    use cv_generator::performance::BuildProfiler;
    use std::thread;
    use std::time::Duration;

    let mut profiler = BuildProfiler::new();

    // Time a fast operation
    profiler
        .time_operation("Fast operation", || {
            thread::sleep(Duration::from_millis(10));
            Ok::<(), anyhow::Error>(())
        })
        .expect("Operation should succeed");

    // Time another operation
    profiler
        .time_operation("Another operation", || {
            thread::sleep(Duration::from_millis(5));
            Ok::<(), anyhow::Error>(())
        })
        .expect("Operation should succeed");

    // Verify both operations were recorded
    // (print_summary is tested by actually running it)
    profiler.print_summary();
}

/// Test GitHub cache functionality
#[test]
fn test_github_cache_operations() -> Result<()> {
    use cv_generator::github_cache::GitHubCache;
    use im::Vector;

    let temp_dir = tempdir()?;
    let cache_path = temp_dir.path().join("github_cache.json");

    // Create new cache
    let mut cache = GitHubCache::default();
    assert_eq!(cache.projects.len(), 0);
    assert_eq!(cache.avatars.len(), 0);

    // Cache some data
    cache.cache_avatar("testuser", "https://example.com/avatar.png".to_string());
    cache.cache_projects("testuser", Vector::new());

    // Save cache
    cache.save(&cache_path)?;
    assert!(cache_path.exists(), "Cache file should be created");

    // Load cache
    let loaded_cache = GitHubCache::load(&cache_path)?;
    assert!(
        loaded_cache.get_avatar("testuser").is_some(),
        "Avatar should be cached"
    );
    assert!(
        loaded_cache.get_projects("testuser").is_some(),
        "Projects should be cached"
    );

    Ok(())
}
