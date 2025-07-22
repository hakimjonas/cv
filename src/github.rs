use anyhow::{Context, Result};
use im::Vector;
use once_cell::sync::Lazy;
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue, USER_AGENT};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::runtime::Runtime;

use crate::config;
use crate::cv_data::{GitHubSource, Project};

// Shared Tokio runtime for all synchronous API calls
static RUNTIME: Lazy<Arc<Runtime>> =
    Lazy::new(|| Arc::new(Runtime::new().expect("Failed to create Tokio runtime")));

/// Creates headers for GitHub API requests
///
/// # Arguments
///
/// * `token` - Optional GitHub API token for authentication
///
/// # Returns
///
/// A HeaderMap with appropriate headers for GitHub API requests
fn create_github_headers(token: Option<&str>) -> Result<HeaderMap> {
    // Create a base header map with the User-Agent
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("CV-Generator/1.0"));

    // Add authorization header if token is provided
    if let Some(token_str) = token {
        let auth_value = format!("token {token_str}");
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&auth_value).context("Invalid token format")?,
        );
    }

    Ok(headers)
}

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
pub async fn fetch_github_projects(username: &str, token: Option<&str>) -> Result<Vector<Project>> {
    // Create a client with appropriate headers
    let client = reqwest::Client::new();
    let headers = create_github_headers(token)?;

    // Fetch repositories from GitHub API
    let url = format!("https://api.github.com/users/{username}/repos?sort=updated&per_page=10");

    // Make the request
    let response = client
        .get(&url)
        .headers(headers)
        .send()
        .await
        .context("Failed to fetch GitHub repositories")?;

    // Check if the request was successful
    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "GitHub API request failed with status: {}",
            response.status()
        ));
    }

    // Parse the response
    let repos = response
        .json::<Vec<GitHubRepo>>()
        .await
        .context("Failed to parse GitHub API response")?;

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
pub async fn fetch_github_org_projects(
    org_name: &str,
    token: Option<&str>,
) -> Result<Vector<Project>> {
    // Create a client with appropriate headers
    let client = reqwest::Client::new();
    let headers = create_github_headers(token)?;

    // Fetch repositories from GitHub API
    let url = format!("https://api.github.com/orgs/{org_name}/repos?sort=updated&per_page=10");

    // Make the request
    let response = client
        .get(&url)
        .headers(headers)
        .send()
        .await
        .context("Failed to fetch GitHub organization repositories")?;

    // Check if the request was successful
    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "GitHub API request failed with status: {}",
            response.status()
        ));
    }

    // Parse the response
    let repos = response
        .json::<Vec<GitHubRepo>>()
        .await
        .context("Failed to parse GitHub API response")?;

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
pub async fn fetch_all_github_projects(
    username: &str,
    org_name: &str,
    token: Option<&str>,
) -> Result<Vector<Project>> {
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
    fetch_projects_from_sources(&sources, token).await
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

/// Synchronous version of fetch_github_projects for use in non-async contexts
///
/// # Arguments
///
/// * `username` - GitHub username
/// * `token` - Optional GitHub API token for authentication
///
/// # Returns
///
/// A Result containing a Vector of Project structs
#[allow(dead_code)]
pub fn fetch_github_projects_sync(username: &str, token: Option<&str>) -> Result<Vector<Project>> {
    // Use the shared runtime to run the async function
    RUNTIME.block_on(fetch_github_projects(username, token))
}

/// Synchronous version of fetch_github_org_projects for use in non-async contexts
///
/// # Arguments
///
/// * `org_name` - GitHub organization name
/// * `token` - Optional GitHub API token for authentication
///
/// # Returns
///
/// A Result containing a Vector of Project structs
#[allow(dead_code)]
pub fn fetch_github_org_projects_sync(
    org_name: &str,
    token: Option<&str>,
) -> Result<Vector<Project>> {
    // Use the shared runtime to run the async function
    RUNTIME.block_on(fetch_github_org_projects(org_name, token))
}

/// Synchronous version of fetch_all_github_projects for use in non-async contexts
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
/// This function is deprecated. Use `fetch_projects_from_sources_sync` instead.
#[deprecated(since = "0.1.0", note = "Use fetch_projects_from_sources_sync instead")]
#[allow(dead_code)]
pub fn fetch_all_github_projects_sync(
    username: &str,
    org_name: &str,
    token: Option<&str>,
) -> Result<Vector<Project>> {
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
    fetch_projects_from_sources_sync(&sources, token)
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
pub async fn fetch_projects_from_sources(
    sources: &Vector<GitHubSource>,
    token: Option<&str>,
) -> Result<Vector<Project>> {
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
            match fetch_github_projects(username, token).await {
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
            match fetch_github_org_projects(org_name, token).await {
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

/// Reads projects from the GitHub cache file
///
/// # Arguments
///
/// * `cache_path` - Path to the cache file
///
/// # Returns
///
/// A Result containing a Vector of Project structs if the cache exists and is valid,
/// or an error if the cache doesn't exist or is invalid
fn read_github_cache(cache_path: &Path) -> Result<Vector<Project>> {
    // Check if the cache file exists
    if !cache_path.exists() {
        return Err(anyhow::anyhow!("GitHub cache file does not exist"));
    }

    // Check if the cache file is recent (less than 1 hour old)
    let metadata = fs::metadata(cache_path).context("Failed to read cache file metadata")?;
    let modified = metadata
        .modified()
        .context("Failed to get cache file modification time")?;
    let now = SystemTime::now();
    let age = now
        .duration_since(modified)
        .unwrap_or(Duration::from_secs(0));

    // If the cache is older than 1 hour, consider it invalid
    if age > Duration::from_secs(3600) {
        return Err(anyhow::anyhow!("GitHub cache is too old"));
    }

    // Read the cache file
    let cache_data = fs::read_to_string(cache_path).context("Failed to read GitHub cache file")?;

    // Parse the cache data
    let projects: Vector<Project> =
        serde_json::from_str(&cache_data).context("Failed to parse GitHub cache data")?;

    Ok(projects)
}

/// Writes projects to the GitHub cache file
///
/// # Arguments
///
/// * `cache_path` - Path to the cache file
/// * `projects` - Vector of Project structs to cache
///
/// # Returns
///
/// A Result indicating success or failure
fn write_github_cache(cache_path: &Path, projects: &Vector<Project>) -> Result<()> {
    // Create the parent directory if it doesn't exist
    if let Some(parent) = cache_path.parent() {
        fs::create_dir_all(parent).context("Failed to create cache directory")?;
    }

    // Serialize the projects to JSON
    let cache_data = serde_json::to_string_pretty(projects)
        .context("Failed to serialize GitHub projects for cache")?;

    // Write the cache file
    fs::write(cache_path, cache_data).context("Failed to write GitHub cache file")?;

    Ok(())
}

/// Synchronous version of fetch_projects_from_sources for use in non-async contexts
///
/// # Arguments
///
/// * `sources` - Vector of GitHubSource structs
/// * `token` - Optional GitHub API token for authentication
///
/// # Returns
///
/// A Result containing a Vector of Project structs
pub fn fetch_projects_from_sources_sync(
    sources: &Vector<GitHubSource>,
    token: Option<&str>,
) -> Result<Vector<Project>> {
    // Get the cache path from the config
    let config = config::Config::default();
    let cache_path_str = config
        .github_cache_path_str()
        .context("Failed to get GitHub cache path")?;
    let cache_path = Path::new(&cache_path_str);

    // Try to read from the cache first
    match read_github_cache(cache_path) {
        Ok(projects) => {
            println!(
                "Using cached GitHub projects (from {})",
                cache_path.display()
            );
            return Ok(projects);
        }
        Err(e) => {
            println!("GitHub cache not available: {e}");
            println!("Fetching projects from GitHub API...");
        }
    }

    // Use the shared runtime to run the async function
    let projects = RUNTIME.block_on(fetch_projects_from_sources(sources, token))?;

    // Write the results to the cache
    if let Err(e) = write_github_cache(cache_path, &projects) {
        println!("Warning: Failed to write GitHub cache: {e}");
    } else {
        println!("GitHub projects cached to {}", cache_path.display());
    }

    Ok(projects)
}
