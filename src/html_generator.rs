use anyhow::{Context, Result};
use askama::Template;
use im::Vector;
use std::fs;
use std::path::{Path, PathBuf};

use crate::cv_data::Cv;

/// Template for the CV HTML page
#[derive(Template)]
#[template(path = "cv.html")]
struct CvTemplate<'a> {
    cv: &'a Cv,
}

/// Template for the index HTML page
#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate<'a> {
    cv: &'a Cv,
}

/// Generate HTML from CV data and save it to the specified path
///
/// # Arguments
///
/// * `cv` - The CV data to generate HTML from
/// * `output_path` - Path where the CV HTML will be written
///
/// # Returns
///
/// A Result indicating success or failure
pub fn generate_html(cv: &Cv, output_path: &str) -> Result<()> {
    // Generate CV HTML
    generate_cv_html(cv, output_path)?;

    // Generate index HTML
    let index_path = Path::new(output_path)
        .parent()
        .context("Failed to get parent directory")?
        .join("index.html")
        .to_str()
        .context("Failed to convert path to string")?
        .to_string();

    generate_index_html(cv, &index_path)?;

    Ok(())
}

/// Generate CV HTML from CV data and save it to the specified path
///
/// # Arguments
///
/// * `cv` - The CV data to generate HTML from
/// * `output_path` - Path where the HTML will be written
///
/// # Returns
///
/// A Result indicating success or failure
fn generate_cv_html(cv: &Cv, output_path: &str) -> Result<()> {
    // Create the template with the CV data
    let template = CvTemplate { cv };

    // Render the template to HTML
    let html = template
        .render()
        .context("Failed to render CV HTML template")?;

    // Ensure the output directory exists and write the HTML
    ensure_parent_dir_exists(output_path)?;
    write_file(output_path, &html)?;

    Ok(())
}

/// Generate index HTML from CV data and save it to the specified path
///
/// # Arguments
///
/// * `cv` - The CV data to generate HTML from
/// * `output_path` - Path where the HTML will be written
///
/// # Returns
///
/// A Result indicating success or failure
fn generate_index_html(cv: &Cv, output_path: &str) -> Result<()> {
    // Create the template with the CV data
    let template = IndexTemplate { cv };

    // Render the template to HTML
    let html = template
        .render()
        .context("Failed to render index HTML template")?;

    // Ensure the output directory exists and write the HTML
    ensure_parent_dir_exists(output_path)?;
    write_file(output_path, &html)?;

    Ok(())
}

/// Ensures that the parent directory of a file path exists
///
/// # Arguments
///
/// * `file_path` - Path to a file whose parent directory should exist
///
/// # Returns
///
/// A Result indicating success or failure
fn ensure_parent_dir_exists(file_path: &str) -> Result<()> {
    Path::new(file_path)
        .parent()
        .map(fs::create_dir_all)
        .transpose()
        .context("Failed to create parent directory")?;

    Ok(())
}

/// Writes content to a file
///
/// # Arguments
///
/// * `path` - Path where the content will be written
/// * `content` - Content to write to the file
///
/// # Returns
///
/// A Result indicating success or failure
fn write_file(path: &str, content: &str) -> Result<()> {
    fs::write(path, content).with_context(|| format!("Failed to write to {}", path))
}

/// Copy static assets to the output directory
///
/// # Arguments
///
/// * `static_dir` - Directory containing static assets
/// * `output_dir` - Directory where assets will be copied
///
/// # Returns
///
/// A Result indicating success or failure
#[allow(dead_code)]
pub fn copy_static_assets(static_dir: &str, output_dir: &str) -> Result<()> {
    copy_static_assets_except(static_dir, output_dir, &[])
}

/// Copy static assets to the output directory, excluding specified files
///
/// # Arguments
///
/// * `static_dir` - Directory containing static assets
/// * `output_dir` - Directory where assets will be copied
/// * `exclude` - Array of filenames to exclude from copying
///
/// # Returns
///
/// A Result indicating success or failure
pub fn copy_static_assets_except(
    static_dir: &str,
    output_dir: &str,
    exclude: &[&str],
) -> Result<()> {
    // Ensure the output directory exists
    fs::create_dir_all(output_dir).context("Failed to create output directory")?;

    // Recursively copy the static directory, excluding specified files
    copy_dir_recursively_except(static_dir, output_dir, exclude)
}

