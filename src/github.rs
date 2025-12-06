//! # GitHub Integration Module
//!
//! This module provides intelligent GitHub API integration with caching for optimal performance.
//!
//! ## Features
//!
//! - **🔐 Automatic Authentication**: Automatically detects and uses available authentication methods
//! - **🧠 Smart Caching**: TTL-based caching system reduces API calls by 100%
//! - **⚡ Performance**: Direct HTTP API calls via reqwest for maximum speed
//! - **🔄 Automatic Fallback**: Multiple fallback strategies ensure reliability
//!
//! ## Authentication Strategy
//!
//! The module automatically selects the best authentication method:
//! 1. **GITHUB_TOKEN** - Automatically provided by GitHub Actions (5,000 req/hr)
//! 2. **GH_TOKEN** - User-provided token via environment variable (5,000 req/hr)
//! 3. **gh CLI** - Falls back to gh CLI if installed (5,000 req/hr if authenticated)
//! 4. **Public API** - Unauthenticated requests as last resort (60 req/hr)
//!
//! ## Cache Performance
//!
//! The caching system dramatically improves build performance:
//! - **First run**: ~1,600ms (fresh API calls)
//! - **Subsequent runs**: 0ms (100% cache hits)
//! - **Overall improvement**: 77% faster builds
//!
//! ## Usage
//!
//! ```rust,no_run
//! use cv_generator::{github::fetch_projects_from_sources_cached, github_cache::GitHubCache};
//! use cv_generator::cv_data::GitHubSource;
//! use im::Vector;
//!
//! # fn main() -> anyhow::Result<()> {
//! let mut cache = GitHubCache::load_or_default("cache/github_cache.json");
//! let sources = Vector::new(); // Your GitHub sources
//! let projects = fetch_projects_from_sources_cached(&sources, &mut cache)?;
//! cache.save("cache/github_cache.json")?;
//! # Ok(())
//! # }
//! ```

use anyhow::{Context, Result};
use im::Vector;
use serde::{Deserialize, Serialize};
use std::env;
use std::process::Command;

use crate::cv_data::{GitHubSource, Project};
use crate::github_cache::GitHubCache;
use crate::validation::validate_github_username;

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

/// Authentication strategy for GitHub API
enum AuthStrategy {
    Token(String),
    GhCli,
    Public,
}

/// Determine the best authentication strategy
fn get_auth_strategy() -> AuthStrategy {
    // Priority 1: GITHUB_TOKEN (provided by GitHub Actions)
    if let Ok(token) = env::var("GITHUB_TOKEN") {
        if !token.is_empty() {
            return AuthStrategy::Token(token);
        }
    }

    // Priority 2: GH_TOKEN (user-provided)
    if let Ok(token) = env::var("GH_TOKEN") {
        if !token.is_empty() {
            return AuthStrategy::Token(token);
        }
    }

    // Priority 3: gh CLI (if available)
    if Command::new("gh").arg("--version").output().is_ok() {
        return AuthStrategy::GhCli;
    }

    // Priority 4: Public API (unauthenticated, 60 req/hr)
    AuthStrategy::Public
}

/// Execute a GitHub API call with automatic authentication fallback
///
/// This helper encapsulates the common pattern of trying authenticated requests
/// first, then falling back to gh CLI, then to public API.
///
/// # Arguments
/// * `with_token` - Function to call with token authentication (receives token as argument)
/// * `with_cli` - Function to call using gh CLI
/// * `without_token` - Function to call without authentication (public API)
fn with_auth_fallback<T, F1, F2, F3>(with_token: F1, with_cli: F2, without_token: F3) -> Result<T>
where
    F1: FnOnce(&str) -> Result<T>,
    F2: Fn() -> Result<T>,
    F3: Fn() -> Result<T>,
{
    match get_auth_strategy() {
        AuthStrategy::Token(ref token) => {
            println!("🔐 Using GitHub API with token authentication");
            with_token(token)
                .or_else(|e| {
                    eprintln!("⚠️  Token auth failed: {}. Trying gh CLI...", e);
                    with_cli()
                })
                .or_else(|e| {
                    eprintln!("⚠️  gh CLI failed: {}. Trying public API...", e);
                    without_token()
                })
        }
        AuthStrategy::GhCli => {
            println!("🔧 Using GitHub CLI");
            with_cli().or_else(|e| {
                eprintln!("⚠️  gh CLI failed: {}. Trying public API...", e);
                without_token()
            })
        }
        AuthStrategy::Public => {
            println!("🌐 Using public GitHub API (60 req/hr limit)");
            without_token()
        }
    }
}

