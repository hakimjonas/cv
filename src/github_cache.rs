/// GitHub cache module for the CV generator
///
/// This module provides an enhanced caching mechanism for GitHub API data,
/// with support for per-item TTL, background refresh, and rate limit handling.
use anyhow::{Context, Result};
use im::Vector;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::time::{Duration, SystemTime};
use tracing::{debug, info, warn};

use crate::cv_data::Project;
use crate::unified_config::AppConfig;

/// Represents the cache metadata for the entire cache
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMetadata {
    /// Version of the cache format
    pub version: u32,
    /// When the cache was last refreshed
    pub last_refresh: u64,
    /// When the cache will expire (global TTL)
    pub expires_at: u64,
    /// Number of items in the cache
    pub item_count: usize,
    /// GitHub API rate limit information
    pub rate_limit: Option<RateLimitInfo>,
}

/// Represents rate limit information from the GitHub API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitInfo {
    /// Number of requests remaining in the current rate limit window
    pub remaining: u32,
    /// Total number of requests allowed in the rate limit window
    pub limit: u32,
    /// When the rate limit will reset (Unix timestamp)
    pub reset_at: u64,
}

/// Represents a cached project with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedProject {
    /// The project data
    pub project: Project,
    /// When the project was fetched
    pub fetched_at: u64,
    /// When the project will expire
    pub expires_at: u64,
    /// ETag for conditional requests
    pub etag: Option<String>,
}

/// Represents the entire cache
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubCache {
    /// Cache metadata
    pub metadata: CacheMetadata,
    /// Cached projects
    pub projects: Vector<CachedProject>,
}

impl GitHubCache {
    /// Creates a new empty cache
    pub fn new() -> Self {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            metadata: CacheMetadata {
                version: 1,
                last_refresh: now,
                expires_at: now + 3600, // Default 1 hour TTL
                item_count: 0,
                rate_limit: None,
            },
            projects: Vector::new(),
        }
    }

    /// Creates a new cache with the given projects
    pub fn with_projects(projects: &Vector<Project>, config: &AppConfig) -> Self {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let ttl = config.github_cache_ttl;
        let expires_at = now + ttl;

        let cached_projects = projects
            .iter()
            .map(|project| CachedProject {
                project: project.clone(),
                fetched_at: now,
                expires_at,
                etag: None,
            })
            .collect();

        Self {
            metadata: CacheMetadata {
                version: 1,
                last_refresh: now,
                expires_at,
                item_count: projects.len(),
                rate_limit: None,
            },
            projects: cached_projects,
        }
    }

    /// Updates the cache with new projects
    pub fn update_with_projects(&mut self, projects: &Vector<Project>, config: &AppConfig) {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let ttl = config.github_cache_ttl;
        let expires_at = now + ttl;

        // Create a map of existing projects by repository URL for quick lookup
        let mut existing_projects = std::collections::HashMap::new();
        for (i, cached_project) in self.projects.iter().enumerate() {
            if let Some(repo_url) = &cached_project.project.repository {
                existing_projects.insert(repo_url.clone(), i);
            }
        }

        // Update existing projects and add new ones
        let mut updated_projects = Vector::new();
        for project in projects {
            if let Some(repo_url) = &project.repository {
                if let Some(index) = existing_projects.get(repo_url) {
                    // Update existing project
                    let mut cached_project = self.projects[*index].clone();
                    cached_project.project = project.clone();
                    cached_project.fetched_at = now;
                    cached_project.expires_at = expires_at;
                    updated_projects.push_back(cached_project);
                } else {
                    // Add new project
                    updated_projects.push_back(CachedProject {
                        project: project.clone(),
                        fetched_at: now,
                        expires_at,
                        etag: None,
                    });
                }
            } else {
                // Project without repository URL, just add it
                updated_projects.push_back(CachedProject {
                    project: project.clone(),
                    fetched_at: now,
                    expires_at,
                    etag: None,
                });
            }
        }

        // Update cache metadata
        self.metadata.last_refresh = now;
        self.metadata.expires_at = expires_at;
        self.metadata.item_count = updated_projects.len();
        self.projects = updated_projects;
    }

    /// Updates the rate limit information
    pub fn update_rate_limit(&mut self, remaining: u32, limit: u32, reset_at: u64) {
        self.metadata.rate_limit = Some(RateLimitInfo {
            remaining,
            limit,
            reset_at,
        });
    }

    /// Checks if the cache is expired
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        self.metadata.expires_at < now
    }

    /// Gets the projects from the cache
    pub fn get_projects(&self) -> Vector<Project> {
        self.projects.iter().map(|cp| cp.project.clone()).collect()
    }

    /// Reads the cache from a file
    pub fn from_file(path: &Path, config: &AppConfig) -> Result<Self> {
        // Check if the cache file exists
        if !path.exists() {
            debug!("GitHub cache file does not exist: {}", path.display());
            return Err(anyhow::anyhow!("GitHub cache file does not exist"));
        }

        // Read the cache file
        let cache_data = fs::read_to_string(path).context("Failed to read GitHub cache file")?;

        // Try to parse as the new format first
        match serde_json::from_str::<GitHubCache>(&cache_data) {
            Ok(cache) => {
                // Check if the cache is expired based on the configuration
                if cache.is_expired() {
                    debug!("GitHub cache is expired");
                    return Err(anyhow::anyhow!("GitHub cache is expired"));
                }

                debug!(
                    "Successfully loaded GitHub cache with {} projects",
                    cache.projects.len()
                );
                Ok(cache)
            }
            Err(_) => {
                // Try to parse as the old format (just a Vector<Project>)
                debug!("Trying to parse GitHub cache as old format");
                let projects: Vector<Project> = serde_json::from_str(&cache_data)
                    .context("Failed to parse GitHub cache data")?;

                // Convert to the new format
                debug!("Converting old cache format to new format");
                Ok(Self::with_projects(&projects, config))
            }
        }
    }

    /// Writes the cache to a file
    pub fn to_file(&self, path: &Path) -> Result<()> {
        // Create the parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).context("Failed to create cache directory")?;
        }

        // Serialize the cache to JSON
        let cache_data =
            serde_json::to_string_pretty(self).context("Failed to serialize GitHub cache")?;

        // Write the cache file
        fs::write(path, cache_data).context("Failed to write GitHub cache file")?;

        debug!("Successfully wrote GitHub cache to {}", path.display());
        Ok(())
    }
}

/// Default implementation for GitHubCache
impl Default for GitHubCache {
    fn default() -> Self {
        Self::new()
    }
}
