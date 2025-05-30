use anyhow::{Context, Result};
use im::HashMap;
use serde::{Deserialize, Serialize};
use std::fs;

/// Represents a mapping of language names to their corresponding icons
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LanguageIcons(pub HashMap<String, String>);

impl LanguageIcons {
    /// Load language icons from a JSON file
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the JSON file containing language icons
    ///
    /// # Returns
    ///
    /// A Result containing the parsed language icons or an error
    pub fn from_json(path: &str) -> Result<Self> {
        let data = fs::read_to_string(path)
            .with_context(|| format!("Failed to read language icons from {}", path))?;

        Self::from_json_str(&data, path)
    }

    /// Load language icons from a JSON string
    ///
    /// # Arguments
    ///
    /// * `json_str` - JSON string containing language icons
    /// * `source` - Source identifier for error messages
    ///
    /// # Returns
    ///
    /// A Result containing the parsed language icons or an error
    pub fn from_json_str(json_str: &str, source: &str) -> Result<Self> {
        serde_json::from_str(json_str)
            .with_context(|| format!("Failed to parse language icons from {}", source))
    }

    /// Get the icon for a language
    ///
    /// # Arguments
    ///
    /// * `language` - The language to get the icon for
    ///
    /// # Returns
    ///
    /// The icon for the language, or an empty string if not found
    pub fn get_icon(&self, language: &str) -> &str {
        let normalized_lang = language.to_lowercase();

        // Direct match
        if let Some(icon) = self.0.get(&normalized_lang) {
            return icon;
        }

        // Check for partial matches (e.g., "scala3" should match "scala")
        for (key, value) in self.0.iter() {
            if normalized_lang.contains(key) {
                return value;
            }
        }

        // Default icon for unknown languages
        ""
    }

    /// Detect the primary language for a project
    ///
    /// # Arguments
    ///
    /// * `project_name` - The project name, which may contain language information
    /// * `technologies` - A list of technologies used in the project
    ///
    /// # Returns
    ///
    /// The detected language, or None if no language could be detected
    pub fn detect_language(&self, project_name: &str, technologies: &[String]) -> Option<String> {
        let normalized_name = project_name.to_lowercase();

        // Collect all language keys and sort by length (longest first)
        let mut lang_keys: Vec<String> = self.0.keys().cloned().collect();
        lang_keys.sort_by_key(|b| std::cmp::Reverse(b.len()));

        // Check project name for language hints, prioritizing longer language names
        for lang in &lang_keys {
            // Check if the language name is a whole word
            if normalized_name == *lang {
                return Some(lang.clone());
            }

            // Check if the language name is part of a compound word
            // Only match if it's a word boundary or part of a compound word with hyphens/underscores
            let lang_pattern = format!(
                "\\b{}\\b|\\b{}-|-{}\\b|\\b{}_|_{}\\b",
                lang, lang, lang, lang, lang
            );
            if regex::Regex::new(&lang_pattern)
                .ok()
                .filter(|re| re.is_match(&normalized_name))
                .is_some()
            {
                return Some(lang.clone());
            }
        }

        // Check technologies for language hints, prioritizing longer language names
        for tech in technologies {
            let normalized_tech = tech.to_lowercase();

            // Direct match with a technology
            if let Some(lang) = lang_keys.iter().find(|&lang| normalized_tech == *lang) {
                return Some(lang.clone());
            }

            // Check if the technology contains a language name at word boundaries
            for lang in &lang_keys {
                let lang_pattern = format!(
                    "\\b{}\\b|\\b{}-|-{}\\b|\\b{}_|_{}\\b",
                    lang, lang, lang, lang, lang
                );
                if regex::Regex::new(&lang_pattern)
                    .ok()
                    .filter(|re| re.is_match(&normalized_tech))
                    .is_some()
                {
                    return Some(lang.clone());
                }
            }
        }

        None
    }

    /// Detect the primary language for a project using a Vector of technologies
    ///
    /// # Arguments
    ///
    /// * `project_name` - The project name, which may contain language information
    /// * `technologies` - A Vector of technologies used in the project
    ///
    /// # Returns
    ///
    /// The detected language, or None if no language could be detected
    pub fn detect_language_vector(
        &self,
        project_name: &str,
        technologies: &im::Vector<String>,
    ) -> Option<String> {
        // Convert technologies to a slice and delegate to the existing method
        let tech_slice: Vec<String> = technologies.iter().cloned().collect();
        self.detect_language(project_name, &tech_slice)
    }
}
