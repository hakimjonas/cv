use crate::blog_data::{BlogManager, BlogPost, Tag};
use crate::db::Database;
use anyhow::Result;
use axum::{
    Router,
    extract::{Path, State},
    http::{Method, StatusCode},
    response::{IntoResponse, Json, Response},
    routing::{delete, get, post, put},
};
use im::Vector;
use std::{path::PathBuf, sync::Arc};
use thiserror::Error;
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
    timeout::TimeoutLayer,
};

/// API error types
#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Internal server error: {0}")]
    InternalError(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ApiError::ValidationError(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::DatabaseError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            ApiError::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        (status, error_message).into_response()
    }
}

/// API state containing the database connection and blog manager
pub struct ApiState {
    pub blog_manager: Arc<BlogManager>,
    pub db: Database,
}

/// Gets all blog posts
#[axum::debug_handler]
async fn get_all_posts(
    State(state): State<Arc<ApiState>>,
) -> Result<Json<Vector<BlogPost>>, StatusCode> {
    state
        .blog_manager
        .get_all_posts()
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// Gets a blog post by slug
#[axum::debug_handler]
async fn get_post_by_slug(
    State(state): State<Arc<ApiState>>,
    Path(slug): Path<String>,
) -> Result<Json<BlogPost>, StatusCode> {
    state
        .blog_manager
        .get_post_by_slug(&slug)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
        .and_then(|post_opt| post_opt.ok_or(StatusCode::NOT_FOUND))
        .map(Json)
}

/// Creates a new blog post
#[axum::debug_handler]
async fn create_post(
    State(state): State<Arc<ApiState>>,
    Json(post): Json<BlogPost>,
) -> Result<(StatusCode, Json<BlogPost>), (StatusCode, String)> {
    println!("=== CREATE POST REQUEST ===");
    println!("Received create post request: {:?}", post);

    // Validate required fields
    if post.title.is_empty()
        || post.slug.is_empty()
        || post.content.is_empty()
        || post.date.is_empty()
        || post.author.is_empty()
    {
        println!("❌ Validation failed: Missing required fields");
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            "Missing required fields: title, slug, content, date, author must be non-empty"
                .to_string(),
        ));
    }

    println!("✅ Validation passed");

    // Try to create or update the post
    println!("Creating/updating post in database...");
    let post_id = match state.blog_manager.create_or_update_post(&post) {
        Ok(id) => {
            println!("✅ Post created/updated successfully with ID: {}", id);
            id
        }
        Err(e) => {
            // Special handling for SQLite locking errors which might actually indicate success
            if e.to_string().contains("locked") || e.to_string().contains("busy") {
                println!(
                    "⚠️ Warning: Database lock detected during post creation, but operation may have succeeded"
                );
                // Try to see if the post was actually created despite the error
                if let Ok(Some(created_post)) = state.blog_manager.get_post_by_slug(&post.slug) {
                    println!("✅ Post was successfully created despite transaction lock error");
                    return Ok((StatusCode::CREATED, Json(created_post)));
                }
            }

            println!("❌ Error creating post: {:?}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to create post: {}", e),
            ));
        }
    };

    // Get the created post from the database to ensure all fields are properly set
    println!("Retrieving created post from database...");
    let updated_post = match state.blog_manager.get_post_by_id(post_id) {
        Ok(Some(post)) => {
            println!("✅ Successfully retrieved created post with ID {}", post_id);
            post
        }
        Ok(None) => {
            println!(
                "⚠️  Warning: Post with ID {} was created but not found when retrieving it",
                post_id
            );
            println!("Database may have consistency issues - using constructed post instead");

            BlogPost {
                id: Some(post_id),
                ..post
            }
        }
        Err(e) => {
            println!("⚠️  Warning: Error retrieving created post: {:?}", e);
            println!("Will return constructed post instead of retrieved one");

            BlogPost {
                id: Some(post_id),
                ..post
            }
        }
    };

    println!("✅ Successfully created post with ID: {}", post_id);
    println!("Response structure prepared successfully");

    // Create a simplified response for debugging
    let response_json = Json(updated_post);

    println!("=== CREATE POST RESPONSE ===");
    println!("Sending successful response");

    Ok((StatusCode::CREATED, response_json))
}

