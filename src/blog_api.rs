use crate::blog_data::{BlogPost, Tag};
use crate::blog_error::BlogError;
use crate::db::{BlogRepository, Database};

/// API result type
type ApiResult<T> = std::result::Result<T, ApiError>;
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
use tracing::{debug, error, info, instrument, warn};

/// Convert from repository::BlogPost to blog_data::BlogPost
fn repo_to_api_post(repo_post: crate::db::repository::BlogPost) -> BlogPost {
    BlogPost {
        id: repo_post.id,
        title: repo_post.title,
        slug: repo_post.slug,
        date: repo_post.date,
        author: repo_post.author,
        excerpt: repo_post.excerpt,
        content: repo_post.content,
        published: repo_post.published,
        featured: repo_post.featured,
        image: repo_post.image,
        tags: repo_post.tags.into_iter().map(repo_to_api_tag).collect(),
        metadata: repo_post.metadata.into_iter().collect(),
    }
}

/// Convert from blog_data::BlogPost to repository::BlogPost
fn api_to_repo_post(api_post: &BlogPost) -> crate::db::repository::BlogPost {
    crate::db::repository::BlogPost {
        id: api_post.id,
        title: api_post.title.clone(),
        slug: api_post.slug.clone(),
        date: api_post.date.clone(),
        author: api_post.author.clone(),
        excerpt: api_post.excerpt.clone(),
        content: api_post.content.clone(),
        published: api_post.published,
        featured: api_post.featured,
        image: api_post.image.clone(),
        tags: api_post.tags.iter().map(api_to_repo_tag).collect(),
        metadata: api_post
            .metadata
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect(),
    }
}

/// Convert from repository::Tag to blog_data::Tag
fn repo_to_api_tag(repo_tag: crate::db::repository::Tag) -> Tag {
    Tag {
        id: repo_tag.id,
        name: repo_tag.name,
        slug: repo_tag.slug,
    }
}

/// Convert from blog_data::Tag to repository::Tag
fn api_to_repo_tag(api_tag: &Tag) -> crate::db::repository::Tag {
    crate::db::repository::Tag {
        id: api_tag.id,
        name: api_tag.name.clone(),
        slug: api_tag.slug.clone(),
    }
}

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

impl From<BlogError> for ApiError {
    fn from(error: BlogError) -> Self {
        match error {
            BlogError::NotFound(msg) => ApiError::NotFound(msg),
            BlogError::Validation(msg) => ApiError::ValidationError(msg),
            BlogError::Database(e) => ApiError::DatabaseError(format!("Database error: {}", e)),
            BlogError::MutexLock(msg) => {
                ApiError::DatabaseError(format!("Database lock error: {}", msg))
            }
            _ => ApiError::InternalError(format!("Internal error: {}", error)),
        }
    }
}

/// API state containing the database connection and blog repository
pub struct ApiState {
    pub blog_repo: BlogRepository,
    pub db: Database,
}

/// Gets all blog posts
#[axum::debug_handler]
#[instrument(skip(state), err)]
async fn get_all_posts(State(state): State<Arc<ApiState>>) -> ApiResult<Json<Vector<BlogPost>>> {
    match state.blog_repo.get_all_posts().await {
        Ok(repo_posts) => {
            let posts: Vector<BlogPost> = repo_posts.into_iter().map(repo_to_api_post).collect();
            debug!("Retrieved {} blog posts", posts.len());
            Ok(Json(posts))
        }
        Err(e) => {
            error!("Failed to get all posts: {}", e);
            Err(ApiError::DatabaseError(e.to_string()))
        }
    }
}

/// Gets a blog post by slug
#[axum::debug_handler]
#[instrument(skip(state), err)]
async fn get_post_by_slug(
    State(state): State<Arc<ApiState>>,
    Path(slug): Path<String>,
) -> ApiResult<Json<BlogPost>> {
    match state.blog_repo.get_post_by_slug(&slug).await {
        Ok(Some(repo_post)) => {
            let post = repo_to_api_post(repo_post);
            debug!("Retrieved blog post with slug: {}", slug);
            Ok(Json(post))
        }
        Ok(None) => {
            warn!("Blog post with slug '{}' not found", slug);
            Err(ApiError::NotFound(format!(
                "Blog post with slug '{}' not found",
                slug
            )))
        }
        Err(e) => {
            error!("Failed to get post by slug '{}': {}", slug, e);
            Err(ApiError::DatabaseError(e.to_string()))
        }
    }
}

