#[cfg(test)]
mod tests {
    use crate::cv_data::{Cv, Education, Experience, GitHubSource, PersonalInfo, Project, SkillCategory};
    use anyhow::Result;
    use im::{HashMap, Vector};
    use std::fs;
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

    // Test serializing and deserializing a CV to/from JSON
    #[test]
    fn test_cv_json_roundtrip() -> Result<()> {
        let cv = create_test_cv();
        
        // Serialize to JSON
        let json = serde_json::to_string_pretty(&cv)?;
        
        // Deserialize from JSON
        let deserialized_cv: Cv = serde_json::from_str(&json)?;
        
        // Check that the deserialized CV matches the original
        assert_eq!(deserialized_cv.personal_info.name, cv.personal_info.name);
        assert_eq!(deserialized_cv.personal_info.email, cv.personal_info.email);
        assert_eq!(deserialized_cv.experiences.len(), cv.experiences.len());
        assert_eq!(deserialized_cv.education.len(), cv.education.len());
        assert_eq!(deserialized_cv.skill_categories.len(), cv.skill_categories.len());
        assert_eq!(deserialized_cv.projects.len(), cv.projects.len());
        assert_eq!(deserialized_cv.languages.len(), cv.languages.len());
        assert_eq!(deserialized_cv.certifications.len(), cv.certifications.len());
        assert_eq!(deserialized_cv.github_sources.len(), cv.github_sources.len());
        
        Ok(())
    }

    // Test loading a CV from a JSON file
    #[test]
    fn test_cv_from_json_file() -> Result<()> {
        // Create a temporary directory for the test
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("test_cv.json");
        
        // Create a test CV and serialize it to JSON
        let cv = create_test_cv();
        let json = serde_json::to_string_pretty(&cv)?;
        
        // Write the JSON to a file
        fs::write(&file_path, json)?;
        
        // Load the CV from the file
        let loaded_cv = Cv::from_json(file_path.to_str().unwrap())?;
        
        // Check that the loaded CV matches the original
        assert_eq!(loaded_cv.personal_info.name, cv.personal_info.name);
        assert_eq!(loaded_cv.personal_info.email, cv.personal_info.email);
        assert_eq!(loaded_cv.experiences.len(), cv.experiences.len());
        assert_eq!(loaded_cv.education.len(), cv.education.len());
        assert_eq!(loaded_cv.skill_categories.len(), cv.skill_categories.len());
        assert_eq!(loaded_cv.projects.len(), cv.projects.len());
        assert_eq!(loaded_cv.languages.len(), cv.languages.len());
        assert_eq!(loaded_cv.certifications.len(), cv.certifications.len());
        assert_eq!(loaded_cv.github_sources.len(), cv.github_sources.len());
        
        Ok(())
    }

    // Test loading a CV from a JSON string
    #[test]
    fn test_cv_from_json_str() -> Result<()> {
        // Create a test CV and serialize it to JSON
        let cv = create_test_cv();
        let json = serde_json::to_string_pretty(&cv)?;
        
        // Load the CV from the JSON string
        let loaded_cv = Cv::from_json_str(&json, "test")?;
        
        // Check that the loaded CV matches the original
        assert_eq!(loaded_cv.personal_info.name, cv.personal_info.name);
        assert_eq!(loaded_cv.personal_info.email, cv.personal_info.email);
        assert_eq!(loaded_cv.experiences.len(), cv.experiences.len());
        assert_eq!(loaded_cv.education.len(), cv.education.len());
        assert_eq!(loaded_cv.skill_categories.len(), cv.skill_categories.len());
        assert_eq!(loaded_cv.projects.len(), cv.projects.len());
        assert_eq!(loaded_cv.languages.len(), cv.languages.len());
        assert_eq!(loaded_cv.certifications.len(), cv.certifications.len());
        assert_eq!(loaded_cv.github_sources.len(), cv.github_sources.len());
        
        Ok(())
    }

    // Test error handling for invalid JSON
    #[test]
    fn test_cv_from_invalid_json() {
        let invalid_json = r#"{ "personal_info": { "name": "Test User" }"#; // Missing closing brace
        
        let result = Cv::from_json_str(invalid_json, "test");
        assert!(result.is_err());
        
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Failed to parse CV data from test"));
    }

    // Test error handling for missing file
    #[test]
    fn test_cv_from_missing_file() {
        let result = Cv::from_json("nonexistent_file.json");
        assert!(result.is_err());
        
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Failed to read CV data from nonexistent_file.json"));
    }

