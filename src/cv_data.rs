use anyhow::{Context, Result};
use im::Vector;
use serde::{Deserialize, Serialize};
use std::{env, fs};

/// Represents personal information in a CV
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PersonalInfo {
    pub name: String,
    pub title: String,
    pub email: String,
    pub phone: Option<String>,
    pub website: Option<String>,
    pub location: Option<String>,
    pub summary: String,
    pub social_links: im::HashMap<String, String>,
    /// Optional custom profile image (will use GitHub avatar if not provided)
    pub profile_image: Option<String>,
    /// GitHub avatar URL (automatically fetched)
    #[serde(skip)]
    pub github_avatar_url: Option<String>,
}

/// Represents a work experience entry
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Experience {
    pub company: String,
    pub position: String,
    pub start_date: String,
    pub end_date: Option<String>,
    pub location: Option<String>,
    pub description: String,
    pub achievements: Vector<String>,
    pub technologies: Vector<String>,
}

/// Represents an education entry
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Education {
    pub institution: String,
    pub degree: String,
    pub field: String,
    pub start_date: String,
    pub end_date: Option<String>,
    pub location: Option<String>,
    pub gpa: Option<String>,
    pub achievements: Vector<String>,
}

/// Represents a skill category and its skills
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SkillCategory {
    pub name: String,
    pub skills: Vector<String>,
}

/// Represents a project
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Project {
    pub name: String,
    pub description: String,
    pub url: Option<String>,
    pub repository: Option<String>,
    pub technologies: Vector<String>,
    pub highlights: Vector<String>,
    pub stars: Option<u32>,
    pub owner_username: Option<String>,
    pub owner_avatar: Option<String>,
    #[serde(skip)]
    pub language: Option<String>,
    #[serde(skip)]
    pub language_icon: Option<String>,
    #[serde(skip)]
    pub display_name: Option<String>,
}

/// Represents GitHub sources to fetch projects from
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GitHubSource {
    pub username: Option<String>,
    pub organization: Option<String>,
}

/// Top-level CV structure that contains all CV data
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Cv {
    pub personal_info: PersonalInfo,
    pub experiences: Vector<Experience>,
    pub education: Vector<Education>,
    pub skill_categories: Vector<SkillCategory>,
    pub projects: Vector<Project>,
    pub languages: im::HashMap<String, String>,
    pub certifications: Vector<String>,
    #[serde(default)]
    pub github_sources: Vector<GitHubSource>,
}

impl Cv {
    /// Load CV data from a JSON file
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the JSON file containing CV data
    ///
    /// # Returns
    ///
    /// A Result containing the parsed CV data or an error
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use cv_generator::cv_data::Cv;
    /// let cv = Cv::from_json("data/cv_data.json").expect("Failed to load CV data");
    /// ```
    pub fn from_json(path: &str) -> Result<Self> {
        let data = fs::read_to_string(path).with_context(|| {
            format!(
                "Failed to read CV data file: {}\n\
                                      \n\
                                      Please ensure:\n\
                                      - The file exists at the specified path\n\
                                      - You have read permissions\n\
                                      - Current directory: {}",
                path,
                env::current_dir().unwrap_or_default().display()
            )
        })?;

        Self::from_json_str(&data, path)
    }

    /// Load CV data from a JSON string
    ///
    /// # Arguments
    ///
    /// * `json_str` - JSON string containing CV data
    /// * `source` - Source identifier for error messages
    ///
    /// # Returns
    ///
    /// A Result containing the parsed CV data or an error
    pub fn from_json_str(json_str: &str, source: &str) -> Result<Self> {
        serde_json::from_str(json_str).with_context(|| {
            format!(
                "Failed to parse CV data from: {}\n\
                                      \n\
                                      JSON parsing error. Please check:\n\
                                      - Valid JSON syntax (use https://jsonlint.com/ to validate)\n\
                                      - Required fields are present (name, title, email, summary)\n\
                                      - See example structure in README.md or data/cv_data.json",
                source
            )
        })
    }

