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
            .with_context(|| format!("Failed to read language icons from {path}"))?;

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
            .with_context(|| format!("Failed to parse language icons from {source}"))
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
        // Collect language keys, longest first, so a longer key (e.g.
        // "typescript") wins over a shorter substring ("script") when both
        // appear as tokens.
        let mut lang_keys: Vec<String> = self.0.keys().cloned().collect();
        lang_keys.sort_by_key(|b| std::cmp::Reverse(b.len()));

        // A language matches only as a *whole token*: the text is split on
        // any non-alphanumeric boundary and each piece compared exactly.
        // This is why "rumil-dart" -> dart but "dart3" does not, and why a
        // single-letter key like "c" never spuriously matches inside a word
        // such as "calculus-of-constructions".
        let matches_as_token = |text: &str, lang: &str| -> bool {
            text.to_lowercase()
                .split(|c: char| !c.is_ascii_alphanumeric())
                .any(|token| token == lang)
        };

        // Project name takes priority, then technologies, each scanned with
        // languages in longest-first order.
        for lang in &lang_keys {
            if matches_as_token(project_name, lang) {
                return Some(lang.clone());
            }
        }
        for tech in technologies {
            for lang in &lang_keys {
                if matches_as_token(tech, lang) {
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
        // Convert technologies to a Vec and then get a slice to delegate to the existing method
        let tech_vec: Vec<String> = technologies.iter().cloned().collect();
        self.detect_language(project_name, &tech_vec)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn icons() -> LanguageIcons {
        // A representative subset, including the single-letter "c" key that
        // caused spurious substring matches in the old implementation.
        let json = r#"{
            "dart": "d", "rust": "r", "scala": "s", "typescript": "t",
            "c": "c", "lua": "l", "go": "g"
        }"#;
        LanguageIcons::from_json_str(json, "test").unwrap()
    }

    #[test]
    fn matches_language_token_in_hyphenated_name() {
        // "rumil-dart" splits into ["rumil", "dart"]; "dart" is a whole token.
        assert_eq!(
            icons().detect_language("rumil-dart", &[]),
            Some("dart".into())
        );
    }

    #[test]
    fn does_not_match_language_as_substring_of_a_token() {
        // The root-cause regression: "c" must NOT match inside
        // "calculus-of-constructions" (a tech tag on the Doxa card).
        let techs = vec!["calculus-of-constructions".to_string()];
        assert_eq!(icons().detect_language("doxa", &techs), None);
    }

    #[test]
    fn does_not_match_when_key_is_a_strict_substring() {
        // "dart3" is one token that is not equal to "dart" -> no match,
        // matching the previous behaviour for version-suffixed tokens.
        assert_eq!(icons().detect_language("dart3", &[]), None);
    }

    #[test]
    fn name_takes_priority_over_technologies() {
        // Name resolves first; technologies are only a fallback.
        let techs = vec!["rust".to_string()];
        assert_eq!(
            icons().detect_language("my-dart-lib", &techs),
            Some("dart".into())
        );
    }

    #[test]
    fn falls_back_to_technologies_when_name_has_no_language() {
        let techs = vec!["Rust".to_string()];
        assert_eq!(icons().detect_language("fin", &techs), Some("rust".into()));
    }

    #[test]
    fn prefers_longer_language_key_when_both_are_tokens() {
        // "typescript" must win over a hypothetical shorter overlap.
        assert_eq!(
            icons().detect_language("typescript", &[]),
            Some("typescript".into())
        );
    }
}