    // Test validation of CV data structure
    #[test]
    fn test_cv_structure() {
        let cv = create_test_cv();
        
        // Test personal_info
        assert!(!cv.personal_info.name.is_empty());
        assert!(!cv.personal_info.email.is_empty());
        assert!(!cv.personal_info.title.is_empty());
        assert!(!cv.personal_info.summary.is_empty());
        
        // Test experiences
        assert!(!cv.experiences.is_empty());
        for exp in cv.experiences.iter() {
            assert!(!exp.company.is_empty());
            assert!(!exp.position.is_empty());
            assert!(!exp.start_date.is_empty());
            assert!(!exp.description.is_empty());
        }
        
        // Test education
        assert!(!cv.education.is_empty());
        for edu in cv.education.iter() {
            assert!(!edu.institution.is_empty());
            assert!(!edu.degree.is_empty());
            assert!(!edu.field.is_empty());
            assert!(!edu.start_date.is_empty());
        }
        
        // Test skill_categories
        assert!(!cv.skill_categories.is_empty());
        for cat in cv.skill_categories.iter() {
            assert!(!cat.name.is_empty());
            assert!(!cat.skills.is_empty());
        }
        
        // Test projects
        assert!(!cv.projects.is_empty());
        for proj in cv.projects.iter() {
            assert!(!proj.name.is_empty());
            assert!(!proj.description.is_empty());
        }
        
        // Test languages
        assert!(!cv.languages.is_empty());
        
        // Test github_sources
        assert!(!cv.github_sources.is_empty());
        for source in cv.github_sources.iter() {
            assert!(source.username.is_some() || source.organization.is_some());
        }
    }

    // Test deep equality of CV data
    #[test]
    fn test_cv_deep_equality() -> Result<()> {
        let cv1 = create_test_cv();
        let json = serde_json::to_string(&cv1)?;
        let cv2: Cv = serde_json::from_str(&json)?;
        
        // Check personal_info
        assert_eq!(cv1.personal_info.name, cv2.personal_info.name);
        assert_eq!(cv1.personal_info.title, cv2.personal_info.title);
        assert_eq!(cv1.personal_info.email, cv2.personal_info.email);
        assert_eq!(cv1.personal_info.phone, cv2.personal_info.phone);
        assert_eq!(cv1.personal_info.website, cv2.personal_info.website);
        assert_eq!(cv1.personal_info.location, cv2.personal_info.location);
        assert_eq!(cv1.personal_info.summary, cv2.personal_info.summary);
        assert_eq!(cv1.personal_info.social_links.len(), cv2.personal_info.social_links.len());
        
        // Check experiences
        assert_eq!(cv1.experiences.len(), cv2.experiences.len());
        for (exp1, exp2) in cv1.experiences.iter().zip(cv2.experiences.iter()) {
            assert_eq!(exp1.company, exp2.company);
            assert_eq!(exp1.position, exp2.position);
            assert_eq!(exp1.start_date, exp2.start_date);
            assert_eq!(exp1.end_date, exp2.end_date);
            assert_eq!(exp1.location, exp2.location);
            assert_eq!(exp1.description, exp2.description);
            assert_eq!(exp1.achievements.len(), exp2.achievements.len());
            assert_eq!(exp1.technologies.len(), exp2.technologies.len());
        }
        
        // Check education
        assert_eq!(cv1.education.len(), cv2.education.len());
        for (edu1, edu2) in cv1.education.iter().zip(cv2.education.iter()) {
            assert_eq!(edu1.institution, edu2.institution);
            assert_eq!(edu1.degree, edu2.degree);
            assert_eq!(edu1.field, edu2.field);
            assert_eq!(edu1.start_date, edu2.start_date);
            assert_eq!(edu1.end_date, edu2.end_date);
            assert_eq!(edu1.location, edu2.location);
            assert_eq!(edu1.gpa, edu2.gpa);
            assert_eq!(edu1.achievements.len(), edu2.achievements.len());
        }
        
        // Check skill_categories
        assert_eq!(cv1.skill_categories.len(), cv2.skill_categories.len());
        for (cat1, cat2) in cv1.skill_categories.iter().zip(cv2.skill_categories.iter()) {
            assert_eq!(cat1.name, cat2.name);
            assert_eq!(cat1.skills.len(), cat2.skills.len());
        }
        
        // Check projects
        assert_eq!(cv1.projects.len(), cv2.projects.len());
        for (proj1, proj2) in cv1.projects.iter().zip(cv2.projects.iter()) {
            assert_eq!(proj1.name, proj2.name);
            assert_eq!(proj1.description, proj2.description);
            assert_eq!(proj1.url, proj2.url);
            assert_eq!(proj1.repository, proj2.repository);
            assert_eq!(proj1.technologies.len(), proj2.technologies.len());
            assert_eq!(proj1.highlights.len(), proj2.highlights.len());
            assert_eq!(proj1.stars, proj2.stars);
            assert_eq!(proj1.owner_username, proj2.owner_username);
            assert_eq!(proj1.owner_avatar, proj2.owner_avatar);
        }
        
        // Check languages
        assert_eq!(cv1.languages.len(), cv2.languages.len());
        
        // Check certifications
        assert_eq!(cv1.certifications.len(), cv2.certifications.len());
        
        // Check github_sources
        assert_eq!(cv1.github_sources.len(), cv2.github_sources.len());
        for (source1, source2) in cv1.github_sources.iter().zip(cv2.github_sources.iter()) {
            assert_eq!(source1.username, source2.username);
            assert_eq!(source1.organization, source2.organization);
        }
        
        Ok(())
    }
}