/// Creates a new blog post
#[axum::debug_handler]
#[instrument(skip(state, post), err)]
async fn create_post(
    State(state): State<Arc<ApiState>>,
    Json(post): Json<BlogPost>,
) -> ApiResult<(StatusCode, Json<BlogPost>)> {
    info!("Creating new blog post with slug: {}", post.slug);

    // Validate required fields
    if post.title.is_empty()
        || post.slug.is_empty()
        || post.content.is_empty()
        || post.date.is_empty()
        || post.author.is_empty()
    {
        warn!("Validation failed: Missing required fields for blog post");
        return Err(ApiError::ValidationError(
            "Missing required fields: title, slug, content, date, author must be non-empty"
                .to_string(),
        ));
    }

    debug!("Validation passed for new blog post");

    // Convert API post to repository post
    let repo_post = api_to_repo_post(&post);

    // Try to save the post
    match state.blog_repo.save_post(&repo_post).await {
        Ok(post_id) => {
            info!("Successfully created blog post with ID: {}", post_id);

            // Get the created post from the database
            match state.blog_repo.get_post_by_id(post_id).await {
                Ok(Some(created_repo_post)) => {
                    let created_post = repo_to_api_post(created_repo_post);
                    debug!("Retrieved created blog post with ID: {}", post_id);
                    Ok((StatusCode::CREATED, Json(created_post)))
                }
                Ok(None) => {
                    warn!(
                        "Post with ID {} was created but not found when retrieving it",
                        post_id
                    );
                    // Return a constructed post instead
                    let constructed_post = BlogPost {
                        id: Some(post_id),
                        ..post
                    };
                    Ok((StatusCode::CREATED, Json(constructed_post)))
                }
                Err(e) => {
                    warn!("Error retrieving created post: {}", e);
                    // Return a constructed post instead
                    let constructed_post = BlogPost {
                        id: Some(post_id),
                        ..post
                    };
                    Ok((StatusCode::CREATED, Json(constructed_post)))
                }
            }
        }
        Err(e) => {
            // Special handling for SQLite locking errors which might actually indicate success
            if e.to_string().contains("locked") || e.to_string().contains("busy") {
                warn!(
                    "Database lock detected during post creation, but operation may have succeeded"
                );

                // Try to see if the post was actually created despite the error
                match state.blog_repo.get_post_by_slug(&post.slug).await {
                    Ok(Some(created_repo_post)) => {
                        info!("Post was successfully created despite transaction lock error");
                        let created_post = repo_to_api_post(created_repo_post);
                        return Ok((StatusCode::CREATED, Json(created_post)));
                    }
                    _ => {
                        error!("Failed to create post due to database lock: {}", e);
                        return Err(ApiError::DatabaseError(format!(
                            "Failed to create post: {}",
                            e
                        )));
                    }
                }
            }

            error!("Failed to create post: {}", e);
            Err(ApiError::DatabaseError(format!(
                "Failed to create post: {}",
                e
            )))
        }
    }
}

/// Updates an existing blog post
#[axum::debug_handler]
#[instrument(skip(state, post), err)]
async fn update_post(
    State(state): State<Arc<ApiState>>,
    Path(slug): Path<String>,
    Json(post): Json<BlogPost>,
) -> ApiResult<Json<BlogPost>> {
    info!("Updating blog post with slug: {}", slug);

    // Validate required fields
    if post.title.is_empty()
        || post.slug.is_empty()
        || post.content.is_empty()
        || post.date.is_empty()
        || post.author.is_empty()
    {
        warn!("Validation failed: Missing required fields for blog post update");
        return Err(ApiError::ValidationError(
            "Missing required fields: title, slug, content, date, author must be non-empty"
                .to_string(),
        ));
    }

    // First, check if post exists
    let existing_post = match state.blog_repo.get_post_by_slug(&slug).await {
        Ok(Some(repo_post)) => {
            debug!("Found existing post with slug: {}", slug);
            repo_to_api_post(repo_post)
        }
        Ok(None) => {
            warn!("Post with slug '{}' not found for update", slug);
            return Err(ApiError::NotFound(format!(
                "Post with slug '{}' not found",
                slug
            )));
        }
        Err(e) => {
            error!("Error getting post with slug {}: {}", slug, e);
            return Err(ApiError::DatabaseError(format!(
                "Failed to retrieve post: {}",
                e
            )));
        }
    };

    // Create a new post with the existing ID
    let post_to_update = BlogPost {
        id: existing_post.id,
        ..post
    };

    // Convert to repository post
    let repo_post = api_to_repo_post(&post_to_update);

    // Update the post
    match state.blog_repo.update_post(&repo_post).await {
        Ok(_) => {
            debug!("Successfully updated post in database");
        }
        Err(e) => {
            error!("Error updating post: {}", e);
            return Err(ApiError::DatabaseError(format!(
                "Failed to update post: {}",
                e
            )));
        }
    };

    // Return the updated post
    match state.blog_repo.get_post_by_slug(&post_to_update.slug).await {
        Ok(Some(updated_repo_post)) => {
            let updated_post = repo_to_api_post(updated_repo_post);
            info!("Successfully updated post with ID: {:?}", updated_post.id);
            Ok(Json(updated_post))
        }
        Ok(None) => {
            error!("Post was updated but could not be retrieved");
            Err(ApiError::InternalError(
                "Post was updated but could not be retrieved".to_string(),
            ))
        }
        Err(e) => {
            error!("Error retrieving updated post: {}", e);
            Err(ApiError::DatabaseError(format!(
                "Failed to retrieve updated post: {}",
                e
            )))
        }
    }
}

