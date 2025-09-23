use anyhow::{Context, Result, anyhow};
use regex::Regex;
use std::process::Command;

/// Represents a Git user identity
#[derive(Debug, Clone)]
pub struct GitIdentity {
    /// The user's name from Git config
    pub name: String,
    /// The user's email from Git config
    pub email: String,
    /// The user's GitHub username (if available)
    pub github_username: Option<String>,
}

/// Service for extracting Git identity information
pub struct GitIdentityService;

impl Default for GitIdentityService {
    fn default() -> Self {
        Self::new()
    }
}

impl GitIdentityService {
    /// Creates a new GitIdentityService
    pub fn new() -> Self {
        Self
    }

    /// Extracts Git identity from local Git configuration
    pub fn get_identity(&self) -> Result<GitIdentity> {
        let name = self.get_git_config_value("user.name")?;
        let email = self.get_git_config_value("user.email")?;
        let github_username = self.extract_github_username().ok();

        Ok(GitIdentity {
            name,
            email,
            github_username,
        })
    }

    /// Verifies that Git is properly configured with user name and email
    pub fn verify_git_setup(&self) -> Result<()> {
        let name = self.get_git_config_value("user.name");
        let email = self.get_git_config_value("user.email");

        match (name, email) {
            (Ok(_), Ok(_)) => Ok(()),
            (Err(_), Ok(_)) => Err(anyhow!(
                "Git user.name is not configured. Please run: git config --global user.name \"Your Name\""
            )),
            (Ok(_), Err(_)) => Err(anyhow!(
                "Git user.email is not configured. Please run: git config --global user.email \"your.email@example.com\""
            )),
            (Err(_), Err(_)) => Err(anyhow!(
                "Git user.name and user.email are not configured. Please run:\ngit config --global user.name \"Your Name\"\ngit config --global user.email \"your.email@example.com\""
            )),
        }
    }

    /// Gets a value from Git config
    fn get_git_config_value(&self, key: &str) -> Result<String> {
        let output = Command::new("git")
            .args(["config", "--global", key])
            .output()
            .context(format!("Failed to execute git config --global {key}"))?;

        if output.status.success() {
            let value = String::from_utf8(output.stdout)
                .context("Git config output is not valid UTF-8")?
                .trim()
                .to_string();

            if value.is_empty() {
                return Err(anyhow!("Git config {} is empty", key));
            }

            Ok(value)
        } else {
            Err(anyhow!("Git config {} is not set", key))
        }
    }

    /// Extracts GitHub username from remote URL
    fn extract_github_username(&self) -> Result<String> {
        // Try to get the GitHub username from the remote URL
        let remote_url = self.get_git_remote_url()?;

        // Parse different GitHub remote URL formats
        let username = if let Some(username) = self.parse_github_url(&remote_url) {
            username
        } else {
            // Fallback to GitHub CLI if available
            self.get_github_username_from_cli()?
        };

        Ok(username)
    }

    /// Gets the Git remote URL
    fn get_git_remote_url(&self) -> Result<String> {
        let output = Command::new("git")
            .args(["remote", "get-url", "origin"])
            .output()
            .context("Failed to execute git remote get-url origin")?;

        if output.status.success() {
            let url = String::from_utf8(output.stdout)
                .context("Git remote URL is not valid UTF-8")?
                .trim()
                .to_string();

            if url.is_empty() {
                return Err(anyhow!("Git remote URL is empty"));
            }

            Ok(url)
        } else {
            Err(anyhow!("Failed to get Git remote URL"))
        }
    }

    /// Parses GitHub username from various URL formats
    fn parse_github_url(&self, url: &str) -> Option<String> {
        // SSH format: git@github.com:username/repo.git
        let ssh_regex = Regex::new(r"git@github\.com:([^/]+)/").ok()?;
        if let Some(captures) = ssh_regex.captures(url) {
            return captures.get(1).map(|m| m.as_str().to_string());
        }

        // HTTPS format: https://github.com/username/repo.git
        let https_regex = Regex::new(r"https://github\.com/([^/]+)/").ok()?;
        if let Some(captures) = https_regex.captures(url) {
            return captures.get(1).map(|m| m.as_str().to_string());
        }

        // HTTPS with auth: https://username@github.com/username/repo.git
        let https_auth_regex = Regex::new(r"https://[^@]+@github\.com/([^/]+)/").ok()?;
        if let Some(captures) = https_auth_regex.captures(url) {
            return captures.get(1).map(|m| m.as_str().to_string());
        }

        None
    }

    /// Gets GitHub username from GitHub CLI if available
    fn get_github_username_from_cli(&self) -> Result<String> {
        let output = Command::new("gh")
            .args(["auth", "status", "--show-token"])
            .output()
            .context("Failed to execute gh auth status")?;

        if output.status.success() {
            let output_str =
                String::from_utf8(output.stdout).context("GitHub CLI output is not valid UTF-8")?;

            // Extract username from output like "Logged in to github.com as username"
            let regex = Regex::new(r"Logged in to github\.com as ([^\s]+)")
                .ok()
                .context("Failed to create regex for GitHub CLI output")?;

            if let Some(captures) = regex.captures(&output_str)
                && let Some(username) = captures.get(1)
            {
                return Ok(username.as_str().to_string());
            }
        }

        Err(anyhow!("Could not determine GitHub username"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_github_url_ssh() {
        let service = GitIdentityService::new();
        let url = "git@github.com:hakimjonas/cv.git";
        assert_eq!(
            service.parse_github_url(url),
            Some("hakimjonas".to_string())
        );
    }

    #[test]
    fn test_parse_github_url_https() {
        let service = GitIdentityService::new();
        let url = "https://github.com/hakimjonas/cv.git";
        assert_eq!(
            service.parse_github_url(url),
            Some("hakimjonas".to_string())
        );
    }

    #[test]
    fn test_parse_github_url_https_with_auth() {
        let service = GitIdentityService::new();
        let url = "https://hakimjonas@github.com/hakimjonas/cv.git";
        assert_eq!(
            service.parse_github_url(url),
            Some("hakimjonas".to_string())
        );
    }
}