    /// Create a minimal CV data structure for testing
    #[cfg(test)]
    pub fn create_minimal() -> Self {
        Cv {
            personal_info: PersonalInfo {
                name: "Test User".to_string(),
                title: "Software Developer".to_string(),
                email: "test@example.com".to_string(),
                phone: Some("+1234567890".to_string()),
                website: Some("https://example.com".to_string()),
                location: Some("Test City".to_string()),
                summary: "Test summary".to_string(),
                social_links: im::HashMap::new(),
                profile_image: None,
                github_avatar_url: None,
            },
            experiences: Vector::new(),
            education: Vector::new(),
            skill_categories: Vector::new(),
            projects: Vector::new(),
            languages: im::HashMap::new(),
            certifications: Vector::new(),
            github_sources: Vector::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_cv_creation_minimal() {
        let cv = Cv::create_minimal();

        assert_eq!(cv.personal_info.name, "Test User");
        assert_eq!(cv.personal_info.title, "Software Developer");
        assert_eq!(cv.personal_info.email, "test@example.com");
        assert!(cv.experiences.is_empty());
        assert!(cv.education.is_empty());
        assert!(cv.skill_categories.is_empty());
        assert!(cv.projects.is_empty());
        assert!(cv.certifications.is_empty());
    }

    #[test]
    fn test_cv_from_json_str_valid() {
        let json_data = r#"{
            "personal_info": {
                "name": "John Doe",
                "title": "Senior Developer",
                "email": "john@example.com",
                "summary": "Experienced developer",
                "social_links": {}
            },
            "experiences": [],
            "education": [],
            "skill_categories": [],
            "projects": [],
            "languages": {},
            "certifications": []
        }"#;

        let cv = Cv::from_json_str(json_data, "test").unwrap();
        assert_eq!(cv.personal_info.name, "John Doe");
        assert_eq!(cv.personal_info.title, "Senior Developer");
        assert_eq!(cv.personal_info.email, "john@example.com");
    }

    #[test]
    fn test_cv_from_json_str_invalid() {
        let invalid_json = r#"{ "invalid": json }"#;

        let result = Cv::from_json_str(invalid_json, "test");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to parse CV data from: test"));
    }

    #[test]
    fn test_cv_from_json_str_missing_required_field() {
        let json_missing_field = r#"{
            "personal_info": {
                "name": "John Doe"
            }
        }"#;

        let result = Cv::from_json_str(json_missing_field, "test");
        assert!(result.is_err());
    }

    #[test]
    fn test_cv_from_json_file() {
        let json_data = r#"{
            "personal_info": {
                "name": "Jane Smith",
                "title": "Product Manager",
                "email": "jane@example.com",
                "summary": "Strategic product manager",
                "social_links": {}
            },
            "experiences": [],
            "education": [],
            "skill_categories": [],
            "projects": [],
            "languages": {},
            "certifications": []
        }"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(json_data.as_bytes()).unwrap();

        let cv = Cv::from_json(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(cv.personal_info.name, "Jane Smith");
        assert_eq!(cv.personal_info.title, "Product Manager");
    }

    #[test]
    fn test_cv_from_json_file_not_found() {
        let result = Cv::from_json("nonexistent_file.json");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to read CV data file: nonexistent_file.json"));
    }

    #[test]
    fn test_personal_info_serialization() {
        let mut social_links = im::HashMap::new();
        social_links.insert("github".to_string(), "https://github.com/user".to_string());
        social_links.insert(
            "linkedin".to_string(),
            "https://linkedin.com/in/user".to_string(),
        );

        let personal_info = PersonalInfo {
            name: "Test User".to_string(),
            title: "Developer".to_string(),
            email: "test@example.com".to_string(),
            phone: Some("+1234567890".to_string()),
            website: Some("https://example.com".to_string()),
            location: Some("Test City".to_string()),
            summary: "Test summary".to_string(),
            social_links,
            profile_image: Some("profile.jpg".to_string()),
            github_avatar_url: Some("https://github.com/avatar.jpg".to_string()),
        };

        let json = serde_json::to_string(&personal_info).unwrap();
        let deserialized: PersonalInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(personal_info.name, deserialized.name);
        assert_eq!(personal_info.title, deserialized.title);
        assert_eq!(
            personal_info.social_links.len(),
            deserialized.social_links.len()
        );
        // github_avatar_url should be skipped during serialization
        assert!(deserialized.github_avatar_url.is_none());
    }

