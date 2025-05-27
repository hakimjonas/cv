/// Configuration module for the CV generator
///
/// This module provides a centralized place for all configuration settings
/// used throughout the application, following functional programming principles
/// with immutable data structures.
use anyhow::{Context, Result};
use im::HashMap;
use std::path::{Path, PathBuf};

/// Represents the application configuration
#[derive(Debug, Clone)]
pub struct Config {
    /// Path to the CV data JSON file
    pub data_path: PathBuf,

    /// Directory containing static assets
    pub static_dir: PathBuf,

    /// Directory where output files will be written
    pub output_dir: PathBuf,

    /// Path where the HTML CV will be written
    pub html_output: PathBuf,

    /// Temporary file for Typst markup
    pub typst_temp: PathBuf,

    /// Path where the PDF CV will be written
    pub pdf_output: PathBuf,

    /// Additional configuration options
    #[allow(dead_code)]
    pub options: HashMap<String, String>,
}

impl Default for Config {
    /// Creates a default configuration
    fn default() -> Self {
        let output_dir = PathBuf::from("dist");

        Self {
            data_path: PathBuf::from("data/cv_data.json"),
            static_dir: PathBuf::from("static"),
            output_dir: output_dir.clone(),
            html_output: output_dir.join("cv.html"),
            typst_temp: PathBuf::from("temp_cv.typ"),
            pdf_output: output_dir.join("cv.pdf"),
            options: HashMap::new(),
        }
    }
}

impl Config {
    /// Creates a new configuration with custom settings
    #[allow(dead_code)]
    pub fn new(
        data_path: PathBuf,
        static_dir: PathBuf,
        output_dir: PathBuf,
        typst_temp: PathBuf,
    ) -> Self {
        let html_output = output_dir.join("cv.html");
        let pdf_output = output_dir.join("cv.pdf");

        Self {
            data_path,
            static_dir,
            output_dir,
            html_output,
            typst_temp,
            pdf_output,
            options: HashMap::new(),
        }
    }

    /// Adds an option to the configuration
    #[allow(dead_code)]
    pub fn with_option(self, key: &str, value: &str) -> Self {
        let mut options = self.options.clone();
        options.insert(key.to_string(), value.to_string());

        Self { options, ..self }
    }

    /// Gets a path as a string, with proper error handling
    pub fn path_to_string(&self, path: &Path) -> Result<String> {
        path.to_str()
            .map(String::from)
            .context(format!("Failed to convert path to string: {:?}", path))
    }

    /// Gets the HTML output path as a string
    pub fn html_output_str(&self) -> Result<String> {
        self.path_to_string(&self.html_output)
    }

    /// Gets the PDF output path as a string
    pub fn pdf_output_str(&self) -> Result<String> {
        self.path_to_string(&self.pdf_output)
    }

    /// Gets the Typst temp path as a string
    pub fn typst_temp_str(&self) -> Result<String> {
        self.path_to_string(&self.typst_temp)
    }

    /// Gets the static directory path as a string
    pub fn static_dir_str(&self) -> Result<String> {
        self.path_to_string(&self.static_dir)
    }

    /// Gets the output directory path as a string
    pub fn output_dir_str(&self) -> Result<String> {
        self.path_to_string(&self.output_dir)
    }

    /// Gets the data path as a string
    pub fn data_path_str(&self) -> Result<String> {
        self.path_to_string(&self.data_path)
    }
}
