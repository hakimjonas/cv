use anyhow::{Context, Result};
use im::Vector;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use serde::{Deserialize, Serialize};

use crate::cv_data::Project;

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
///
/// # Returns
///
/// A Result containing a Vector of Project structs
pub async fn fetch_github_projects(username: &str) -> Result<Vector<Project>> {
    // Create a client with appropriate headers
    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("CV-Generator/1.0"));

    // Fetch repositories from GitHub API
    let url = format!(
        "https://api.github.com/users/{}/repos?sort=updated&per_page=10",
        username
    );
    let response = client
        .get(&url)
        .headers(headers.clone())
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
    let repos: Vec<GitHubRepo> = response
        .json()
        .await
        .context("Failed to parse GitHub API response")?;

    // Convert GitHub repositories to Project structs
    let projects = convert_repos_to_projects(repos);

    Ok(projects)
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
pub async fn fetch_github_org_projects(org_name: &str) -> Result<Vector<Project>> {
    // Create a client with appropriate headers
    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("CV-Generator/1.0"));

    // Fetch repositories from GitHub API
    let url = format!(
        "https://api.github.com/orgs/{}/repos?sort=updated&per_page=10",
        org_name
    );
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
    let repos: Vec<GitHubRepo> = response
        .json()
        .await
        .context("Failed to parse GitHub API response")?;

    // Convert GitHub repositories to Project structs
    let projects = convert_repos_to_projects(repos);

    Ok(projects)
}

/// Fetches public GitHub repositories for both a user and an organization
///
/// # Arguments
///
/// * `username` - GitHub username
/// * `org_name` - GitHub organization name
///
/// # Returns
///
/// A Result containing a Vector of Project structs
pub async fn fetch_all_github_projects(username: &str, org_name: &str) -> Result<Vector<Project>> {
    // Fetch user repositories
    let user_projects = fetch_github_projects(username).await.unwrap_or_else(|e| {
        println!("Warning: Failed to fetch user GitHub projects: {}", e);
        Vector::new()
    });

    // Fetch organization repositories
    let org_projects = fetch_github_org_projects(org_name)
        .await
        .unwrap_or_else(|e| {
            println!(
                "Warning: Failed to fetch organization GitHub projects: {}",
                e
            );
            Vector::new()
        });

    // Combine user and organization projects
    let all_projects = user_projects
        .iter()
        .chain(org_projects.iter())
        .cloned()
        .collect();

    Ok(all_projects)
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
            let technologies = if let Some(lang) = repo.language.clone() {
                let mut techs = Vector::new();
                techs.push_back(lang);
                if let Some(topics) = repo.topics.clone() {
                    for topic in topics {
                        techs.push_back(topic);
                    }
                }
                techs
            } else {
                Vector::new()
            };

            // Add owner information and language to the name if available
            let mut name = if let Some(ref owner) = repo.owner {
                if owner.login == "fungal-lang" {
                    format!("{} (Fungal)", repo.name)
                } else {
                    repo.name.clone()
                }
            } else {
                repo.name.clone()
            };

            // Add primary language to the name if available
            if let Some(lang) = &repo.language {
                name = format!("{} - {}", name, lang);
            };

            // Extract owner information if available
            let (owner_username, owner_avatar) = if let Some(owner) = &repo.owner {
                (Some(owner.login.clone()), Some(owner.avatar_url.clone()))
            } else {
                (None, None)
            };

            Project {
                name,
                description: repo
                    .description
                    .unwrap_or_else(|| "No description provided.".to_string()),
                url: None,
                repository: Some(repo.html_url),
                technologies,
                highlights: Vector::new(), // GitHub API doesn't provide highlights
                stars: Some(repo.stargazers_count),
                owner_username,
                owner_avatar,
            }
        })
        .collect::<Vector<_>>()
}

/// Synchronous version of fetch_github_projects for use in non-async contexts
///
/// # Arguments
///
/// * `username` - GitHub username
///
/// # Returns
///
/// A Result containing a Vector of Project structs
#[allow(dead_code)]
pub fn fetch_github_projects_sync(username: &str) -> Result<Vector<Project>> {
    // Create a runtime for the async function
    let rt = tokio::runtime::Runtime::new().context("Failed to create Tokio runtime")?;

    // Run the async function in the runtime
    rt.block_on(fetch_github_projects(username))
}

/// Synchronous version of fetch_github_org_projects for use in non-async contexts
///
/// # Arguments
///
/// * `org_name` - GitHub organization name
///
/// # Returns
///
/// A Result containing a Vector of Project structs
#[allow(dead_code)]
pub fn fetch_github_org_projects_sync(org_name: &str) -> Result<Vector<Project>> {
    // Create a runtime for the async function
    let rt = tokio::runtime::Runtime::new().context("Failed to create Tokio runtime")?;

    // Run the async function in the runtime
    rt.block_on(fetch_github_org_projects(org_name))
}

/// Synchronous version of fetch_all_github_projects for use in non-async contexts
///
/// # Arguments
///
/// * `username` - GitHub username
/// * `org_name` - GitHub organization name
///
/// # Returns
///
/// A Result containing a Vector of Project structs
pub fn fetch_all_github_projects_sync(username: &str, org_name: &str) -> Result<Vector<Project>> {
    // Create a runtime for the async function
    let rt = tokio::runtime::Runtime::new().context("Failed to create Tokio runtime")?;

    // Run the async function in the runtime
    rt.block_on(fetch_all_github_projects(username, org_name))
}