/// Updates an existing blog post
#[axum::debug_handler]
async fn update_post(
    State(state): State<Arc<ApiState>>,
    Path(slug): Path<String>,
    Json(post): Json<BlogPost>,
) -> Result<Json<BlogPost>, (StatusCode, String)> {
    println!("Received update post request for slug {}: {:?}", slug, post);

    // Validate required fields
    if post.title.is_empty()
        || post.slug.is_empty()
        || post.content.is_empty()
        || post.date.is_empty()
        || post.author.is_empty()
    {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            "Missing required fields: title, slug, content, date, author must be non-empty"
                .to_string(),
        ));
    }

    // First, check if post exists
    let existing_post = match state.blog_manager.get_post_by_slug(&slug) {
        Ok(Some(post)) => post,
        Ok(None) => {
            return Err((
                StatusCode::NOT_FOUND,
                format!("Post with slug '{}' not found", slug),
            ));
        }
        Err(e) => {
            println!("Error getting post with slug {}: {:?}", slug, e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to retrieve post: {}", e),
            ));
        }
    };

    // Create a new post with the existing ID
    let post_to_update = BlogPost {
        id: existing_post.id,
        ..post
    };

    // Update the post
    match state.blog_manager.create_or_update_post(&post_to_update) {
        Ok(_) => {}
        Err(e) => {
            println!("Error updating post: {:?}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to update post: {}", e),
            ));
        }
    };

    // Return the updated post
    match state.blog_manager.get_post_by_slug(&post_to_update.slug) {
        Ok(Some(updated_post)) => {
            println!("Successfully updated post with ID: {:?}", updated_post.id);
            Ok(Json(updated_post))
        }
        Ok(None) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Post was updated but could not be retrieved".to_string(),
        )),
        Err(e) => {
            println!("Error retrieving updated post: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to retrieve updated post: {}", e),
            ))
        }
    }
}

/// Deletes a blog post
#[axum::debug_handler]
async fn delete_post(
    State(state): State<Arc<ApiState>>,
    Path(slug): Path<String>,
) -> Result<StatusCode, StatusCode> {
    // First, check if post exists
    let existing_post = state
        .blog_manager
        .get_post_by_slug(&slug)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
        .and_then(|post_opt| post_opt.ok_or(StatusCode::NOT_FOUND))?;

    // Delete the post
    state
        .blog_manager
        .delete_post(existing_post.id.unwrap())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}

