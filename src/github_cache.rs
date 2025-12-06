//! GitHub API response caching system
//!
//! This module provides intelligent caching for GitHub API responses to dramatically
//! improve build performance by avoiding redundant API calls.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use im::{HashMap, Vector};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::cv_data::Project;

/// Cache entry for GitHub API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubCacheEntry<T> {
    /// Cached data
    pub data: T,
    /// When the data was cached
    pub cached_at: DateTime<Utc>,
    /// Cache expiration time (in minutes)
    pub ttl_minutes: u32,
}

impl<T> GitHubCacheEntry<T> {
    /// Create a new cache entry with default TTL of 60 minutes
    pub fn new(data: T) -> Self {
        Self {
            data,
            cached_at: Utc::now(),
            ttl_minutes: 60,
        }
    }

    /// Create a new cache entry with custom TTL
    pub fn with_ttl(data: T, ttl_minutes: u32) -> Self {
        Self {
            data,
            cached_at: Utc::now(),
            ttl_minutes,
        }
    }

    /// Check if the cache entry is still valid
    pub fn is_valid(&self) -> bool {
        let expiry = self.cached_at + chrono::Duration::minutes(self.ttl_minutes as i64);
        Utc::now() < expiry
    }

    /// Get age of the cache entry in minutes
    #[allow(dead_code)]
    pub fn age_minutes(&self) -> i64 {
        (Utc::now() - self.cached_at).num_minutes()
    }
}

/// GitHub API cache for projects and avatar data
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GitHubCache {
    /// Cached GitHub projects by username
    pub projects: HashMap<String, GitHubCacheEntry<Vector<Project>>>,
    /// Cached avatar URLs by username
    pub avatars: HashMap<String, GitHubCacheEntry<String>>,
    /// Cache metadata
    pub metadata: CacheMetadata,
}

/// Metadata about the cache file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMetadata {
    /// Cache format version
    pub version: String,
    /// When the cache was created
    pub created_at: DateTime<Utc>,
    /// Last time cache was updated
    pub updated_at: DateTime<Utc>,
}

impl Default for CacheMetadata {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            version: "1.0.0".to_string(),
            created_at: now,
            updated_at: now,
        }
    }
}

impl GitHubCache {
    /// Load cache from file, or create new cache if file doesn't exist
    pub fn load_or_default<P: AsRef<Path>>(cache_path: P) -> Self {
        match Self::load(&cache_path) {
            Ok(cache) => {
                println!(
                    "🔄 Loaded GitHub cache from {}",
                    cache_path.as_ref().display()
                );
                cache
            }
            Err(_) => {
                println!("📄 Creating new GitHub cache");
                Self::default()
            }
        }
    }

    /// Load cache from file
    pub fn load<P: AsRef<Path>>(cache_path: P) -> Result<Self> {
        let content = fs::read_to_string(&cache_path).with_context(|| {
            format!(
                "Failed to read cache file: {}",
                cache_path.as_ref().display()
            )
        })?;

        let cache: Self = serde_json::from_str(&content).with_context(|| {
            format!(
                "Failed to parse cache file: {}",
                cache_path.as_ref().display()
            )
        })?;

        Ok(cache)
    }

