/// CV domain models
///
/// This module contains all CV-related domain entities.
/// Uses immutable data structures throughout for functional programming approach.
use anyhow::{Context, Result};
use im::{HashMap, Vector};
use serde::{Deserialize, Serialize};
use std::fs;

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
    pub social_links: HashMap<String, String>,
}

impl PersonalInfo {
    /// Validates personal information
    pub fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty() {
            return Err(anyhow::anyhow!("Name cannot be empty"));
        }
        if self.title.trim().is_empty() {
            return Err(anyhow::anyhow!("Title cannot be empty"));
        }
        if self.email.trim().is_empty() {
            return Err(anyhow::anyhow!("Email cannot be empty"));
        }
        if !self.email.contains('@') {
            return Err(anyhow::anyhow!("Invalid email format"));
        }
        Ok(())
    }
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

impl Experience {
    /// Validates experience entry
    pub fn validate(&self) -> Result<()> {
        if self.company.trim().is_empty() {
            return Err(anyhow::anyhow!("Company cannot be empty"));
        }
        if self.position.trim().is_empty() {
            return Err(anyhow::anyhow!("Position cannot be empty"));
        }
        if self.start_date.trim().is_empty() {
            return Err(anyhow::anyhow!("Start date cannot be empty"));
        }
        // Validate date format
        chrono::NaiveDate::parse_from_str(&self.start_date, "%Y-%m-%d")
            .or_else(|_| chrono::NaiveDate::parse_from_str(&self.start_date, "%Y-%m"))
            .context("Invalid start date format")?;

        if let Some(ref end_date) = self.end_date {
            chrono::NaiveDate::parse_from_str(end_date, "%Y-%m-%d")
                .or_else(|_| chrono::NaiveDate::parse_from_str(end_date, "%Y-%m"))
                .context("Invalid end date format")?;
        }
        Ok(())
    }

    /// Check if this is a current position
    pub fn is_current(&self) -> bool {
        self.end_date.is_none()
    }

    /// Get duration of employment
    pub fn duration(&self) -> String {
        match &self.end_date {
            Some(end) => format!("{} - {}", self.start_date, end),
            None => format!("{} - Present", self.start_date),
        }
    }
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

impl Education {
    /// Validates education entry
    pub fn validate(&self) -> Result<()> {
        if self.institution.trim().is_empty() {
            return Err(anyhow::anyhow!("Institution cannot be empty"));
        }
        if self.degree.trim().is_empty() {
            return Err(anyhow::anyhow!("Degree cannot be empty"));
        }
        if self.field.trim().is_empty() {
            return Err(anyhow::anyhow!("Field cannot be empty"));
        }
        Ok(())
    }
}

/// Represents a skill category and its skills
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SkillCategory {
    pub name: String,
    pub skills: Vector<String>,
}

impl SkillCategory {
    /// Creates a new skill category
    pub fn new(name: &str, skills: Vector<String>) -> Self {
        Self {
            name: name.to_string(),
            skills,
        }
    }

    /// Validates skill category
    pub fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty() {
            return Err(anyhow::anyhow!("Skill category name cannot be empty"));
        }
        if self.skills.is_empty() {
            return Err(anyhow::anyhow!(
                "Skill category must have at least one skill"
            ));
        }
        Ok(())
    }
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

    // Computed fields (not serialized in JSON)
    #[serde(skip)]
    pub language: Option<String>,
    #[serde(skip)]
    pub language_icon: Option<String>,
    #[serde(skip)]
    pub display_name: Option<String>,
}

impl Project {
    /// Creates a new project
    pub fn new(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            url: None,
            repository: None,
            technologies: Vector::new(),
            highlights: Vector::new(),
            stars: None,
            owner_username: None,
            owner_avatar: None,
            language: None,
            language_icon: None,
            display_name: None,
        }
    }

    /// Validates project
    pub fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty() {
            return Err(anyhow::anyhow!("Project name cannot be empty"));
        }
        if self.description.trim().is_empty() {
            return Err(anyhow::anyhow!("Project description cannot be empty"));
        }
        Ok(())
    }

    /// Check if this is a GitHub project
    pub fn is_github_project(&self) -> bool {
        self.repository.is_some()
    }

    /// Get the effective display name
    pub fn effective_display_name(&self) -> &str {
        self.display_name.as_deref().unwrap_or(&self.name)
    }
}

/// Represents GitHub sources to fetch projects from
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GitHubSource {
    pub username: Option<String>,
    pub organization: Option<String>,
}

impl GitHubSource {
    /// Creates a new GitHub source for a user
    pub fn user(username: &str) -> Self {
        Self {
            username: Some(username.to_string()),
            organization: None,
        }
    }

    /// Creates a new GitHub source for an organization
    pub fn organization(org: &str) -> Self {
        Self {
            username: None,
            organization: Some(org.to_string()),
        }
    }

    /// Validates GitHub source
    pub fn validate(&self) -> Result<()> {
        if self.username.is_none() && self.organization.is_none() {
            return Err(anyhow::anyhow!(
                "GitHub source must have either username or organization"
            ));
        }
        if self.username.is_some() && self.organization.is_some() {
            return Err(anyhow::anyhow!(
                "GitHub source cannot have both username and organization"
            ));
        }
        Ok(())
    }

    /// Get the source identifier for API calls
    pub fn identifier(&self) -> Option<&str> {
        self.username.as_deref().or(self.organization.as_deref())
    }

    /// Check if this is a user source
    pub fn is_user(&self) -> bool {
        self.username.is_some()
    }