/// Gets all tags
#[axum::debug_handler]
#[allow(dead_code)]
async fn get_all_tags(State(state): State<Arc<ApiState>>) -> Result<Json<Vector<Tag>>, StatusCode> {
    state
        .blog_manager
        .get_all_tags()
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// Gets all published blog posts
#[axum::debug_handler]
async fn get_published_posts(
    State(state): State<Arc<ApiState>>,
) -> Result<Json<Vector<BlogPost>>, StatusCode> {
    state
        .blog_manager
        .get_published_posts()
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// Gets all featured blog posts
#[axum::debug_handler]
async fn get_featured_posts(
    State(state): State<Arc<ApiState>>,
) -> Result<Json<Vector<BlogPost>>, StatusCode> {
    state
        .blog_manager
        .get_featured_posts()
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// Gets posts by tag
#[axum::debug_handler]
async fn get_posts_by_tag(
    State(state): State<Arc<ApiState>>,
    Path(tag_slug): Path<String>,
) -> Result<Json<Vector<BlogPost>>, StatusCode> {
    state
        .blog_manager
        .get_posts_by_tag(&tag_slug)
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// Serves a simple HTML page at the root
async fn root_handler() -> impl axum::response::IntoResponse {
    axum::response::Html(
        r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>Blog API Server</title>
        <style>
            body {
                font-family: Arial, sans-serif;
                line-height: 1.6;
                margin: 0;
                padding: 30px;
                max-width: 800px;
                margin: 0 auto;
            }
            h1 {
                color: #333;
                border-bottom: 1px solid #eee;
                padding-bottom: 10px;
            }
            ul {
                padding-left: 20px;
            }
            li {
                margin-bottom: 10px;
            }
            code {
                background-color: #f4f4f4;
                padding: 2px 5px;
                border-radius: 3px;
            }
            .endpoint {
                font-weight: bold;
            }
            #api-results {
                background-color: #f9f9f9;
                padding: 15px;
                border-radius: 5px;
                border: 1px solid #ddd;
                margin-top: 20px;
                white-space: pre-wrap;
                font-family: monospace;
                max-height: 300px;
                overflow-y: auto;
            }
            button {
                background-color: #4CAF50;
                color: white;
                padding: 8px 15px;
                border: none;
                border-radius: 4px;
                cursor: pointer;
                font-size: 14px;
                margin-top: 10px;
            }
            button:hover {
                background-color: #45a049;
            }
        </style>
        <script src="/static/js/blog-debug.js"></script>
    </head>
    <body>
        <h1>Blog API Server</h1>
        <p>Welcome to the Blog API Server. The following endpoints are available:</p>

        <ul>
            <li><span class="endpoint">GET /api/blog</span> - Get all blog posts</li>
            <li><span class="endpoint">POST /api/blog</span> - Create a new blog post</li>
            <li><span class="endpoint">GET /api/blog/{slug}</span> - Get a blog post by slug</li>
            <li><span class="endpoint">PUT /api/blog/{slug}</span> - Update a blog post</li>
            <li><span class="endpoint">DELETE /api/blog/{slug}</span> - Delete a blog post</li>
            <li><span class="endpoint">GET /api/blog/tags</span> - Get all tags</li>
            <li><span class="endpoint">GET /api/blog/published</span> - Get all published posts</li>
            <li><span class="endpoint">GET /api/blog/featured</span> - Get all featured posts</li>
            <li><span class="endpoint">GET /api/blog/tag/{tag_slug}</span> - Get posts by tag</li>
        </ul>

        <p>Try accessing <a href="/api/blog">/api/blog</a> to see all blog posts.</p>
        <p>For a more user-friendly interface, check out the <a href="/static/blog-client.html">Blog Client</a>.</p>
        <p>Having issues? Use the <a href="/static/blog-debug.html">Blog Debug Tool</a> to diagnose problems.</p>

        <div>
            <button onclick="testAndDisplayConnection()">Test API Connection</button>
            <div id="api-results">API results will appear here...</div>
        </div>

        <script>
            async function testAndDisplayConnection() {
                const resultsDiv = document.getElementById('api-results');
                resultsDiv.textContent = 'Testing API connection...';

                try {
                    const result = await testApiConnection();
                    if (result.success) {
                        resultsDiv.textContent = `Connection successful!\nSource: ${result.source}\nFound ${result.posts.length} posts.\n\nFirst post: ${JSON.stringify(result.posts[0], null, 2)}`;
                    } else {
                        resultsDiv.textContent = 'Could not connect to any data source.';
                    }
                } catch (error) {
                    resultsDiv.textContent = `Error: ${error.message}`;
                }
            }
        </script>
    </body>
    </html>
    "#,
    )
}

/// Creates the blog API router
pub fn create_blog_api_router(db_path: PathBuf) -> Result<Router> {
    let blog_manager = Arc::new(BlogManager::new(&db_path)?);
    let db = Database::new(&db_path)?;
    let state = Arc::new(ApiState { blog_manager, db });

    // Create a static file service for the static directory
    let static_service = ServeDir::new("static");

    // Configure CORS to allow all origins
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(Any)
        .max_age(std::time::Duration::from_secs(86400)); // 24 hours timeout

    // Add a simple diagnostic handler
    async fn diagnostic_handler() -> impl axum::response::IntoResponse {
        let test_data = serde_json::json!({
            "status": "ok",
            "message": "Blog API is functioning",
            "timestamp": chrono::Local::now().to_rfc3339()
        });
        Json(test_data)
    }

    // Route order matters! More specific routes need to come before less specific ones
    // to avoid route conflicts, especially with path parameters
    let router = Router::new()
        // Root route
        .route("/", get(root_handler))
        // Diagnostic endpoint
        .route("/api/blog/test", get(diagnostic_handler))
        // Blog specific routes need to come before routes with parameters
        .route("/api/blog/tags", get(get_all_tags))
        .route("/api/blog/published", get(get_published_posts))
        .route("/api/blog/featured", get(get_featured_posts))
        .route("/api/blog/tag/{tag_slug}", get(get_posts_by_tag))
        // Basic blog routes
        .route("/api/blog", get(get_all_posts))
        .route("/api/blog", post(create_post))
        .route("/api/blog/{slug}", get(get_post_by_slug))
        .route("/api/blog/{slug}", put(update_post))
        .route("/api/blog/{slug}", delete(delete_post))
        // Serve static files
        .nest_service("/static", static_service)
        // Apply CORS middleware
        .layer(cors)
        // Add request timeout middleware - increased to 120 seconds for database operations
        .layer(TimeoutLayer::new(std::time::Duration::from_secs(120)))
        .with_state(state);

    Ok(router)
}
