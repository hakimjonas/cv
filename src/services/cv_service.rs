/// Pure async CV service
///
/// This service handles all CV-related operations using pure async patterns.
use anyhow::Result;

use crate::configuration::AppConfiguration;
use crate::db::Database;
use crate::domain::cv::Cv;

/// Async CV service for all CV-related operations
pub struct CvService {
    config: AppConfiguration,
    database: Option<Database>,
}

impl CvService {
    /// Create a new CV service with configuration
    pub fn new(config: AppConfiguration) -> Self {
        Self {
            config,
            database: None,
        }
    }

    /// Create a new CV service with database support
    pub async fn with_database(config: AppConfiguration) -> Result<Self> {
        let database = Database::new(&config.paths.database)?;
        Ok(Self {
            config,
            database: Some(database),
        })
    }

    /// Load CV data from JSON file
    pub async fn load_cv_from_file(&self) -> Result<Cv> {
        let cv = Cv::from_json(self.config.paths.data_file.as_str())?;
        Ok(cv)
    }

    /// Load CV data from database if available, otherwise from file
    pub async fn load_cv(&self) -> Result<Cv> {
        match &self.database {
            Some(db) => {
                // Try to load from database first
                match db.load_cv_async().await {
                    Ok(old_cv) => {
                        // Convert from old cv_data::Cv to new domain::cv::Cv
                        Ok(self.convert_old_cv_to_domain(old_cv)?)
                    }
                    Err(_) => {
                        // Fallback to file if database load fails
                        self.load_cv_from_file().await
                    }
                }
            }
            None => self.load_cv_from_file().await,
        }
    }

    /// Save CV data to database
    pub async fn save_cv_to_database(&self, cv: &Cv) -> Result<()> {
        match &self.database {
            Some(db) => {
                // Convert from new domain::cv::Cv to old cv_data::Cv for database
                let old_cv = self.convert_domain_cv_to_old(cv)?;
                db.insert_cv_async(&old_cv).await?;
                Ok(())
            }
            None => Err(anyhow::anyhow!("No database configured for CV service")),
        }
    }

    /// Migrate CV data from JSON file to database
    pub async fn migrate_json_to_database(&self) -> Result<()> {
        let cv = self.load_cv_from_file().await?;
        self.save_cv_to_database(&cv).await
    }

    /// Generate HTML output
    pub async fn generate_html(&self, cv: &Cv) -> Result<String> {
        // This would integrate with the HTML generator
        // For now, just validate the CV
        cv.validate()?;

        // TODO: Integrate with actual HTML generator
        Ok(format!("HTML output for CV: {}", cv.personal_info.name))
    }

    /// Generate PDF output
    pub async fn generate_pdf(&self, cv: &Cv) -> Result<Vec<u8>> {
        // This would integrate with the PDF/Typst generator
        // For now, just validate the CV
        cv.validate()?;

        // TODO: Integrate with actual PDF generator
        Ok(vec![]) // Placeholder
    }

    /// Validate CV data
    pub async fn validate_cv(&self, cv: &Cv) -> Result<()> {
        cv.validate()
    }

    /// Get CV statistics
    pub async fn get_cv_statistics(&self, cv: &Cv) -> Result<CvStatistics> {
        Ok(CvStatistics {
            total_experiences: cv.experiences.len(),
            total_projects: cv.projects.len(),
            total_education: cv.education.len(),
            total_skills: cv.skill_categories.iter().map(|cat| cat.skills.len()).sum(),
            total_technologies: cv.all_technologies().len(),
            years_experience: cv.total_experience_years(),
            current_positions: cv.current_experiences().len(),
        })
    }

    /// Get output paths for generated files
    pub fn get_output_paths(&self) -> crate::configuration::OutputPaths {
        self.config.output_paths()
    }

    /// Check if output directory exists and create if needed
    pub async fn ensure_output_directory(&self) -> Result<()> {
        let output_dir = &self.config.paths.output_dir;
        if !output_dir.exists() {
            tokio::fs::create_dir_all(output_dir).await?;
        }
        Ok(())
    }

    /// Check if a CV file is newer than the last generated output
    pub async fn needs_regeneration(&self) -> Result<bool> {
        let data_file = &self.config.paths.data_file;
        let html_output = self.config.paths.html_output();

        if !html_output.exists() {
            return Ok(true);
        }

        let data_metadata = tokio::fs::metadata(data_file).await?;
        let output_metadata = tokio::fs::metadata(html_output).await?;

        Ok(data_metadata.modified()? > output_metadata.modified()?)
    }

    /// Convert from old cv_data::Cv to new domain::cv::Cv
    /// This is a temporary bridge while we migrate the database layer
    fn convert_old_cv_to_domain(&self, old_cv: crate::cv_data::Cv) -> Result<Cv> {
        // For now, serialize to JSON and deserialize to new format
        // This is not efficient but ensures compatibility
        let json = serde_json::to_string(&old_cv)?;
        let new_cv: Cv = serde_json::from_str(&json)?;
        Ok(new_cv)
    }

    /// Convert from new domain::cv::Cv to old cv_data::Cv
    /// This is a temporary bridge while we migrate the database layer
    fn convert_domain_cv_to_old(&self, new_cv: &Cv) -> Result<crate::cv_data::Cv> {
        // For now, serialize to JSON and deserialize to old format
        // This is not efficient but ensures compatibility
        let json = serde_json::to_string(new_cv)?;
        let old_cv: crate::cv_data::Cv = serde_json::from_str(&json)?;
        Ok(old_cv)
    }
}

/// CV statistics for reporting and display
#[derive(Debug, Clone)]
pub struct CvStatistics {
    pub total_experiences: usize,
    pub total_projects: usize,
    pub total_education: usize,
    pub total_skills: usize,
    pub total_technologies: usize,
    pub years_experience: f32,
    pub current_positions: usize,
}

impl CvStatistics {
    /// Get a summary string of the statistics
    pub fn summary(&self) -> String {
        format!(
            "CV contains {} experiences, {} projects, {} education entries, {} skills across {} technologies. {:.1} years total experience with {} current positions.",
            self.total_experiences,
            self.total_projects,
            self.total_education,
            self.total_skills,
            self.total_technologies,
            self.years_experience,
            self.current_positions
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::configuration::AppConfiguration;

    #[tokio::test]
    async fn test_cv_service_creation() {
        let config = AppConfiguration::default();
        let service = CvService::new(config);

        let paths = service.get_output_paths();
        assert_eq!(paths.html.file_name(), Some("cv.html"));
        assert_eq!(paths.pdf.file_name(), Some("cv.pdf"));
    }

    #[test]
    fn test_cv_statistics() {
        let stats = CvStatistics {
            total_experiences: 5,
            total_projects: 10,
            total_education: 2,
            total_skills: 25,
            total_technologies: 15,
            years_experience: 8.5,
            current_positions: 1,
        };

        let summary = stats.summary();
        assert!(summary.contains("5 experiences"));
        assert!(summary.contains("8.5 years"));
    }
}
