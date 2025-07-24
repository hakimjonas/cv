/*!
 * Image storage module
 * This module provides functionality for storing and retrieving images
 */

use anyhow::{Result, anyhow};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use tokio::task;
use tracing::{info, instrument};
use uuid::Uuid;

/// Configuration for image storage
#[derive(Debug, Clone)]
pub struct ImageStorageConfig {
    /// Base directory for storing images
    pub base_dir: PathBuf,
    /// Maximum file size in bytes
    pub max_file_size: usize,
    /// Allowed file extensions
    pub allowed_extensions: Vec<String>,
}

impl Default for ImageStorageConfig {
    fn default() -> Self {
        Self {
            base_dir: PathBuf::from("uploads/images"),
            max_file_size: 5 * 1024 * 1024, // 5MB
            allowed_extensions: vec![
                "jpg".to_string(),
                "jpeg".to_string(),
                "png".to_string(),
                "gif".to_string(),
                "webp".to_string(),
            ],
        }
    }
}

/// Image storage service
#[derive(Debug, Clone)]
pub struct ImageStorage {
    config: ImageStorageConfig,
}

impl ImageStorage {
    /// Create a new image storage service with the given configuration
    pub fn new(config: ImageStorageConfig) -> Result<Self> {
        // Ensure the base directory exists
        fs::create_dir_all(&config.base_dir)?;

        Ok(Self { config })
    }

    /// Create a new image storage service with default configuration
    ///
    /// This method returns a Result since it may fail when creating directories.
    pub fn default_with_result() -> Result<Self> {
        Self::new(ImageStorageConfig::default())
    }

    /// Store an image with the given filename and data
    #[instrument(skip(self, data), err)]
    pub async fn store_image(&self, filename: &str, data: &[u8]) -> Result<String> {
        // Validate file size
        if data.len() > self.config.max_file_size {
            return Err(anyhow!("File size exceeds maximum allowed size"));
        }

        // Validate file extension
        let extension = Path::new(filename)
            .extension()
            .and_then(|ext| ext.to_str())
            .ok_or_else(|| anyhow!("Invalid file extension"))?
            .to_lowercase();

        if !self.config.allowed_extensions.contains(&extension) {
            return Err(anyhow!("File extension not allowed"));
        }

        // Generate a unique filename
        let unique_filename = format!("{}.{}", Uuid::new_v4(), extension);
        let file_path = self.config.base_dir.join(&unique_filename);

        // Store the file
        let file_path_clone = file_path.clone();
        let data = data.to_vec();

        task::spawn_blocking(move || -> Result<()> {
            let mut file = File::create(file_path_clone)?;
            file.write_all(&data)?;
            Ok(())
        })
        .await??;

        info!("Stored image: {}", unique_filename);
        Ok(unique_filename)
    }

    /// Retrieve an image by filename
    #[instrument(skip(self), err)]
    pub async fn get_image(&self, filename: &str) -> Result<Vec<u8>> {
        let file_path = self.config.base_dir.join(filename);

        task::spawn_blocking(move || -> Result<Vec<u8>> {
            let mut file = File::open(&file_path)?;
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)?;
            Ok(buffer)
        })
        .await?
    }

    /// Delete an image by filename
    #[instrument(skip(self), err)]
    pub async fn delete_image(&self, filename: &str) -> Result<()> {
        let file_path = self.config.base_dir.join(filename);

        task::spawn_blocking(move || -> Result<()> {
            fs::remove_file(&file_path)?;
            Ok(())
        })
        .await??;

        info!("Deleted image: {}", filename);
        Ok(())
    }

    /// List all images
    #[instrument(skip(self), err)]
    pub async fn list_images(&self) -> Result<Vec<String>> {
        let base_dir = self.config.base_dir.clone();

        task::spawn_blocking(move || -> Result<Vec<String>> {
            let entries = fs::read_dir(base_dir)?;
            let mut filenames = Vec::new();

            for entry in entries {
                let entry = entry?;
                if entry.file_type()?.is_file() {
                    if let Some(filename) = entry.file_name().to_str() {
                        filenames.push(filename.to_string());
                    }
                }
            }

            Ok(filenames)
        })
        .await?
    }

    /// Get the URL for an image
    pub fn get_image_url(&self, filename: &str, base_url: &str) -> String {
        format!("{base_url}/api/images/{filename}")
    }
}