    /// Save cache to file
    pub fn save<P: AsRef<Path>>(&mut self, cache_path: P) -> Result<()> {
        // Update metadata
        self.metadata.updated_at = Utc::now();

        // Ensure parent directory exists
        if let Some(parent) = cache_path.as_ref().parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create cache directory: {}", parent.display())
            })?;
        }

        let content =
            serde_json::to_string_pretty(self).context("Failed to serialize cache data")?;

        fs::write(&cache_path, content).with_context(|| {
            format!(
                "Failed to write cache file: {}",
                cache_path.as_ref().display()
            )
        })?;

        println!("💾 Saved GitHub cache to {}", cache_path.as_ref().display());
        Ok(())
    }

    /// Get cached projects for a username, if valid
    pub fn get_projects(&self, username: &str) -> Option<&Vector<Project>> {
        self.projects
            .get(username)
            .filter(|entry| entry.is_valid())
            .map(|entry| &entry.data)
    }

    /// Cache projects for a username
    pub fn cache_projects(&mut self, username: &str, projects: Vector<Project>) {
        println!(
            "📦 Caching {} projects for user: {}",
            projects.len(),
            username
        );
        self.projects
            .insert(username.to_string(), GitHubCacheEntry::new(projects));
    }

    /// Get cached avatar URL for a username, if valid
    pub fn get_avatar(&self, username: &str) -> Option<&str> {
        self.avatars
            .get(username)
            .filter(|entry| entry.is_valid())
            .map(|entry| entry.data.as_str())
    }

    /// Cache avatar URL for a username
    pub fn cache_avatar(&mut self, username: &str, avatar_url: String) {
        println!("🖼️  Caching avatar for user: {}", username);
        self.avatars.insert(
            username.to_string(),
            GitHubCacheEntry::with_ttl(avatar_url, 240), // 4 hours TTL for avatars
        );
    }

    /// Clean up expired entries from cache (returns new cache with expired entries removed)
    pub fn cleanup_expired(&mut self) -> usize {
        let initial_projects = self.projects.len();
        let initial_avatars = self.avatars.len();

        // Filter to keep only valid entries (functional approach with im::HashMap)
        self.projects = self
            .projects
            .iter()
            .filter(|(_, entry)| entry.is_valid())
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        self.avatars = self
            .avatars
            .iter()
            .filter(|(_, entry)| entry.is_valid())
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        let cleaned_projects = initial_projects - self.projects.len();
        let cleaned_avatars = initial_avatars - self.avatars.len();
        let total_cleaned = cleaned_projects + cleaned_avatars;

        if total_cleaned > 0 {
            println!("🧹 Cleaned {} expired cache entries", total_cleaned);
        }

        total_cleaned
    }

    /// Print cache statistics
    #[allow(dead_code)]
    pub fn print_stats(&self) {
        let valid_projects: usize = self
            .projects
            .values()
            .map(|entry| if entry.is_valid() { 1 } else { 0 })
            .sum();

        let valid_avatars: usize = self
            .avatars
            .values()
            .map(|entry| if entry.is_valid() { 1 } else { 0 })
            .sum();

        println!("📊 GitHub Cache Stats:");
        println!(
            "  Projects: {} valid, {} total",
            valid_projects,
            self.projects.len()
        );
        println!(
            "  Avatars:  {} valid, {} total",
            valid_avatars,
            self.avatars.len()
        );

        if !self.projects.is_empty() {
            let oldest_project = self
                .projects
                .values()
                .map(|entry| entry.age_minutes())
                .max()
                .unwrap_or(0);
            println!("  Oldest entry: {} minutes ago", oldest_project);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_cache_entry_validity() {
        let entry = GitHubCacheEntry::with_ttl("test data".to_string(), 60);
        assert!(entry.is_valid());
        assert_eq!(entry.age_minutes(), 0);
    }

    #[test]
    fn test_cache_save_load() {
        let dir = tempdir().unwrap();
        let cache_path = dir.path().join("test_cache.json");

        let mut cache = GitHubCache::default();
        cache.cache_avatar("testuser", "http://example.com/avatar.png".to_string());

        cache.save(&cache_path).unwrap();

        let loaded_cache = GitHubCache::load(&cache_path).unwrap();
        assert!(loaded_cache.get_avatar("testuser").is_some());
        assert_eq!(
            loaded_cache.get_avatar("testuser").unwrap(),
            "http://example.com/avatar.png"
        );
    }

    #[test]
    fn test_cache_cleanup() {
        let mut cache = GitHubCache::default();

        // Add an expired entry (TTL = 0)
        cache.avatars.insert(
            "expired_user".to_string(),
            GitHubCacheEntry::with_ttl("old_avatar".to_string(), 0),
        );

        // Add a valid entry
        cache.cache_avatar("valid_user", "new_avatar".to_string());

        let cleaned = cache.cleanup_expired();
        assert!(cleaned > 0);
        assert!(cache.get_avatar("expired_user").is_none());
        assert!(cache.get_avatar("valid_user").is_some());
    }
}
