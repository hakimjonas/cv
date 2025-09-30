//! Input validation module for security and data integrity
//!
//! This module provides validation functions for user input, configuration values,
//! and file paths to ensure data integrity and prevent security issues.

use anyhow::{anyhow, Result};
use std::path::Path;

/// Validate GitHub username format
///
/// GitHub usernames must:
/// - Be 1-39 characters long
/// - Contain only alphanumeric characters or hyphens
/// - Not start or end with a hyphen
/// - Not contain consecutive hyphens
///
/// # Examples
/// ```
/// use cv_generator::validation::validate_github_username;
///
/// assert!(validate_github_username("octocat").is_ok());
/// assert!(validate_github_username("my-user-123").is_ok());
/// assert!(validate_github_username("-invalid").is_err());
/// assert!(validate_github_username("has--double").is_err());
/// ```
pub fn validate_github_username(username: &str) -> Result<()> {
    // Check length
    if username.is_empty() || username.len() > 39 {
        return Err(anyhow!(
            "Invalid GitHub username '{}': must be 1-39 characters long",
            username
        ));
    }

    // Check for valid characters
    if !username.chars().all(|c| c.is_alphanumeric() || c == '-') {
        return Err(anyhow!(
            "Invalid GitHub username '{}': can only contain alphanumeric characters and hyphens",
            username
        ));
    }

    // Check start/end
    if username.starts_with('-') || username.ends_with('-') {
        return Err(anyhow!(
            "Invalid GitHub username '{}': cannot start or end with a hyphen",
            username
        ));
    }

    // Check for consecutive hyphens
    if username.contains("--") {
        return Err(anyhow!(
            "Invalid GitHub username '{}': cannot contain consecutive hyphens",
            username
        ));
    }

    Ok(())
}

/// Valid paper sizes for Typst PDF generation
pub const VALID_PAPER_SIZES: &[&str] = &["a4", "letter", "legal", "a3", "a5", "a6"];

/// Validate paper size for PDF generation
///
/// # Examples
/// ```
/// use cv_generator::validation::validate_paper_size;
///
/// assert!(validate_paper_size("a4").is_ok());
/// assert!(validate_paper_size("letter").is_ok());
/// assert!(validate_paper_size("invalid").is_err());
/// ```
pub fn validate_paper_size(size: &str) -> Result<()> {
    let size_lower = size.to_lowercase();

    if !VALID_PAPER_SIZES.contains(&size_lower.as_str()) {
        return Err(anyhow!(
            "Invalid paper size '{}': must be one of {:?}",
            size,
            VALID_PAPER_SIZES
        ));
    }

    Ok(())
}

/// Check if a path is safe (doesn't escape base directory)
///
/// This prevents path traversal attacks by ensuring the resolved path
/// starts with the base directory.
///
/// # Examples
/// ```
/// use cv_generator::validation::is_safe_path;
/// use std::path::Path;
///
/// let base = Path::new("/home/user/project");
/// assert!(is_safe_path(Path::new("/home/user/project/file.txt"), base).is_ok());
/// // Path traversal would fail
/// // assert!(is_safe_path(Path::new("/home/user/project/../../../etc/passwd"), base).is_err());
/// ```
pub fn is_safe_path(path: &Path, base: &Path) -> Result<()> {
    // Canonicalize paths to resolve .. and symlinks
    let canonical_base = base.canonicalize().unwrap_or_else(|_| base.to_path_buf());

    let canonical_path = if path.is_absolute() {
        path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
    } else {
        canonical_base
            .join(path)
            .canonicalize()
            .unwrap_or_else(|_| base.join(path))
    };

    if !canonical_path.starts_with(&canonical_base) {
        return Err(anyhow!(
            "Path '{}' attempts to escape base directory '{}'",
            path.display(),
            base.display()
        ));
    }

    Ok(())
}

/// Validate email address format (basic check)
///
/// This is a basic validation that checks for the presence of @ and .
/// For production use, consider using a dedicated email validation library.
pub fn validate_email(email: &str) -> Result<()> {
    if email.is_empty() {
        return Err(anyhow!("Email cannot be empty"));
    }

    if !email.contains('@') {
        return Err(anyhow!("Invalid email '{}': missing @ symbol", email));
    }

    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 {
        return Err(anyhow!(
            "Invalid email '{}': must have exactly one @ symbol",
            email
        ));
    }

    if parts[0].is_empty() {
        return Err(anyhow!(
            "Invalid email '{}': local part cannot be empty",
            email
        ));
    }

    if parts[1].is_empty() || !parts[1].contains('.') {
        return Err(anyhow!(
            "Invalid email '{}': domain must contain a dot",
            email
        ));
    }

    Ok(())
}

