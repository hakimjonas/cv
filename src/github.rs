use anyhow::{Context, Result};
use im::Vector;
use serde::{Deserialize, Serialize};
use std::process::Command;

use crate::cv_data::{GitHubSource, Project};

/// GitHub repository information
#[derive(Debug, Deserialize, Serialize)]
struct GitHubRepo {
    name: String,
    description: Option<String>,
    html_url: String,
    topics: Option<Vector<String>>,
    language: Option<String>,
    fork: bool,
    archived: bool,
    owner: Option<GitHubOwner>,
    stargazers_count: u32,
}

/// GitHub repository owner information
#[derive(Debug, Deserialize, Serialize)]
struct GitHubOwner {
    login: String,
    avatar_url: String,
}

/// Fetches public GitHub repositories for a user
///
/// # Arguments
///
/// * `username` - GitHub username
/// * `token` - Optional GitHub API token for authentication
///
/// # Returns
///
/// A Result containing a Vector of Project structs
///
/// # Rate Limits
///
/// GitHub API has rate limits:
/// - Unauthenticated requests: 60 requests per hour
/// - Authenticated requests: 5,000 requests per hour
///
/// To avoid rate limiting, provide a GitHub API token.
pub fn fetch_github_projects(username: &str) -> Result<Vector<Project>> {
    // Use GitHub CLI to fetch repositories
    let output = Command::new("gh")
        .args([
            "api",
            &format!("/users/{}/repos", username),
            "--jq",
            "map(select(.private == false and .fork == false)) | sort_by(.updated_at) | reverse | .[0:10]",
        ])
        .output()
        .context("Failed to execute gh command. Make sure GitHub CLI is installed and authenticated.")?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!(
            "GitHub CLI request failed for user {}: {}",
            username,
            error
        ));
    }

    // Parse the JSON response
    let json_str =
        String::from_utf8(output.stdout).context("Invalid UTF-8 in gh command output")?;

    let repos: Vec<GitHubRepo> =
        serde_json::from_str(&json_str).context("Failed to parse GitHub API response")?;

    // Convert GitHub repositories to Project structs
    Ok(convert_repos_to_projects(repos))
}

/// Fetches public GitHub repositories for an organization
///
/// # Arguments
///
/// * `org_name` - GitHub organization name
/// * `token` - Optional GitHub API token for authentication
///
/// # Returns
///
/// A Result containing a Vector of Project structs
///
/// # Rate Limits
///
/// GitHub API has rate limits:
/// - Unauthenticated requests: 60 requests per hour
/// - Authenticated requests: 5,000 requests per hour
///
/// To avoid rate limiting, provide a GitHub API token.
pub fn fetch_github_org_projects(org_name: &str) -> Result<Vector<Project>> {
    // Use GitHub CLI to fetch organization repositories
    let output = Command::new("gh")
        .args([
            "api",
            &format!("/orgs/{}/repos", org_name),
            "--jq",
            "map(select(.private == false and .fork == false)) | sort_by(.updated_at) | reverse | .[0:10]",
        ])
        .output()
        .context("Failed to execute gh command. Make sure GitHub CLI is installed and authenticated.")?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!(
            "GitHub CLI request failed for organization {}: {}",
            org_name,
            error
        ));
    }

    // Parse the JSON response
    let json_str =
        String::from_utf8(output.stdout).context("Invalid UTF-8 in gh command output")?;

    let repos: Vec<GitHubRepo> =
        serde_json::from_str(&json_str).context("Failed to parse GitHub API response")?;

    // Convert GitHub repositories to Project structs
    Ok(convert_repos_to_projects(repos))
}

/// Fetches public GitHub repositories for both a user and an organization
///
/// # Arguments
///
/// * `username` - GitHub username
/// * `org_name` - GitHub organization name
/// * `token` - Optional GitHub API token for authentication
///
/// # Returns
///
/// A Result containing a Vector of Project structs
///
/// # Deprecated
///
/// This function is deprecated. Use `fetch_projects_from_sources` instead.
#[deprecated(since = "0.1.0", note = "Use fetch_projects_from_sources instead")]
#[allow(dead_code)]
pub fn fetch_all_github_projects(username: &str, org_name: &str) -> Result<Vector<Project>> {
    // Create a Vector of GitHubSource structs
    let sources = Vector::from_iter([
        GitHubSource {
            username: Some(username.to_string()),
            organization: None,
        },
        GitHubSource {
            username: None,
            organization: Some(org_name.to_string()),
        },
    ]);

    // Use the recommended function
    fetch_projects_from_sources(&sources)
}