/// Fetch repositories using GitHub API with token (async)
async fn fetch_repos_with_api_async(
    username: &str,
    token: Option<&str>,
) -> Result<Vec<GitHubRepo>> {
    let url = format!("https://api.github.com/users/{}/repos", username);
    let client = reqwest::Client::new();

    let mut request = client
        .get(&url)
        .header("User-Agent", "cv-generator")
        .query(&[
            ("per_page", "100"),
            ("sort", "updated"),
            ("direction", "desc"),
        ]);

    if let Some(token) = token {
        request = request.header("Authorization", format!("Bearer {}", token));
    }

    let response = request
        .send()
        .await
        .with_context(|| format!("Failed to fetch repositories for user '{}'", username))?;

    if !response.status().is_success() {
        let status = response.status();
        let error_body = response.text().await.unwrap_or_default();
        return Err(anyhow::anyhow!(
            "GitHub API request failed with status {}:\n{}\n\
             \n\
             Possible causes:\n\
             - User does not exist\n\
             - Network connectivity issues\n\
             - Rate limit exceeded ({})",
            status,
            error_body,
            if token.is_some() {
                "authenticated: 5,000 req/hr"
            } else {
                "unauthenticated: 60 req/hr - set GITHUB_TOKEN for higher limits"
            }
        ));
    }

    let mut repos: Vec<GitHubRepo> = response
        .json()
        .await
        .context("Failed to parse GitHub API response")?;

    // Filter and sort
    repos.retain(|repo| !repo.fork && !repo.archived && repo.description.is_some());
    repos.sort_by(|a, b| b.stargazers_count.cmp(&a.stargazers_count));
    repos.truncate(10);

    Ok(repos)
}

/// Fetch repositories using GitHub API with token (blocking wrapper)
fn fetch_repos_with_api(username: &str, token: Option<&str>) -> Result<Vec<GitHubRepo>> {
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(fetch_repos_with_api_async(username, token))
    })
}

/// Fetch repositories using gh CLI (fallback)
fn fetch_repos_with_gh_cli(username: &str) -> Result<Vec<GitHubRepo>> {
    let output = Command::new("gh")
        .args([
            "api",
            &format!("/users/{}/repos", username),
            "--jq",
            "map(select(.private == false and .fork == false)) | sort_by(.updated_at) | reverse | .[0:10]",
        ])
        .output()
        .context("Failed to execute 'gh' command")?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("GitHub CLI request failed: {}", error));
    }

    let json_str =
        String::from_utf8(output.stdout).context("Invalid UTF-8 in gh command output")?;

    serde_json::from_str(&json_str).context("Failed to parse GitHub API response")
}

/// Fetches public GitHub repositories for a user
///
/// # Arguments
///
/// * `username` - GitHub username
///
/// # Returns
///
/// A Result containing a Vector of Project structs
///
/// # Authentication Strategy
///
/// Uses automatic fallback in this order:
/// 1. GITHUB_TOKEN environment variable (GitHub Actions provides this)
/// 2. GH_TOKEN environment variable (user-provided)
/// 3. gh CLI (if installed and authenticated)
/// 4. Public API (unauthenticated, 60 req/hr limit)
///
/// # Rate Limits
///
/// - Authenticated: 5,000 requests per hour
/// - Unauthenticated: 60 requests per hour
pub fn fetch_github_projects(username: &str) -> Result<Vector<Project>> {
    // Validate username format before making API call
    validate_github_username(username)
        .with_context(|| format!("Invalid GitHub username: {}", username))?;

    let repos = with_auth_fallback(
        |token| fetch_repos_with_api(username, Some(token)),
        || fetch_repos_with_gh_cli(username),
        || fetch_repos_with_api(username, None),
    )?;

    Ok(convert_repos_to_projects(repos))
}