    /// Check if this is an organization source
    pub fn is_organization(&self) -> bool {
        self.organization.is_some()
    }
}

/// Top-level CV structure that contains all CV data
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Cv {
    pub personal_info: PersonalInfo,
    pub experiences: Vector<Experience>,
    pub education: Vector<Education>,
    pub skill_categories: Vector<SkillCategory>,
    pub projects: Vector<Project>,
    pub languages: HashMap<String, String>,
    pub certifications: Vector<String>,
    #[serde(default)]
    pub github_sources: Vector<GitHubSource>,
}

impl Cv {
    /// Load CV data from a JSON file
    pub fn from_json(path: &str) -> Result<Self> {
        let data = fs::read_to_string(path)
            .with_context(|| format!("Failed to read CV data from {}", path))?;

        Self::from_json_str(&data, path)
    }

    /// Load CV data from a JSON string
    pub fn from_json_str(json_str: &str, source: &str) -> Result<Self> {
        let cv: Self = serde_json::from_str(json_str)
            .with_context(|| format!("Failed to parse CV data from {}", source))?;

        cv.validate()
            .with_context(|| format!("CV data validation failed for {}", source))?;

        Ok(cv)
    }

    /// Validate the entire CV
    pub fn validate(&self) -> Result<()> {
        self.personal_info
            .validate()
            .context("Personal info validation failed")?;

        for (i, experience) in self.experiences.iter().enumerate() {
            experience
                .validate()
                .with_context(|| format!("Experience {} validation failed", i + 1))?;
        }

        for (i, education) in self.education.iter().enumerate() {
            education
                .validate()
                .with_context(|| format!("Education {} validation failed", i + 1))?;
        }

        for (i, category) in self.skill_categories.iter().enumerate() {
            category
                .validate()
                .with_context(|| format!("Skill category {} validation failed", i + 1))?;
        }

        for (i, project) in self.projects.iter().enumerate() {
            project
                .validate()
                .with_context(|| format!("Project {} validation failed", i + 1))?;
        }

        for (i, source) in self.github_sources.iter().enumerate() {
            source
                .validate()
                .with_context(|| format!("GitHub source {} validation failed", i + 1))?;
        }

        Ok(())
    }

    /// Get all technologies used across experiences and projects
    pub fn all_technologies(&self) -> Vector<String> {
        let mut technologies = Vector::new();

        // Collect from experiences
        for experience in &self.experiences {
            for tech in &experience.technologies {
                if !technologies.contains(tech) {
                    technologies.push_back(tech.clone());
                }
            }
        }

        // Collect from projects
        for project in &self.projects {
            for tech in &project.technologies {
                if !technologies.contains(tech) {
                    technologies.push_back(tech.clone());
                }
            }
        }

        technologies
    }

    /// Get projects filtered by technology
    pub fn projects_by_technology(&self, technology: &str) -> Vector<&Project> {
        self.projects
            .iter()
            .filter(|project| project.technologies.contains(&technology.to_string()))
            .collect()
    }

    /// Get current experiences (those without end date)
    pub fn current_experiences(&self) -> Vector<&Experience> {
        self.experiences
            .iter()
            .filter(|exp| exp.is_current())
            .collect()
    }

    /// Get total years of experience
    pub fn total_experience_years(&self) -> f32 {
        // This is a simplified calculation - in practice you'd want more sophisticated date arithmetic
        self.experiences.len() as f32 * 1.5 // Rough estimate
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_personal_info_validation() {
        let valid_info = PersonalInfo {
            name: "John Doe".to_string(),
            title: "Software Engineer".to_string(),
            email: "john@example.com".to_string(),
            phone: None,
            website: None,
            location: None,
            summary: "Summary".to_string(),
            social_links: HashMap::new(),
        };
        assert!(valid_info.validate().is_ok());

        let invalid_info = PersonalInfo {
            name: "".to_string(),
            title: "Software Engineer".to_string(),
            email: "invalid-email".to_string(),
            phone: None,
            website: None,
            location: None,
            summary: "Summary".to_string(),
            social_links: HashMap::new(),
        };
        assert!(invalid_info.validate().is_err());
    }

    #[test]
    fn test_github_source() {
        let user_source = GitHubSource::user("johndoe");
        assert!(user_source.is_user());
        assert!(!user_source.is_organization());
        assert_eq!(user_source.identifier(), Some("johndoe"));

        let org_source = GitHubSource::organization("myorg");
        assert!(!org_source.is_user());
        assert!(org_source.is_organization());
        assert_eq!(org_source.identifier(), Some("myorg"));
    }

    #[test]
    fn test_project_creation() {
        let project = Project::new("Test Project", "A test project");
        assert_eq!(project.name, "Test Project");
        assert_eq!(project.description, "A test project");
        assert!(!project.is_github_project());
        assert!(project.validate().is_ok());
    }

    #[test]
    fn test_experience_duration() {
        let current_exp = Experience {
            company: "Company".to_string(),
            position: "Position".to_string(),
            start_date: "2023-01".to_string(),
            end_date: None,
            location: None,
            description: "Description".to_string(),
            achievements: Vector::new(),
            technologies: Vector::new(),
        };
        assert!(current_exp.is_current());
        assert_eq!(current_exp.duration(), "2023-01 - Present");

        let past_exp = Experience {
            end_date: Some("2024-01".to_string()),
            ..current_exp
        };
        assert!(!past_exp.is_current());
        assert_eq!(past_exp.duration(), "2023-01 - 2024-01");
    }
}