/// Converts GitHub repositories to Project structs
///
/// # Arguments
///
/// * `repos` - Vector of GitHub repositories
///
/// # Returns
///
/// A Vector of Project structs
fn convert_repos_to_projects(repos: Vec<GitHubRepo>) -> Vector<Project> {
    repos
        .into_iter()
        .filter(|repo| !repo.fork && !repo.archived) // Filter out forks and archived repos
        .map(|repo| {
            // Create technologies vector
            let technologies = if let Some(lang) = repo.language.clone() {
                // Start with the primary language
                let base_techs = Vector::from_iter([lang]);

                // Add topics if available
                if let Some(topics) = repo.topics.clone() {
                    // Clean up topics: remove malformed tags and normalize case
                    let cleaned_topics = topics
                        .into_iter()
                        .filter(|topic| !topic.ends_with('-')) // Remove malformed tags ending with hyphen
                        .map(|topic| topic.to_lowercase()) // Normalize case
                        .collect::<Vector<_>>();

                    // Combine base_techs with cleaned_topics, avoiding duplicates
                    let mut all_techs = base_techs;
                    for topic in cleaned_topics {
                        if !all_techs.contains(&topic) && !all_techs.contains(&topic.to_lowercase())
                        {
                            all_techs.push_back(topic);
                        }
                    }
                    all_techs
                } else {
                    base_techs
                }
            } else {
                Vector::new()
            };

            // Create base name with owner information if available
            let base_name = if let Some(ref owner) = repo.owner {
                if owner.login == "fungal-lang" {
                    format!("{} (Fungal)", repo.name)
                } else {
                    repo.name.clone()
                }
            } else {
                repo.name.clone()
            };

            // Add primary language to the name if available
            let name = if let Some(lang) = &repo.language {
                format!("{base_name} - {lang}")
            } else {
                base_name
            };

            // Extract owner information if available
            let (owner_username, owner_avatar) = if let Some(owner) = &repo.owner {
                (Some(owner.login.clone()), Some(owner.avatar_url.clone()))
            } else {
                (None, None)
            };

            // Use the public HTTPS URL directly
            let repo_url = repo.html_url.clone();

            Project {
                name,
                description: repo
                    .description
                    .unwrap_or_else(|| "No description provided.".to_string()),
                url: None,
                repository: Some(repo_url),
                technologies,
                highlights: Vector::new(), // GitHub API doesn't provide highlights
                stars: Some(repo.stargazers_count),
                owner_username,
                owner_avatar,
                language: None,
                language_icon: None,
                display_name: None,
            }
        })
        .collect::<Vector<_>>()
}

/// Fetches public GitHub repositories from a list of sources
///
/// # Arguments
///
/// * `sources` - Vector of GitHubSource structs
/// * `token` - Optional GitHub API token for authentication
///
/// # Returns
///
/// A Result containing a Vector of Project structs
///
/// # Rate Limits
///
/// GitHub API has rate limits:
/// - Unauthenticated requests: 60 requests per hour
/// - Authenticated requests: 5,000 requests per hour
///
/// To avoid rate limiting, provide a GitHub API token.
pub fn fetch_projects_from_sources(sources: &Vector<GitHubSource>) -> Result<Vector<Project>> {
    // Function to merge projects
    fn merge_projects(base: &Vector<Project>, new_projects: &Vector<Project>) -> Vector<Project> {
        base.iter().chain(new_projects.iter()).cloned().collect()
    }

    // Start with an empty vector
    let mut all_projects = Vector::new();

    // Process each source
    for source in sources {
        // Process username if available
        if let Some(username) = &source.username {
            // Fetch user repositories
            match fetch_github_projects(username) {
                Ok(projects) => {
                    // Merge the new projects with the existing ones
                    all_projects = merge_projects(&all_projects, &projects);
                }
                Err(e) => {
                    println!("Warning: Failed to fetch GitHub projects for user {username}: {e}");
                }
            }
        }

        // Process organization if available
        if let Some(org_name) = &source.organization {
            // Fetch organization repositories
            match fetch_github_org_projects(org_name) {
                Ok(projects) => {
                    // Merge the new projects with the existing ones
                    all_projects = merge_projects(&all_projects, &projects);
                }
                Err(e) => {
                    println!(
                        "Warning: Failed to fetch GitHub projects for organization {org_name}: {e}"
                    );
                }
            }
        }
    }

    Ok(all_projects)
}