/// Deletes a blog post
#[axum::debug_handler]
#[instrument(skip(state), err)]
async fn delete_post(
    State(state): State<Arc<ApiState>>,
    Path(slug): Path<String>,
) -> ApiResult<StatusCode> {
    // First, check if post exists
    let existing_post = match state.blog_repo.get_post_by_slug(&slug).await {
        Ok(Some(post)) => post,
        Ok(None) => {
            warn!("Blog post with slug '{}' not found for deletion", slug);
            return Err(ApiError::NotFound(format!(
                "Blog post with slug '{}' not found",
                slug
            )));
        }
        Err(e) => {
            error!("Failed to get post by slug '{}' for deletion: {}", slug, e);
            return Err(ApiError::DatabaseError(e.to_string()));
        }
    };

    // Delete the post
    match state.blog_repo.delete_post(existing_post.id.unwrap()).await {
        Ok(_) => {
            info!("Deleted blog post with slug: {}", slug);
            Ok(StatusCode::NO_CONTENT)
        }
        Err(e) => {
            error!("Failed to delete post with slug '{}': {}", slug, e);
            Err(ApiError::DatabaseError(e.to_string()))
        }
    }
}

/// Gets all tags
#[axum::debug_handler]
#[instrument(skip(state), err)]
#[allow(dead_code)]
async fn get_all_tags(State(state): State<Arc<ApiState>>) -> ApiResult<Json<Vector<Tag>>> {
    match state.blog_repo.get_all_tags().await {
        Ok(repo_tags) => {
            let tags: Vector<Tag> = repo_tags.into_iter().map(repo_to_api_tag).collect();
            debug!("Retrieved {} tags", tags.len());
            Ok(Json(tags))
        }
        Err(e) => {
            error!("Failed to get all tags: {}", e);
            Err(ApiError::DatabaseError(e.to_string()))
        }
    }
}

/// Gets all published blog posts
#[axum::debug_handler]
#[instrument(skip(state), err)]
async fn get_published_posts(
    State(state): State<Arc<ApiState>>,
) -> ApiResult<Json<Vector<BlogPost>>> {
    match state.blog_repo.get_published_posts().await {
        Ok(repo_posts) => {
            let posts: Vector<BlogPost> = repo_posts.into_iter().map(repo_to_api_post).collect();
            debug!("Retrieved {} published blog posts", posts.len());
            Ok(Json(posts))
        }
        Err(e) => {
            error!("Failed to get published posts: {}", e);
            Err(ApiError::DatabaseError(e.to_string()))
        }
    }
}

/// Gets all featured blog posts
#[axum::debug_handler]
#[instrument(skip(state), err)]
async fn get_featured_posts(
    State(state): State<Arc<ApiState>>,
) -> ApiResult<Json<Vector<BlogPost>>> {
    match state.blog_repo.get_published_posts().await {
        Ok(repo_posts) => {
            // Filter for featured posts
            let featured_posts: Vector<BlogPost> = repo_posts
                .into_iter()
                .map(repo_to_api_post)
                .filter(|post| post.featured)
                .collect();
            debug!("Retrieved {} featured blog posts", featured_posts.len());
            Ok(Json(featured_posts))
        }
        Err(e) => {
            error!("Failed to get featured posts: {}", e);
            Err(ApiError::DatabaseError(e.to_string()))
        }
    }
}

