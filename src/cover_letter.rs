//! Cover letter loading
//!
//! Loads cover letter content from markdown files with optional YAML frontmatter.
//!
//! # Format
//!
//! ```markdown
//! ---
//! company: "Acme Corp"
//! position: "Software Engineer"
//! recipient_name: "Jane Smith"
//! recipient_title: "Engineering Manager"
//! ---
//!
//! Dear Jane Smith,
//!
//! [Body paragraphs...]
//!
//! Best regards,
//! ```

use anyhow::{Context, Result};
use gray_matter::Matter;
use serde::Deserialize;
use std::fs;
use std::path::Path;

/// YAML frontmatter for cover letter metadata
#[derive(Debug, Default, Deserialize)]
pub struct CoverLetterFrontMatter {
    /// Company name
    pub company: Option<String>,
    /// Position title
    pub position: Option<String>,
    /// Recipient's name
    pub recipient_name: Option<String>,
    /// Recipient's title
    pub recipient_title: Option<String>,
    /// Company address
    pub address: Option<String>,
}

/// Represents a cover letter with optional metadata
#[derive(Debug)]
pub struct CoverLetter {
    /// Optional frontmatter metadata
    pub frontmatter: CoverLetterFrontMatter,
    /// Letter body content (plain text)
    pub body: String,
}

impl CoverLetter {
    /// Load a cover letter from a file
    pub fn from_file(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read cover letter: {}", path.display()))?;

        Self::parse(&content)
    }

    /// Parse cover letter content with optional frontmatter
    fn parse(content: &str) -> Result<Self> {
        let matter = Matter::<gray_matter::engine::YAML>::new();
        let parsed = matter
            .parse::<CoverLetterFrontMatter>(content)
            .context("Failed to parse cover letter")?;

        let frontmatter = parsed.data.unwrap_or_default();
        let body = parsed.content.trim().to_string();

        Ok(CoverLetter { frontmatter, body })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_cover_letter_without_frontmatter() {
        let content = "Dear Hiring Manager,\n\nI am interested in the position.\n\nBest regards,";

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(content.as_bytes()).unwrap();

        let letter = CoverLetter::from_file(temp_file.path()).unwrap();
        assert!(letter.body.contains("Dear Hiring Manager"));
        assert!(letter.body.contains("interested in the position"));
        assert!(letter.frontmatter.company.is_none());
    }

    #[test]
    fn test_load_cover_letter_with_frontmatter() {
        let content = r#"---
company: "Acme Corp"
position: "Software Engineer"
recipient_name: "Jane Smith"
recipient_title: "Engineering Manager"
---

Dear Jane Smith,

I am interested in the position.

Best regards,"#;

        let letter = CoverLetter::parse(content).unwrap();
        assert_eq!(letter.frontmatter.company, Some("Acme Corp".to_string()));
        assert_eq!(
            letter.frontmatter.position,
            Some("Software Engineer".to_string())
        );
        assert_eq!(
            letter.frontmatter.recipient_name,
            Some("Jane Smith".to_string())
        );
        assert_eq!(
            letter.frontmatter.recipient_title,
            Some("Engineering Manager".to_string())
        );
        assert!(letter.body.contains("Dear Jane Smith"));
    }
}