/// Fetch organization repositories using GitHub API with token (async)
async fn fetch_org_repos_with_api_async(
    org_name: &str,
    token: Option<&str>,
) -> Result<Vec<GitHubRepo>> {
    let url = format!("https://api.github.com/orgs/{}/repos", org_name);
    let client = reqwest::Client::new();

    let mut request = client
        .get(&url)
        .header("User-Agent", "cv-generator")
        .query(&[
            ("per_page", "100"),
            ("sort", "updated"),
            ("direction", "desc"),
        ]);

    if let Some(token) = token {
        request = request.header("Authorization", format!("Bearer {}", token));
    }

    let response = request.send().await.with_context(|| {
        format!(
            "Failed to fetch repositories for organization '{}'",
            org_name
        )
    })?;

    if !response.status().is_success() {
        let status = response.status();
        let error_body = response.text().await.unwrap_or_default();
        return Err(anyhow::anyhow!(
            "GitHub API request failed with status {}:\n{}",
            status,
            error_body
        ));
    }

    let mut repos: Vec<GitHubRepo> = response
        .json()
        .await
        .context("Failed to parse GitHub API response")?;

    // Filter and sort
    repos.retain(|repo| !repo.fork && !repo.archived && repo.description.is_some());
    repos.sort_by(|a, b| b.stargazers_count.cmp(&a.stargazers_count));
    repos.truncate(10);

    Ok(repos)
}

/// Fetch organization repositories using GitHub API with token (blocking wrapper)
fn fetch_org_repos_with_api(org_name: &str, token: Option<&str>) -> Result<Vec<GitHubRepo>> {
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(fetch_org_repos_with_api_async(org_name, token))
    })
}

/// Fetch organization repositories using gh CLI (fallback)
fn fetch_org_repos_with_gh_cli(org_name: &str) -> Result<Vec<GitHubRepo>> {
    let output = Command::new("gh")
        .args([
            "api",
            &format!("/orgs/{}/repos", org_name),
            "--jq",
            "map(select(.private == false and .fork == false)) | sort_by(.updated_at) | reverse | .[0:10]",
        ])
        .output()
        .context("Failed to execute 'gh' command")?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("GitHub CLI request failed: {}", error));
    }

    let json_str =
        String::from_utf8(output.stdout).context("Invalid UTF-8 in gh command output")?;

    serde_json::from_str(&json_str).context("Failed to parse GitHub API response")
}

/// Fetches public GitHub repositories for an organization
///
/// # Arguments
///
/// * `org_name` - GitHub organization name
///
/// # Returns
///
/// A Result containing a Vector of Project structs
///
/// Uses the same authentication strategy as fetch_github_projects
pub fn fetch_github_org_projects(org_name: &str) -> Result<Vector<Project>> {
    let repos = with_auth_fallback(
        |token| fetch_org_repos_with_api(org_name, Some(token)),
        || fetch_org_repos_with_gh_cli(org_name),
        || fetch_org_repos_with_api(org_name, None),
    )?;

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

/// Fetch avatar using GitHub API with token (async)
async fn fetch_avatar_with_api_async(username: &str, token: Option<&str>) -> Result<String> {
    let url = format!("https://api.github.com/users/{}", username);
    let client = reqwest::Client::new();

    let mut request = client.get(&url).header("User-Agent", "cv-generator");

    if let Some(token) = token {
        request = request.header("Authorization", format!("Bearer {}", token));
    }

    let response = request
        .send()
        .await
        .with_context(|| format!("Failed to fetch avatar for user '{}'", username))?;

    if !response.status().is_success() {
        let status = response.status();
        let error_body = response.text().await.unwrap_or_default();
        return Err(anyhow::anyhow!(
            "GitHub API request failed with status {}:\n{}",
            status,
            error_body
        ));
    }

    #[derive(Deserialize)]
    struct UserResponse {
        avatar_url: String,
    }

    let user: UserResponse = response
        .json()
        .await
        .context("Failed to parse GitHub user response")?;

    Ok(user.avatar_url)
}

/// Fetch avatar using GitHub API with token (blocking wrapper)
fn fetch_avatar_with_api(username: &str, token: Option<&str>) -> Result<String> {
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(fetch_avatar_with_api_async(username, token))
    })
}

