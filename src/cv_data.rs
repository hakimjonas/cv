use anyhow::{Context, Result};
use im::Vector;
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
    pub social_links: im::HashMap<String, String>,
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
    /// ```
    /// use cv::cv_data::Cv;
    /// let cv = Cv::from_json("data/cv_data.json").expect("Failed to load CV data");
    /// ```
    pub fn from_json(path: &str) -> Result<Self> {
        let data = fs::read_to_string(path)
            .with_context(|| format!("Failed to read CV data from {}", path))?;

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
        serde_json::from_str(json_str)
            .with_context(|| format!("Failed to parse CV data from {}", source))
    }
}
