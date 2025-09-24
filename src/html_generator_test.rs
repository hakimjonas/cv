#[cfg(test)]
mod tests {
    use crate::cv_data::{Cv, Education, Experience, GitHubSource, PersonalInfo, Project, SkillCategory};
    use crate::html_generator;
    use anyhow::Result;
    use im::{HashMap, Vector};
    use std::fs;
    use std::path::Path;
    use tempfile::TempDir;

    // Helper function to create a minimal valid CV for testing
    fn create_test_cv() -> Cv {
        let personal_info = PersonalInfo {
            name: "Test User".to_string(),
            title: "Software Developer".to_string(),
            email: "test@example.com".to_string(),
            phone: Some("123-456-7890".to_string()),
            website: Some("https://example.com".to_string()),
            location: Some("Test City, Test Country".to_string()),
            summary: "Test summary".to_string(),
            social_links: {
                let mut links = HashMap::new();
                links = links.update("github".to_string(), "https://github.com/testuser".to_string());
                links = links.update("linkedin".to_string(), "https://linkedin.com/in/testuser".to_string());
                links
            },
        };

        let experiences = Vector::from(vec![Experience {
            company: "Test Company".to_string(),
            position: "Software Developer".to_string(),
            start_date: "2020-01-01".to_string(),
            end_date: Some("2023-01-01".to_string()),
            location: Some("Test City, Test Country".to_string()),
            description: "Test description".to_string(),
            achievements: Vector::from(vec!["Achievement 1".to_string(), "Achievement 2".to_string()]),
            technologies: Vector::from(vec!["Rust".to_string(), "TypeScript".to_string()]),
        }]);

        let education = Vector::from(vec![Education {
            institution: "Test University".to_string(),
            degree: "Bachelor of Science".to_string(),
            field: "Computer Science".to_string(),
            start_date: "2016-01-01".to_string(),
            end_date: Some("2020-01-01".to_string()),
            location: Some("Test City, Test Country".to_string()),
            gpa: Some("3.8/4.0".to_string()),
            achievements: Vector::from(vec!["Dean's List".to_string()]),
        }]);

        let skill_categories = Vector::from(vec![SkillCategory {
            name: "Programming Languages".to_string(),
            skills: Vector::from(vec!["Rust".to_string(), "TypeScript".to_string(), "Python".to_string()]),
        }]);

        let projects = Vector::from(vec![Project {
            name: "Test Project".to_string(),
            description: "A test project".to_string(),
            url: Some("https://example.com/project".to_string()),
            repository: Some("https://github.com/testuser/test-project".to_string()),
            technologies: Vector::from(vec!["Rust".to_string(), "TypeScript".to_string()]),
            highlights: Vector::from(vec!["Highlight 1".to_string(), "Highlight 2".to_string()]),
            stars: Some(42),
            owner_username: Some("testuser".to_string()),
            owner_avatar: Some("https://github.com/testuser.png".to_string()),
            language: None,
            language_icon: None,
            display_name: None,
        }]);

        let mut languages = HashMap::new();
        languages = languages.update("English".to_string(), "Native".to_string());
        languages = languages.update("Spanish".to_string(), "Intermediate".to_string());

        let certifications = Vector::from(vec!["Test Certification".to_string()]);

        let github_sources = Vector::from(vec![GitHubSource {
            username: Some("testuser".to_string()),
            organization: None,
        }]);

        Cv {
            personal_info,
            experiences,
            education,
            skill_categories,
            projects,
            languages,
            certifications,
            github_sources,
        }
    }

    // Helper function to create a temporary directory with static assets
    fn create_test_static_dir() -> Result<(TempDir, TempDir)> {
        // Create temporary directories for static and output
        let static_dir = TempDir::new()?;
        let output_dir = TempDir::new()?;

        // Create some static assets
        let css_dir = static_dir.path().join("css");
        fs::create_dir_all(&css_dir)?;
        fs::write(css_dir.join("style.css"), "body { font-family: Arial; }")?;

        let js_dir = static_dir.path().join("js");
        fs::create_dir_all(&js_dir)?;
        fs::write(js_dir.join("script.js"), "console.log('Hello, world!');")?;

        let img_dir = static_dir.path().join("img");
        fs::create_dir_all(&img_dir)?;
        fs::write(img_dir.join("logo.png"), "fake image data")?;

        // Create an index.html file that should be excluded when copying
        fs::write(static_dir.path().join("index.html"), "<html><body>Static index</body></html>")?;

        Ok((static_dir, output_dir))
    }

    // Test generating HTML files
    #[test]
    fn test_generate_html() -> Result<()> {
        let cv = create_test_cv();
        let output_dir = TempDir::new()?;
        let output_path = output_dir.path().to_str().unwrap();

        // Generate HTML files
        html_generator::generate_html(&cv, output_path)?;

        // Check that the expected files were generated
        assert!(Path::new(&format!("{}/index.html", output_path)).exists());
        assert!(Path::new(&format!("{}/cv.html", output_path)).exists());
        assert!(Path::new(&format!("{}/projects.html", output_path)).exists());
        assert!(Path::new(&format!("{}/blog.html", output_path)).exists());

        // Check that the generated HTML contains expected content
        let index_html = fs::read_to_string(format!("{}/index.html", output_path))?;
        assert!(index_html.contains(&cv.personal_info.name));
        assert!(index_html.contains(&cv.personal_info.title));

        let cv_html = fs::read_to_string(format!("{}/cv.html", output_path))?;
        assert!(cv_html.contains(&cv.personal_info.name));
        assert!(cv_html.contains(&cv.personal_info.email));
        assert!(cv_html.contains(&cv.experiences[0].company));
        assert!(cv_html.contains(&cv.education[0].institution));
        assert!(cv_html.contains(&cv.skill_categories[0].name));

        let projects_html = fs::read_to_string(format!("{}/projects.html", output_path))?;
        assert!(projects_html.contains(&cv.projects[0].name));
        assert!(projects_html.contains(&cv.projects[0].description));

        Ok(())
    }