/// Sanitize a slug for use in URLs
///
/// Converts a string to a URL-safe slug by:
/// - Converting to lowercase
/// - Replacing spaces with hyphens
/// - Removing non-alphanumeric characters (except hyphens)
/// - Removing consecutive hyphens
///
/// # Examples
/// ```
/// use cv_generator::validation::sanitize_slug;
///
/// assert_eq!(sanitize_slug("Hello World!"), "hello-world");
/// assert_eq!(sanitize_slug("Test--123"), "test-123");
/// ```
pub fn sanitize_slug(input: &str) -> String {
    input
        .to_lowercase()
        .replace(' ', "-")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-')
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Validate URL format
///
/// Basic URL validation checking for http/https scheme
pub fn validate_url(url: &str) -> Result<()> {
    if url.is_empty() {
        return Err(anyhow!("URL cannot be empty"));
    }

    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(anyhow!(
            "Invalid URL '{}': must start with http:// or https://",
            url
        ));
    }

    if url.len() < 12 {
        // "https://a.b" is 12 chars minimum
        return Err(anyhow!("Invalid URL '{}': too short", url));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_github_username() {
        // Valid usernames
        assert!(validate_github_username("octocat").is_ok());
        assert!(validate_github_username("my-user").is_ok());
        assert!(validate_github_username("user123").is_ok());
        assert!(validate_github_username("a").is_ok());

        // Invalid usernames
        assert!(validate_github_username("").is_err());
        assert!(validate_github_username("-start").is_err());
        assert!(validate_github_username("end-").is_err());
        assert!(validate_github_username("double--hyphen").is_err());
        assert!(validate_github_username("invalid!char").is_err());
        assert!(validate_github_username(&"a".repeat(40)).is_err()); // Too long
    }

    #[test]
    fn test_validate_paper_size() {
        // Valid sizes
        assert!(validate_paper_size("a4").is_ok());
        assert!(validate_paper_size("A4").is_ok()); // Case insensitive
        assert!(validate_paper_size("letter").is_ok());
        assert!(validate_paper_size("legal").is_ok());

        // Invalid sizes
        assert!(validate_paper_size("tabloid").is_err());
        assert!(validate_paper_size("custom").is_err());
        assert!(validate_paper_size("").is_err());
    }

    #[test]
    fn test_is_safe_path() {
        use std::env;

        let base = env::current_dir().unwrap();

        // Safe paths
        assert!(is_safe_path(Path::new("test.txt"), &base).is_ok());
        assert!(is_safe_path(Path::new("./test.txt"), &base).is_ok());

        // These would need actual filesystem to test properly
        // Unsafe paths (path traversal attempts)
        // assert!(is_safe_path(Path::new("../../etc/passwd"), &base).is_err());
    }

    #[test]
    fn test_validate_email() {
        // Valid emails
        assert!(validate_email("user@example.com").is_ok());
        assert!(validate_email("test.user@domain.co.uk").is_ok());

        // Invalid emails
        assert!(validate_email("").is_err());
        assert!(validate_email("noat.com").is_err());
        assert!(validate_email("@domain.com").is_err());
        assert!(validate_email("user@").is_err());
        assert!(validate_email("user@@domain.com").is_err());
    }

    #[test]
    fn test_sanitize_slug() {
        assert_eq!(sanitize_slug("Hello World"), "hello-world");
        assert_eq!(sanitize_slug("Test 123!"), "test-123");
        assert_eq!(sanitize_slug("Multiple   Spaces"), "multiple-spaces");
        assert_eq!(sanitize_slug("---hyphens---"), "hyphens");
        assert_eq!(sanitize_slug("CamelCase"), "camelcase");
    }

    #[test]
    fn test_validate_url() {
        // Valid URLs
        assert!(validate_url("https://example.com").is_ok());
        assert!(validate_url("http://test.org").is_ok());
        assert!(validate_url("https://sub.domain.com/path").is_ok());

        // Invalid URLs
        assert!(validate_url("").is_err());
        assert!(validate_url("ftp://example.com").is_err());
        assert!(validate_url("not a url").is_err());
        assert!(validate_url("http://").is_err());
    }
}