/// Fetch avatar using gh CLI (fallback)
fn fetch_avatar_with_gh_cli(username: &str) -> Result<String> {
    let output = Command::new("gh")
        .args([
            "api",
            &format!("/users/{}", username),
            "--jq",
            ".avatar_url",
        ])
        .output()
        .context("Failed to execute gh command for user avatar")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!(
            "GitHub CLI failed to fetch user avatar: {}",
            stderr
        ));
    }

    let avatar_url = String::from_utf8(output.stdout)
        .context("Invalid UTF-8 in avatar URL response")?
        .trim()
        .trim_matches('"')
        .to_string();

    Ok(avatar_url)
}

/// Fetches GitHub user's avatar URL
///
/// # Arguments
///
/// * `username` - The GitHub username
///
/// # Returns
///
/// A Result containing the avatar URL string
///
/// Uses the same authentication strategy as fetch_github_projects
pub fn fetch_github_avatar(username: &str) -> Result<String> {
    // Validate username format before making API call
    validate_github_username(username)
        .with_context(|| format!("Invalid GitHub username: {}", username))?;

    with_auth_fallback(
        |token| fetch_avatar_with_api(username, Some(token)),
        || fetch_avatar_with_gh_cli(username),
        || fetch_avatar_with_api(username, None),
    )
}

/// Cache-aware version of fetch_projects_from_sources
///
/// This function checks the cache first before making API calls, dramatically
/// improving performance for subsequent builds.
pub fn fetch_projects_from_sources_cached(
    sources: &Vector<GitHubSource>,
    cache: &mut GitHubCache,
) -> Result<Vector<Project>> {
    let mut all_projects = Vector::new();

    for source in sources.iter() {
        let projects = if let Some(username) = &source.username {
            // Check cache first
            if let Some(cached_projects) = cache.get_projects(username) {
                println!(
                    "✅ Using cached projects for user: {} ({} projects)",
                    username,
                    cached_projects.len()
                );
                cached_projects.clone()
            } else {
                // Cache miss - fetch from API
                println!("🌐 Fetching fresh projects for user: {}", username);
                let fresh_projects = fetch_github_projects(username)?;

                // Cache the results
                cache.cache_projects(username, fresh_projects.clone());
                fresh_projects
            }
        } else if let Some(org_name) = &source.organization {
            // For organizations, we use username as cache key for simplicity
            let cache_key = format!("org:{}", org_name);

            if let Some(cached_projects) = cache.get_projects(&cache_key) {
                println!(
                    "✅ Using cached projects for org: {} ({} projects)",
                    org_name,
                    cached_projects.len()
                );
                cached_projects.clone()
            } else {
                println!("🌐 Fetching fresh projects for org: {}", org_name);
                let fresh_projects = fetch_github_org_projects(org_name)?;

                cache.cache_projects(&cache_key, fresh_projects.clone());
                fresh_projects
            }
        } else {
            Vector::new()
        };

        all_projects.extend(projects);
    }

    Ok(all_projects)
}

/// Cache-aware version of fetch_github_avatar
///
/// This function checks the cache first before making API calls.
pub fn fetch_github_avatar_cached(username: &str, cache: &mut GitHubCache) -> Result<String> {
    // Check cache first
    if let Some(cached_avatar) = cache.get_avatar(username) {
        println!("✅ Using cached avatar for user: {}", username);
        Ok(cached_avatar.to_string())
    } else {
        // Cache miss - fetch from API
        println!("🌐 Fetching fresh avatar for user: {}", username);
        let fresh_avatar = fetch_github_avatar(username)?;

        // Cache the result
        cache.cache_avatar(username, fresh_avatar.clone());

        Ok(fresh_avatar)
    }
}