    // Test copying static assets
    #[test]
    fn test_copy_static_assets() -> Result<()> {
        let (static_dir, output_dir) = create_test_static_dir()?;
        let static_path = static_dir.path().to_str().unwrap();
        let output_path = output_dir.path().to_str().unwrap();

        // Copy static assets
        html_generator::copy_static_assets(static_path, output_path)?;

        // Check that the expected files were copied
        assert!(Path::new(&format!("{}/css/style.css", output_path)).exists());
        assert!(Path::new(&format!("{}/js/script.js", output_path)).exists());
        assert!(Path::new(&format!("{}/img/logo.png", output_path)).exists());
        assert!(Path::new(&format!("{}/index.html", output_path)).exists());

        // Check the content of the copied files
        let css_content = fs::read_to_string(format!("{}/css/style.css", output_path))?;
        assert_eq!(css_content, "body { font-family: Arial; }");

        let js_content = fs::read_to_string(format!("{}/js/script.js", output_path))?;
        assert_eq!(js_content, "console.log('Hello, world!');");

        Ok(())
    }

    // Test copying static assets with exclusions
    #[test]
    fn test_copy_static_assets_except() -> Result<()> {
        let (static_dir, output_dir) = create_test_static_dir()?;
        let static_path = static_dir.path().to_str().unwrap();
        let output_path = output_dir.path().to_str().unwrap();

        // Copy static assets except index.html
        html_generator::copy_static_assets_except(static_path, output_path, &["index.html"])?;

        // Check that the expected files were copied
        assert!(Path::new(&format!("{}/css/style.css", output_path)).exists());
        assert!(Path::new(&format!("{}/js/script.js", output_path)).exists());
        assert!(Path::new(&format!("{}/img/logo.png", output_path)).exists());

        // Check that index.html was not copied
        assert!(!Path::new(&format!("{}/index.html", output_path)).exists());

        Ok(())
    }

    // Test minification of HTML content
    #[test]
    fn test_minify_html_content() -> Result<()> {
        let html = r#"<!DOCTYPE html>
        <html>
            <head>
                <title>Test</title>
            </head>
            <body>
                <h1>Hello, world!</h1>
                <!-- This is a comment -->
            </body>
        </html>"#;

        // Use the private function through a public function that calls it
        let output_dir = TempDir::new()?;
        let output_path = output_dir.path().join("test.html").to_str().unwrap().to_string();

        // Write the HTML to a file
        html_generator::write_file(&output_path, html)?;

        // Read the file back and check that it's minified
        let minified = fs::read_to_string(&output_path)?;
        
        // The minified HTML should be smaller than the original
        assert!(minified.len() < html.len());
        
        // The minified HTML should not contain comments
        assert!(!minified.contains("<!-- This is a comment -->"));
        
        // The minified HTML should still contain the essential content
        assert!(minified.contains("Hello, world!"));

        Ok(())
    }

    // Test error handling for missing directories
    #[test]
    fn test_error_handling_missing_directory() {
        let cv = create_test_cv();
        let result = html_generator::generate_html(&cv, "/nonexistent/directory");
        
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Failed to create directory") || 
                err.contains("No such file or directory") ||
                err.contains("Permission denied"));
    }

    // Test generating HTML with a minimal CV
    #[test]
    fn test_generate_html_minimal_cv() -> Result<()> {
        // Create a minimal CV with only required fields
        let personal_info = PersonalInfo {
            name: "Minimal User".to_string(),
            title: "Developer".to_string(),
            email: "minimal@example.com".to_string(),
            phone: None,
            website: None,
            location: None,
            summary: "Minimal summary".to_string(),
            social_links: HashMap::new(),
        };

        let cv = Cv {
            personal_info,
            experiences: Vector::new(),
            education: Vector::new(),
            skill_categories: Vector::new(),
            projects: Vector::new(),
            languages: HashMap::new(),
            certifications: Vector::new(),
            github_sources: Vector::new(),
        };

        let output_dir = TempDir::new()?;
        let output_path = output_dir.path().to_str().unwrap();

        // Generate HTML files
        html_generator::generate_html(&cv, output_path)?;

        // Check that the expected files were generated
        assert!(Path::new(&format!("{}/index.html", output_path)).exists());
        assert!(Path::new(&format!("{}/cv.html", output_path)).exists());
        assert!(Path::new(&format!("{}/projects.html", output_path)).exists());
        assert!(Path::new(&format!("{}/blog.html", output_path)).exists());

        // Check that the generated HTML contains expected content
        let index_html = fs::read_to_string(format!("{}/index.html", output_path))?;
        assert!(index_html.contains("Minimal User"));
        assert!(index_html.contains("Developer"));

        Ok(())
    }
}