    #[test]
    fn test_experience_creation() {
        let experience = Experience {
            company: "Tech Corp".to_string(),
            position: "Senior Engineer".to_string(),
            start_date: "2020-01".to_string(),
            end_date: Some("2023-12".to_string()),
            location: Some("Remote".to_string()),
            description: "Developed scalable systems".to_string(),
            achievements: Vector::from(vec![
                "Led team of 5".to_string(),
                "Improved performance by 50%".to_string(),
            ]),
            technologies: Vector::from(vec!["Rust".to_string(), "TypeScript".to_string()]),
        };

        assert_eq!(experience.company, "Tech Corp");
        assert_eq!(experience.achievements.len(), 2);
        assert_eq!(experience.technologies.len(), 2);
    }

    #[test]
    fn test_project_creation() {
        let project = Project {
            name: "Awesome Project".to_string(),
            description: "A really cool project".to_string(),
            url: Some("https://project.com".to_string()),
            repository: Some("https://github.com/user/project".to_string()),
            technologies: Vector::from(vec!["Rust".to_string(), "WebAssembly".to_string()]),
            highlights: Vector::from(vec!["First in category".to_string()]),
            stars: Some(150),
            owner_username: Some("user".to_string()),
            owner_avatar: Some("https://github.com/user.jpg".to_string()),
            language: Some("Rust".to_string()),
            language_icon: Some("🦀".to_string()),
            display_name: Some("awesome-project".to_string()),
        };

        assert_eq!(project.name, "Awesome Project");
        assert_eq!(project.stars, Some(150));
        assert_eq!(project.technologies.len(), 2);
        assert_eq!(project.highlights.len(), 1);
    }

    #[test]
    fn test_project_serialization_skips_runtime_fields() {
        let project = Project {
            name: "Test Project".to_string(),
            description: "Test description".to_string(),
            url: None,
            repository: None,
            technologies: Vector::new(),
            highlights: Vector::new(),
            stars: None,
            owner_username: None,
            owner_avatar: None,
            language: Some("Rust".to_string()),
            language_icon: Some("🦀".to_string()),
            display_name: Some("test-project".to_string()),
        };

        let json = serde_json::to_string(&project).unwrap();
        let deserialized: Project = serde_json::from_str(&json).unwrap();

        assert_eq!(project.name, deserialized.name);
        assert_eq!(project.description, deserialized.description);
        // Runtime fields should be skipped
        assert!(deserialized.language.is_none());
        assert!(deserialized.language_icon.is_none());
        assert!(deserialized.display_name.is_none());
    }

    #[test]
    fn test_skill_category_creation() {
        let skill_category = SkillCategory {
            name: "Programming Languages".to_string(),
            skills: Vector::from(vec![
                "Rust".to_string(),
                "TypeScript".to_string(),
                "Python".to_string(),
            ]),
        };

        assert_eq!(skill_category.name, "Programming Languages");
        assert_eq!(skill_category.skills.len(), 3);
        assert!(skill_category.skills.contains(&"Rust".to_string()));
    }

    #[test]
    fn test_github_source_creation() {
        let github_source = GitHubSource {
            username: Some("johndoe".to_string()),
            organization: Some("awesome-org".to_string()),
        };

        assert_eq!(github_source.username, Some("johndoe".to_string()));
        assert_eq!(github_source.organization, Some("awesome-org".to_string()));

        // Test with only username
        let user_only = GitHubSource {
            username: Some("jane".to_string()),
            organization: None,
        };

        assert_eq!(user_only.username, Some("jane".to_string()));
        assert_eq!(user_only.organization, None);
    }

    #[test]
    fn test_education_creation() {
        let education = Education {
            institution: "University of Technology".to_string(),
            degree: "Bachelor of Science".to_string(),
            field: "Computer Science".to_string(),
            start_date: "2016-09".to_string(),
            end_date: Some("2020-05".to_string()),
            location: Some("Tech City".to_string()),
            gpa: Some("3.8".to_string()),
            achievements: Vector::from(vec![
                "Magna Cum Laude".to_string(),
                "Dean's List".to_string(),
            ]),
        };

        assert_eq!(education.institution, "University of Technology");
        assert_eq!(education.degree, "Bachelor of Science");
        assert_eq!(education.field, "Computer Science");
        assert_eq!(education.gpa, Some("3.8".to_string()));
        assert_eq!(education.achievements.len(), 2);
    }
}
