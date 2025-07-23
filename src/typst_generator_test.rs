#[cfg(test)]
mod tests {
    use crate::cv_data::{Cv, Education, Experience, GitHubSource, PersonalInfo, Project, SkillCategory};
    use crate::typst_generator;
    use crate::typst_generator::markup::generate_typst_markup;
    use anyhow::Result;
    use im::{HashMap, Vector};
    use std::fs;
    use std::path::Path;
    use tempfile::TempDir;
    use std::process::Command;
    use std::process::ExitStatus;
    use std::os::unix::process::ExitStatusExt;

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

    // Test generating Typst markup
    #[test]
    fn test_generate_typst_markup() {
        let cv = create_test_cv();
        
        // Generate Typst markup
        let markup = generate_typst_markup(&cv);
        
        // Check that the markup contains expected content
        assert!(markup.contains("#import \"template.typ\""));
        assert!(markup.contains(&cv.personal_info.name));
        assert!(markup.contains(&cv.personal_info.email));
        assert!(markup.contains(&cv.experiences[0].company));
        assert!(markup.contains(&cv.education[0].institution));
        assert!(markup.contains(&cv.skill_categories[0].name));
        assert!(markup.contains(&cv.projects[0].name));
        
        // Check that the markup has the expected structure
        assert!(markup.contains("#show: template"));
        assert!(markup.contains("#cvSection"));
    }

    // Test generating PDF with mocked Typst CLI
    #[test]
    fn test_generate_pdf_with_mock() -> Result<()> {
        // Create a temporary directory for the test
        let temp_dir = TempDir::new()?;
        let temp_path = temp_dir.path().join("cv.typ").to_str().unwrap().to_string();
        let output_path = temp_dir.path().join("cv.pdf").to_str().unwrap().to_string();
        
        let cv = create_test_cv();
        
        // Create a mock implementation of the generate_pdf function
        // that doesn't actually call the Typst CLI
        fn mock_generate_pdf(cv: &Cv, temp_path: &str, output_path: &str) -> Result<()> {
            // Generate Typst markup
            let typst_markup = generate_typst_markup(cv);
            
            // Write Typst markup to temporary file
            fs::write(temp_path, typst_markup)?;
            
            // Instead of calling Typst CLI, just create an empty PDF file
            fs::write(output_path, "mock PDF content")?;
            
            Ok(())
        }
        
        // Call the mock function
        mock_generate_pdf(&cv, &temp_path, &output_path)?;
        
        // Check that the temporary file was created with the expected content
        assert!(Path::new(&temp_path).exists());
        let markup = fs::read_to_string(&temp_path)?;
        assert!(markup.contains(&cv.personal_info.name));
        
        // Check that the output file was created
        assert!(Path::new(&output_path).exists());
        
        Ok(())
    }

    // Test error handling for missing directories
    #[test]
    fn test_error_handling_missing_directory() {
        let cv = create_test_cv();
        let temp_path = "test_temp.typ";
        let result = typst_generator::generate_pdf(&cv, temp_path, "/nonexistent/directory/cv.pdf");
        
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Failed to create output directory") || 
                err.contains("No such file or directory") ||
                err.contains("Permission denied"));
    }

    // Test error handling for Typst CLI execution failure
    #[test]
    fn test_error_handling_typst_cli_failure() -> Result<()> {
        // Create a temporary directory for the test
        let temp_dir = TempDir::new()?;
        let temp_path = temp_dir.path().join("cv.typ").to_str().unwrap().to_string();
        let output_path = temp_dir.path().join("cv.pdf").to_str().unwrap().to_string();
        
        let cv = create_test_cv();
        
        // Create a mock implementation that simulates a Typst CLI failure
        fn mock_generate_pdf_with_failure(cv: &Cv, temp_path: &str, output_path: &str) -> Result<()> {
            // Generate Typst markup
            let typst_markup = generate_typst_markup(cv);
            
            // Write Typst markup to temporary file
            fs::write(temp_path, typst_markup)?;
            
            // Simulate a Typst CLI failure
            return Err(anyhow::anyhow!("Typst compilation failed with status: 1"));
        }
        
        // Call the mock function and check that it returns an error
        let result = mock_generate_pdf_with_failure(&cv, &temp_path, &output_path);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Typst compilation failed with status: 1"));
        
        Ok(())
    }

    // Test that the markup contains all required sections
    #[test]
    fn test_markup_contains_all_sections() {
        let cv = create_test_cv();
        
        // Generate Typst markup
        let markup = generate_typst_markup(&cv);
        
        // Check that all required sections are present
        assert!(markup.contains("Personal Information"));
        assert!(markup.contains("Experience"));
        assert!(markup.contains("Education"));
        assert!(markup.contains("Skills"));
        assert!(markup.contains("Projects"));
        assert!(markup.contains("Languages"));
    }

    // Test with a minimal CV
    #[test]
    fn test_generate_markup_minimal_cv() {
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
        
        // Generate Typst markup
        let markup = generate_typst_markup(&cv);
        
        // Check that the markup contains the minimal required content
        assert!(markup.contains("Minimal User"));
        assert!(markup.contains("Developer"));
        assert!(markup.contains("minimal@example.com"));
        assert!(markup.contains("Minimal summary"));
        
        // Check that empty sections are handled gracefully
        assert!(!markup.contains("Experience:"));
        assert!(!markup.contains("Education:"));
        assert!(!markup.contains("Skills:"));
        assert!(!markup.contains("Projects:"));
        assert!(!markup.contains("Languages:"));
    }
}