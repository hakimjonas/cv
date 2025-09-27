//! Asset processing and file operations
//!
//! This module handles copying static assets, directory operations, and file management
//! for the HTML generation process.

use anyhow::{Context, Result};
use im::Vector;
use std::fs;
use std::path::{Path, PathBuf};

/// Filesystem entry type for directory traversal
#[derive(Debug, Clone)]
pub struct FsEntry {
    pub path: PathBuf,
    pub is_dir: bool,
}

/// Copies static assets from source to destination directory, excluding specified files
///
/// # Arguments
///
/// * `static_dir` - Source directory containing static assets
/// * `output_dir` - Destination directory for copied assets
/// * `exclude` - Array of file/directory names to exclude
///
/// # Returns
///
/// A Result indicating success or failure
pub fn copy_static_assets_except(
    static_dir: &str,
    output_dir: &str,
    exclude: &[&str],
) -> Result<()> {
    println!("Copying static assets from {static_dir} to {output_dir} (excluding: {exclude:?})");
    copy_dir_recursively_except(static_dir, output_dir, exclude)?;
    println!("Static assets copied successfully");
    Ok(())
}

/// Recursively copies a directory and its contents, excluding specified items
///
/// # Arguments
///
/// * `src` - Source directory path
/// * `dst` - Destination directory path
/// * `exclude` - Array of file/directory names to exclude
///
/// # Returns
///
/// A Result indicating success or failure
fn copy_dir_recursively_except(src: &str, dst: &str, exclude: &[&str]) -> Result<()> {
    let src_path = Path::new(src);
    if !src_path.exists() {
        return Err(anyhow::anyhow!("Source directory does not exist: {src}"));
    }

    let entries = list_directory_entries(src)?;

    for entry in entries.iter() {
        let entry_name = entry
            .path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");

        // Skip excluded files/directories
        if exclude.contains(&entry_name) {
            println!("Skipping excluded file: {entry_name}");
            continue;
        }

        if entry.is_dir {
            copy_directory_except(&entry.path, dst, exclude)?;
        } else {
            copy_file(&entry.path, dst)?;
        }
    }

    Ok(())
}

/// Lists all entries in a directory
///
/// # Arguments
///
/// * `dir_path` - Path to the directory to list
///
/// # Returns
///
/// A Result containing a Vector of filesystem entries
fn list_directory_entries(dir_path: &str) -> Result<Vector<FsEntry>> {
    let mut entries = Vector::new();
    let dir =
        fs::read_dir(dir_path).with_context(|| format!("Failed to read directory: {dir_path}"))?;

    for entry in dir {
        let entry =
            entry.with_context(|| format!("Failed to read directory entry in {dir_path}"))?;
        let path = entry.path();
        let is_dir = path.is_dir();

        entries.push_back(FsEntry { path, is_dir });
    }

    Ok(entries)
}

/// Copies a single file to the destination directory
///
/// # Arguments
///
/// * `src_path` - Source file path
/// * `dst_dir` - Destination directory
///
/// # Returns
///
/// A Result indicating success or failure
fn copy_file(src_path: &Path, dst_dir: &str) -> Result<()> {
    let file_name = src_path
        .file_name()
        .context("Failed to get file name")?
        .to_str()
        .context("Invalid UTF-8 in file name")?;

    let dst_path = Path::new(dst_dir).join(file_name);

    // Create destination directory if it doesn't exist
    if let Some(parent) = dst_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }

    fs::copy(src_path, &dst_path).with_context(|| {
        format!(
            "Failed to copy file from {} to {}",
            src_path.display(),
            dst_path.display()
        )
    })?;

    Ok(())
}

/// Copies a directory to the destination directory, excluding specified items
///
/// # Arguments
///
/// * `src_path` - Source directory path
/// * `dst_dir` - Destination directory
/// * `exclude` - Array of file/directory names to exclude
///
/// # Returns
///
/// A Result indicating success or failure
fn copy_directory_except(src_path: &Path, dst_dir: &str, exclude: &[&str]) -> Result<()> {
    let dir_name = src_path
        .file_name()
        .context("Failed to get directory name")?
        .to_str()
        .context("Invalid UTF-8 in directory name")?;

    let new_dst_dir = Path::new(dst_dir).join(dir_name);
    let new_dst_str = new_dst_dir
        .to_str()
        .context("Failed to convert destination path to string")?;

    copy_dir_recursively_except(
        src_path
            .to_str()
            .context("Failed to convert source path to string")?,
        new_dst_str,
        exclude,
    )
}