/// Gets posts by tag
#[axum::debug_handler]
#[instrument(skip(state), err)]
async fn get_posts_by_tag(
    State(state): State<Arc<ApiState>>,
    Path(tag_slug): Path<String>,
) -> ApiResult<Json<Vector<BlogPost>>> {
    match state.blog_repo.get_all_posts().await {
        Ok(repo_posts) => {
            // Filter posts by tag
            let posts_with_tag: Vector<BlogPost> = repo_posts
                .into_iter()
                .map(repo_to_api_post)
                .filter(|post| post.tags.iter().any(|tag| tag.slug == tag_slug))
                .collect();

            debug!(
                "Retrieved {} posts with tag '{}'",
                posts_with_tag.len(),
                tag_slug
            );
            Ok(Json(posts_with_tag))
        }
        Err(e) => {
            error!("Failed to get posts by tag '{}': {}", tag_slug, e);
            Err(ApiError::DatabaseError(e.to_string()))
        }
    }
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
pub fn create_blog_api_router(db_path: PathBuf) -> std::result::Result<Router, BlogError> {
    let db = Database::new(&db_path)?;
    let blog_repo = db.blog_repository();
    let state = Arc::new(ApiState { blog_repo, db });

    // Create a static file service for the static directory (blog tools)
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

    // Health check endpoint
    async fn health_handler() -> impl axum::response::IntoResponse {
        let health_data = serde_json::json!({
            "status": "healthy",
            "timestamp": chrono::Local::now().to_rfc3339()
        });
        Json(health_data)
    }

    // Handler to serve the main index.html from dist
    async fn main_index_handler() -> impl axum::response::IntoResponse {
        match tokio::fs::read_to_string("dist/index.html").await {
            Ok(content) => axum::response::Html(content),
            Err(_) => axum::response::Html(
                r#"<html><body><h1>Main website not found</h1><p>Please run <code>cargo run --bin cv</code> to generate the website files.</p></body></html>"#.to_string()
            )
        }
    }

    // Handler to serve blog.html from dist
    async fn blog_page_handler() -> impl axum::response::IntoResponse {
        match tokio::fs::read_to_string("dist/blog.html").await {
            Ok(content) => axum::response::Html(content),
            Err(_) => axum::response::Html(
                r#"<html><body><h1>Blog page not found</h1><p>Please run <code>cargo run --bin cv</code> to generate the website files.</p></body></html>"#.to_string()
            )
        }
    }

    // Handler to serve cv.html from dist
    async fn cv_page_handler() -> impl axum::response::IntoResponse {
        match tokio::fs::read_to_string("dist/cv.html").await {
            Ok(content) => axum::response::Html(content),
            Err(_) => axum::response::Html(
                r#"<html><body><h1>CV page not found</h1><p>Please run <code>cargo run --bin cv</code> to generate the website files.</p></body></html>"#.to_string()
            )
        }
    }

    // Handler to serve projects.html from dist
    async fn projects_page_handler() -> impl axum::response::IntoResponse {
        match tokio::fs::read_to_string("dist/projects.html").await {
            Ok(content) => axum::response::Html(content),
            Err(_) => axum::response::Html(
                r#"<html><body><h1>Projects page not found</h1><p>Please run <code>cargo run --bin cv</code> to generate the website files.</p></body></html>"#.to_string()
            )
        }
    }

    // Route order matters! More specific routes need to come before less specific ones
    // to avoid route conflicts, especially with path parameters
    let router = Router::new()
        // Health check endpoint
        .route("/health", get(health_handler))
        // Main website routes - serve the proper frontend from dist/
        .route("/", get(main_index_handler))
        .route("/index.html", get(main_index_handler))
        // API diagnostic endpoint
        .route("/api/blog/test", get(diagnostic_handler))
        // Admin/API documentation route
        .route("/admin", get(root_handler))
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
        // Serve static files (blog tools)
        .nest_service("/static", static_service)
        // Serve main website pages
        .route("/blog.html", get(blog_page_handler))
        .route("/cv.html", get(cv_page_handler))
        .route("/projects.html", get(projects_page_handler))
        // Serve main website files from dist (CSS, JS, images, other assets)
        .nest_service("/css", ServeDir::new("dist/css"))
        .nest_service("/js", ServeDir::new("dist/js"))
        .nest_service("/img", ServeDir::new("dist/img"))
        .nest_service("/fonts", ServeDir::new("dist/fonts"))
        // Serve other static assets from dist
        .nest_service("/manifest.json", ServeDir::new("dist"))
        .nest_service("/robots.txt", ServeDir::new("dist"))
        .nest_service("/_headers", ServeDir::new("dist"))
        .nest_service("/_redirects", ServeDir::new("dist"))
        // Apply CORS middleware
        .layer(cors)
        // Add request timeout middleware - increased to 120 seconds for database operations
        .layer(TimeoutLayer::new(std::time::Duration::from_secs(120)))
        .with_state(state);

    Ok(router)
}
