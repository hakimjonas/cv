/*!
 * Feature flags module
 * This module provides functionality for feature flags to enable or disable features
 */

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, RwLock};
use tracing::{error, info, instrument, warn};

/// Feature flag configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlagConfig {
    /// Map of feature names to their enabled state
    pub features: HashMap<String, bool>,
}

impl Default for FeatureFlagConfig {
    fn default() -> Self {
        let mut features = HashMap::new();
        
        // Default feature flags
        features.insert("markdown_editor".to_string(), true);
        features.insert("image_upload".to_string(), true);
        
        Self { features }
    }
}

/// Feature flag service
#[derive(Debug)]
pub struct FeatureFlags {
    /// Feature flag configuration
    config: Arc<RwLock<FeatureFlagConfig>>,
    /// Path to the feature flag configuration file
    config_path: Option<String>,
}

impl FeatureFlags {
    /// Create a new feature flag service with the given configuration
    pub fn new(config: FeatureFlagConfig, config_path: Option<String>) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            config_path,
        }
    }
    
    /// Create a new feature flag service with default configuration
    pub fn default() -> Self {
        Self::new(FeatureFlagConfig::default(), None)
    }
    
    /// Load feature flag configuration from a file
    #[instrument(skip(config_path), err)]
    pub fn from_file(config_path: &str) -> Result<Self, anyhow::Error> {
        let config_str = fs::read_to_string(config_path)?;
        let config: FeatureFlagConfig = serde_json::from_str(&config_str)?;
        
        Ok(Self::new(config, Some(config_path.to_string())))
    }
    
    /// Save feature flag configuration to a file
    #[instrument(skip(self), err)]
    pub fn save_to_file(&self) -> Result<(), anyhow::Error> {
        if let Some(config_path) = &self.config_path {
            let config = self.config.read().map_err(|e| {
                anyhow::anyhow!("Failed to acquire read lock on feature flag config: {}", e)
            })?;
            
            let config_str = serde_json::to_string_pretty(&*config)?;
            fs::write(config_path, config_str)?;
            
            info!("Saved feature flag configuration to {}", config_path);
            Ok(())
        } else {
            Err(anyhow::anyhow!("No configuration file path specified"))
        }
    }
    
    /// Check if a feature is enabled
    #[instrument(skip(self), err)]
    pub fn is_enabled(&self, feature_name: &str) -> Result<bool, anyhow::Error> {
        let config = self.config.read().map_err(|e| {
            anyhow::anyhow!("Failed to acquire read lock on feature flag config: {}", e)
        })?;
        
        Ok(*config.features.get(feature_name).unwrap_or(&false))
    }
    
    /// Enable a feature
    #[instrument(skip(self), err)]
    pub fn enable_feature(&self, feature_name: &str) -> Result<(), anyhow::Error> {
        let mut config = self.config.write().map_err(|e| {
            anyhow::anyhow!("Failed to acquire write lock on feature flag config: {}", e)
        })?;
        
        config.features.insert(feature_name.to_string(), true);
        
        info!("Enabled feature: {}", feature_name);
        
        // Save the configuration if a file path is specified
        drop(config); // Release the write lock before saving
        if self.config_path.is_some() {
            self.save_to_file()?;
        }
        
        Ok(())
    }
    
    /// Disable a feature
    #[instrument(skip(self), err)]
    pub fn disable_feature(&self, feature_name: &str) -> Result<(), anyhow::Error> {
        let mut config = self.config.write().map_err(|e| {
            anyhow::anyhow!("Failed to acquire write lock on feature flag config: {}", e)
        })?;
        
        config.features.insert(feature_name.to_string(), false);
        
        info!("Disabled feature: {}", feature_name);
        
        // Save the configuration if a file path is specified
        drop(config); // Release the write lock before saving
        if self.config_path.is_some() {
            self.save_to_file()?;
        }
        
        Ok(())
    }
    
    /// Get all feature flags
    #[instrument(skip(self), err)]
    pub fn get_all_features(&self) -> Result<HashMap<String, bool>, anyhow::Error> {
        let config = self.config.read().map_err(|e| {
            anyhow::anyhow!("Failed to acquire read lock on feature flag config: {}", e)
        })?;
        
        Ok(config.features.clone())
    }
}

/// Feature flag middleware for checking if a feature is enabled
pub mod middleware {
    use super::*;
    use axum::{
        http::{Request, StatusCode},
        middleware::Next,
        response::Response,
    };
    
    /// Middleware to require a feature to be enabled
    pub async fn require_feature<B>(
        feature_flags: Arc<FeatureFlags>,
        feature_name: &'static str,
        req: Request<B>,
        next: Next,
    ) -> Result<Response, StatusCode> {
        // Check if the feature is enabled
        match feature_flags.is_enabled(feature_name) {
            Ok(true) => {
                // Feature is enabled, continue with the request
                // Convert the request to use axum::body::Body
                let (parts, _) = req.into_parts();
                let req = Request::from_parts(parts, axum::body::Body::empty());
                Ok(next.run(req).await)
            }
            Ok(false) => {
                // Feature is disabled
                warn!("Access to disabled feature attempted: {}", feature_name);
                Err(StatusCode::NOT_FOUND)
            }
            Err(e) => {
                // Error checking feature flag
                error!("Error checking feature flag: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}

/// Rollback procedures for features
pub mod rollback {
    use super::*;
    
    /// Rollback procedure for a feature
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct RollbackProcedure {
        /// Name of the feature
        pub feature_name: String,
        /// Description of the rollback procedure
        pub description: String,
        /// Steps to rollback the feature
        pub steps: Vec<String>,
    }
    
    /// Rollback manager
    #[derive(Debug)]
    pub struct RollbackManager {
        /// Map of feature names to rollback procedures
        procedures: HashMap<String, RollbackProcedure>,
    }
    
    impl RollbackManager {
        /// Create a new rollback manager
        pub fn new() -> Self {
            Self {
                procedures: HashMap::new(),
            }
        }
        
        /// Register a rollback procedure for a feature
        pub fn register_procedure(&mut self, procedure: RollbackProcedure) {
            self.procedures.insert(procedure.feature_name.clone(), procedure);
        }
        
        /// Get the rollback procedure for a feature
        pub fn get_procedure(&self, feature_name: &str) -> Option<&RollbackProcedure> {
            self.procedures.get(feature_name)
        }
        
        /// Get all rollback procedures
        pub fn get_all_procedures(&self) -> Vec<&RollbackProcedure> {
            self.procedures.values().collect()
        }
    }
    
    impl Default for RollbackManager {
        fn default() -> Self {
            let mut manager = Self::new();
            
            // Register rollback procedures for default features
            
            // Markdown Editor
            manager.register_procedure(RollbackProcedure {
                feature_name: "markdown_editor".to_string(),
                description: "Rollback procedure for the Markdown editor feature".to_string(),
                steps: vec![
                    "Disable the markdown_editor feature flag".to_string(),
                    "Update existing Markdown content to HTML using the render_content method".to_string(),
                    "Set content_format to HTML for all blog posts".to_string(),
                ],
            });
            
            // Image Upload
            manager.register_procedure(RollbackProcedure {
                feature_name: "image_upload".to_string(),
                description: "Rollback procedure for the image upload feature".to_string(),
                steps: vec![
                    "Disable the image_upload feature flag".to_string(),
                    "Remove image upload endpoints from the API".to_string(),
                    "Keep existing images accessible but prevent new uploads".to_string(),
                ],
            });
            
            manager
        }
    }
}