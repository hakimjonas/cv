/*!
 * Image API module
 * This module provides API endpoints for image upload and management
 */

use crate::image_storage::{ImageStorage, ImageStorageConfig};
use axum::{
    extract::{Multipart, Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post, delete},
    Router,
};
use std::sync::Arc;
use tracing::{debug, error, info, instrument};

/// API state for image endpoints
#[derive(Clone)]
pub struct ImageApiState {
    /// Image storage service
    pub storage: Arc<ImageStorage>,
    /// Base URL for image URLs
    pub base_url: String,
}

/// Error type for image API
#[derive(Debug, thiserror::Error)]
pub enum ImageApiError {
    #[error("Storage error: {0}")]
    StorageError(#[from] anyhow::Error),
    
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    
    #[error("Image not found: {0}")]
    NotFound(String),
}

impl IntoResponse for ImageApiError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ImageApiError::StorageError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            ImageApiError::InvalidRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            ImageApiError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
        };
        
        (status, error_message).into_response()
    }
}

/// Result type for image API
type ImageApiResult<T> = Result<T, ImageApiError>;

/// Upload an image
#[instrument(skip(state, multipart), err)]
async fn upload_image(
    State(state): State<Arc<ImageApiState>>,
    mut multipart: Multipart,
) -> ImageApiResult<impl IntoResponse> {
    // Process the multipart form
    let mut image_data = None;
    let mut filename = None;
    
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        ImageApiError::InvalidRequest(format!("Failed to process form: {}", e))
    })? {
        let name = field.name().unwrap_or("").to_string();
        
        if name == "image" {
            // Get the filename
            filename = field.file_name().map(|s| s.to_string());
            
            // Get the data
            image_data = Some(field.bytes().await.map_err(|e| {
                ImageApiError::InvalidRequest(format!("Failed to read image data: {}", e))
            })?);
        }
    }
    
    // Validate that we have both filename and data
    let filename = filename.ok_or_else(|| {
        ImageApiError::InvalidRequest("Missing filename".to_string())
    })?;
    
    let image_data = image_data.ok_or_else(|| {
        ImageApiError::InvalidRequest("Missing image data".to_string())
    })?;
    
    // Store the image
    let stored_filename = state.storage.store_image(&filename, &image_data).await
        .map_err(ImageApiError::StorageError)?;
    
    // Generate the URL
    let url = state.storage.get_image_url(&stored_filename, &state.base_url);
    
    // Return the URL
    info!("Uploaded image: {}", stored_filename);
    Ok((
        StatusCode::CREATED,
        [(header::CONTENT_TYPE, "application/json")],
        serde_json::to_string(&serde_json::json!({
            "filename": stored_filename,
            "url": url
        })).unwrap()
    ))
}

/// Get an image by filename
#[instrument(skip(state), err)]
async fn get_image(
    State(state): State<Arc<ImageApiState>>,
    Path(filename): Path<String>,
) -> ImageApiResult<impl IntoResponse> {
    // Get the image data
    let image_data = state.storage.get_image(&filename).await
        .map_err(|e| {
            if e.to_string().contains("No such file") {
                ImageApiError::NotFound(format!("Image not found: {}", filename))
            } else {
                ImageApiError::StorageError(e)
            }
        })?;
    
    // Determine content type based on extension
    let content_type = match filename.split('.').last() {
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("png") => "image/png",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        _ => "application/octet-stream",
    };
    
    debug!("Retrieved image: {}", filename);
    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, content_type)],
        image_data
    ))
}

/// List all images
#[instrument(skip(state), err)]
async fn list_images(
    State(state): State<Arc<ImageApiState>>,
) -> ImageApiResult<impl IntoResponse> {
    // Get the list of images
    let filenames = state.storage.list_images().await
        .map_err(ImageApiError::StorageError)?;
    
    // Generate URLs for each image
    let images = filenames.iter().map(|filename| {
        let url = state.storage.get_image_url(filename, &state.base_url);
        serde_json::json!({
            "filename": filename,
            "url": url
        })
    }).collect::<Vec<_>>();
    
    debug!("Listed {} images", images.len());
    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        serde_json::to_string(&images).unwrap()
    ))
}

/// Delete an image by filename
#[instrument(skip(state), err)]
async fn delete_image(
    State(state): State<Arc<ImageApiState>>,
    Path(filename): Path<String>,
) -> ImageApiResult<impl IntoResponse> {
    // Delete the image
    state.storage.delete_image(&filename).await
        .map_err(|e| {
            if e.to_string().contains("No such file") {
                ImageApiError::NotFound(format!("Image not found: {}", filename))
            } else {
                ImageApiError::StorageError(e)
            }
        })?;
    
    info!("Deleted image: {}", filename);
    Ok(StatusCode::NO_CONTENT)
}

/// Create the image API router
pub fn create_image_api_router(base_url: &str) -> Router {
    // Create the image storage service
    let storage_config = ImageStorageConfig::default();
    let storage = Arc::new(ImageStorage::new(storage_config).expect("Failed to create image storage"));
    
    // Create the API state
    let state = Arc::new(ImageApiState {
        storage,
        base_url: base_url.to_string(),
    });
    
    // Create the router
    Router::new()
        .route("/api/images", post(upload_image))
        .route("/api/images", get(list_images))
        .route("/api/images/:filename", get(get_image))
        .route("/api/images/:filename", delete(delete_image))
        .with_state(state)
}