/// Represents a file system entry (file or directory)
#[derive(Debug, Clone)]
enum FsEntry {
    File(PathBuf),
    Directory(PathBuf),
}

/// Recursively copy a directory and its contents
///
/// # Arguments
///
/// * `src` - Source directory
/// * `dst` - Destination directory
///
/// # Returns
///
/// A Result indicating success or failure
#[allow(dead_code)]
fn copy_dir_recursively(src: &str, dst: &str) -> Result<()> {
    copy_dir_recursively_except(src, dst, &[])
}

/// Recursively copy a directory and its contents, excluding specified files
///
/// # Arguments
///
/// * `src` - Source directory
/// * `dst` - Destination directory
/// * `exclude` - Array of filenames to exclude from copying
///
/// # Returns
///
/// A Result indicating success or failure
fn copy_dir_recursively_except(src: &str, dst: &str, exclude: &[&str]) -> Result<()> {
    // Get all entries in the source directory
    let entries = list_directory_entries(src)?;

    // Process each entry using functional patterns
    entries.iter().try_for_each(|entry| match entry {
        FsEntry::File(path) => {
            // Check if the file should be excluded
            let file_name = path
                .file_name()
                .context("Failed to get file name")?
                .to_str()
                .context("Failed to convert file name to string")?;

            if exclude.contains(&file_name) {
                println!("Skipping excluded file: {}", file_name);
                Ok(())
            } else {
                copy_file(path, dst)
            }
        }
        FsEntry::Directory(path) => copy_directory_except(path, dst, exclude),
    })
}

/// Lists all entries in a directory
///
/// # Arguments
///
/// * `dir_path` - Path to the directory
///
/// # Returns
///
/// A Result containing a Vector of FsEntry or an error
fn list_directory_entries(dir_path: &str) -> Result<Vector<FsEntry>> {
    let entries = fs::read_dir(dir_path)
        .with_context(|| format!("Failed to read directory: {}", dir_path))?;

    // Convert DirEntry stream to Vector<FsEntry> using functional patterns
    let result = entries
        .filter_map(|entry_result| {
            entry_result
                .map_err(|e| anyhow::anyhow!("Failed to read directory entry: {}", e))
                .ok()
                .and_then(|entry| {
                    let path = entry.path();
                    if path.is_file() {
                        Some(FsEntry::File(path))
                    } else if path.is_dir() {
                        Some(FsEntry::Directory(path))
                    } else {
                        // Skip other types of entries (symlinks, etc.)
                        None
                    }
                })
        })
        .collect::<Vector<_>>();

    Ok(result)
}

/// Copies a file to a destination directory
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
        .context("Failed to convert file name to string")?;

    let dst_path = Path::new(dst_dir).join(file_name);

    fs::copy(src_path, &dst_path).with_context(|| {
        format!(
            "Failed to copy {} to {}",
            src_path.display(),
            dst_path.display()
        )
    })?;

    Ok(())
}

/// Copies a directory to a destination directory
///
/// # Arguments
///
/// * `src_path` - Source directory path
/// * `dst_dir` - Destination directory
///
/// # Returns
///
/// A Result indicating success or failure
#[allow(dead_code)]
fn copy_directory(src_path: &Path, dst_dir: &str) -> Result<()> {
    copy_directory_except(src_path, dst_dir, &[])
}

/// Copies a directory to a destination directory, excluding specified files
///
/// # Arguments
///
/// * `src_path` - Source directory path
/// * `dst_dir` - Destination directory
/// * `exclude` - Array of filenames to exclude from copying
///
/// # Returns
///
/// A Result indicating success or failure
fn copy_directory_except(src_path: &Path, dst_dir: &str, exclude: &[&str]) -> Result<()> {
    let dir_name = src_path
        .file_name()
        .context("Failed to get directory name")?
        .to_str()
        .context("Failed to convert directory name to string")?;

    let dst_path = Path::new(dst_dir).join(dir_name);

    // Create the destination directory
    fs::create_dir_all(&dst_path)
        .with_context(|| format!("Failed to create directory: {}", dst_path.display()))?;

    // Recursively copy the subdirectory
    copy_dir_recursively_except(
        src_path
            .to_str()
            .context("Failed to convert path to string")?,
        dst_path
            .to_str()
            .context("Failed to convert path to string")?,
        exclude,
    )
}
