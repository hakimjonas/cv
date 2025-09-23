use crate::blog_data::{BlogPost, Tag};
use crate::blog_error::BlogError;
use crate::blog_validation::{BlogValidationError, validate_and_sanitize_blog_post};
use crate::check_db_permissions::secure_db_permissions;
use crate::db::{BlogRepository, Database};
use crate::feature_flags::FeatureFlags;
use crate::feature_flags::rollback::RollbackManager;
use crate::feed::{FeedConfig, generate_atom_feed, generate_rss_feed};
use crate::git_identity::GitIdentityService;
use crate::image_api::create_image_api_router;
use crate::rate_limiter::RateLimiterConfig;
use crate::simple_auth::{AuthResponse, SimpleAuthService};
use crate::simple_auth_middleware::AuthenticatedUser;
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
};
use chrono::Datelike;
use im::{HashMap, Vector};
use std::path::PathBuf;
use std::sync::Arc;
use tower_http::services::ServeDir;

/// Convert repository blog post to API blog post
fn repo_to_api_post(repo_post: crate::db::repository::BlogPost) -> BlogPost {
    BlogPost {
        id: repo_post.id,
        title: repo_post.title,
        slug: repo_post.slug,
        content: repo_post.content,
        excerpt: repo_post.excerpt,
        author: repo_post.author,
        published: repo_post.published,
        featured: repo_post.featured,
        date: repo_post.date,
        tags: repo_post.tags.into_iter().map(repo_to_api_tag).collect(),
        user_id: repo_post.user_id,
        image: repo_post.image,
        content_format: crate::blog_data::ContentFormat::HTML, // Default to HTML
        metadata: repo_post.metadata,
    }
}

/// Convert API blog post to repository blog post
fn api_to_repo_post(api_post: &BlogPost) -> crate::db::repository::BlogPost {
    crate::db::repository::BlogPost {
        id: api_post.id,
        title: api_post.title.clone(),
        slug: api_post.slug.clone(),
        content: api_post.content.clone(),
        excerpt: api_post.excerpt.clone(),
        author: api_post.author.clone(),
        published: api_post.published,
        featured: api_post.featured,
        date: api_post.date.clone(),
        tags: api_post.tags.iter().map(api_to_repo_tag).collect(),
        user_id: api_post.user_id,
        image: api_post.image.clone(),
        metadata: api_post.metadata.clone(),
    }
}

/// Convert repository tag to API tag
fn repo_to_api_tag(repo_tag: crate::db::repository::Tag) -> Tag {
    Tag {
        id: repo_tag.id,
        name: repo_tag.name,
        slug: repo_tag.slug,
    }
}

/// Convert API tag to repository tag
fn api_to_repo_tag(api_tag: &Tag) -> crate::db::repository::Tag {
    crate::db::repository::Tag {
        id: api_tag.id,
        name: api_tag.name.clone(),
        slug: api_tag.slug.clone(),
    }
}

/// API error types
#[derive(Debug)]
enum ApiError {
    /// Validation error
    ValidationError(String),
    /// Database error
    DatabaseError(String),
    /// Not found error
    NotFound(String),
    /// Internal server error
    InternalError(String),
}

/// Convert API error to HTTP response
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ApiError::ValidationError(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::DatabaseError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            ApiError::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        (status, message).into_response()
    }
}

/// Convert blog error to API error
impl From<BlogError> for ApiError {
    fn from(error: BlogError) -> Self {
        match error {
            BlogError::Validation(msg) => ApiError::ValidationError(msg),
            BlogError::Database(msg) => ApiError::DatabaseError(msg.to_string()),
            BlogError::NotFound(msg) => ApiError::NotFound(msg),
            BlogError::Internal(msg) => ApiError::InternalError(msg),
            BlogError::PermissionError(msg) => ApiError::ValidationError(msg),
            BlogError::ConfigError(msg) => ApiError::InternalError(msg),
            BlogError::FileSystemError(msg) => ApiError::InternalError(msg),
            BlogError::Io(err) => ApiError::InternalError(format!("IO error: {err}")),
            BlogError::Serialization(msg) => {
                ApiError::InternalError(format!("Serialization error: {msg}"))
            }
            BlogError::Deserialization(msg) => {
                ApiError::InternalError(format!("Deserialization error: {msg}"))
            }
            BlogError::MutexLock(msg) => {
                ApiError::InternalError(format!("Mutex lock error: {msg}"))
            }
        }
    }
}

/// API result type
type ApiResult<T> = Result<T, ApiError>;

/// API state containing the blog repository, auth service, and feature flags
pub struct ApiState {
    /// Blog repository
    pub repo: BlogRepository,
    /// Simple authentication service
    pub auth_service: Arc<SimpleAuthService>,
    /// Feature flags
    pub feature_flags: FeatureFlags,
    /// Rollback manager
    pub rollback_manager: Arc<RollbackManager>,
    /// Git identity service
    pub git_identity_service: GitIdentityService,
}

/// Get all blog posts
async fn get_all_posts(State(state): State<Arc<ApiState>>) -> ApiResult<Json<Vector<BlogPost>>> {
    let posts = state
        .repo
        .get_all_posts()
        .await
        .map_err(|e| ApiError::DatabaseError(format!("Failed to get posts: {e}")))?;

    let api_posts = posts.into_iter().map(repo_to_api_post).collect();
    Ok(Json(api_posts))
}

/// Get a blog post by slug
async fn get_post_by_slug(
    State(state): State<Arc<ApiState>>,
    Path(slug): Path<String>,
) -> ApiResult<Json<BlogPost>> {
    let post = state
        .repo
        .get_post_by_slug(&slug)
        .await
        .map_err(|e| ApiError::DatabaseError(format!("Failed to get post: {e}")))?
        .ok_or_else(|| ApiError::NotFound(format!("Post not found: {slug}")))?;

    Ok(Json(repo_to_api_post(post)))
}

/// Create a new blog post
async fn create_post(
    State(state): State<Arc<ApiState>>,
    auth_user: AuthenticatedUser,
    Json(mut post): Json<BlogPost>,
) -> ApiResult<(StatusCode, Json<BlogPost>)> {
    // Set the user ID and author from the authenticated user
    post.user_id = Some(auth_user.0.user_id);
    post.author = auth_user
        .0
        .display_name
        .clone()
        .unwrap_or_else(|| "Unknown".to_string());

    // Validate and sanitize the post
    let validated_post = validate_and_sanitize_blog_post(&post).map_err(|e| match e {
        BlogValidationError::ValidationError(msg) => ApiError::ValidationError(msg),
        BlogValidationError::SanitizationError(msg) => ApiError::ValidationError(msg),
    })?;

    // Convert to repository post and save
    let repo_post = api_to_repo_post(&validated_post);
    let post_id = state
        .repo
        .save_post(&repo_post)
        .await
        .map_err(|e| ApiError::DatabaseError(format!("Failed to save post: {e}")))?;

    // Fetch the saved post by ID
    let saved_post = state
        .repo
        .get_post_by_id(post_id)
        .await
        .map_err(|e| ApiError::DatabaseError(format!("Failed to get saved post: {e}")))?
        .ok_or_else(|| ApiError::InternalError("Saved post not found".to_string()))?;

    // Convert back to API post and return
    Ok((StatusCode::CREATED, Json(repo_to_api_post(saved_post))))
}

/// Update a blog post
async fn update_post(
    State(state): State<Arc<ApiState>>,
    Path(slug): Path<String>,
    auth_user: AuthenticatedUser,
    Json(mut post): Json<BlogPost>,
) -> ApiResult<Json<BlogPost>> {
    // Get the existing post
    let existing_post = state
        .repo
        .get_post_by_slug(&slug)
        .await
        .map_err(|e| ApiError::DatabaseError(format!("Failed to get post: {e}")))?
        .ok_or_else(|| ApiError::NotFound(format!("Post not found: {slug}")))?;

    // Check if the user is the author of the post
    if existing_post.user_id != Some(auth_user.0.user_id) && auth_user.0.role != "Admin" {
        return Err(ApiError::ValidationError(format!(
            "You don't have permission to update post: {slug}"
        )));
    }

    // Keep the original user ID and update the author if needed
    post.user_id = existing_post.user_id;
    if post.author.is_empty() {
        post.author = auth_user
            .0
            .display_name
            .clone()
            .unwrap_or_else(|| "Unknown".to_string());
    }

    // Validate and sanitize the post
    let validated_post = validate_and_sanitize_blog_post(&post).map_err(|e| match e {
        BlogValidationError::ValidationError(msg) => ApiError::ValidationError(msg),
        BlogValidationError::SanitizationError(msg) => ApiError::ValidationError(msg),
    })?;

    // Convert to repository post and update
    let repo_post = api_to_repo_post(&validated_post);

    // Update the post (returns () on success)
    state
        .repo
        .update_post(&repo_post)
        .await
        .map_err(|e| ApiError::DatabaseError(format!("Failed to update post: {e}")))?;

    // Fetch the updated post by slug
    let updated_post = state
        .repo
        .get_post_by_slug(&repo_post.slug)
        .await
        .map_err(|e| ApiError::DatabaseError(format!("Failed to get updated post: {e}")))?
        .ok_or_else(|| ApiError::InternalError("Updated post not found".to_string()))?;

    // Convert back to API post and return
    Ok(Json(repo_to_api_post(updated_post)))
}

/// Delete a blog post
async fn delete_post(
    State(state): State<Arc<ApiState>>,
    Path(slug): Path<String>,
    auth_user: AuthenticatedUser,
) -> ApiResult<StatusCode> {
    // Get the existing post
    let existing_post = state
        .repo
        .get_post_by_slug(&slug)
        .await
        .map_err(|e| ApiError::DatabaseError(format!("Failed to get post: {e}")))?
        .ok_or_else(|| ApiError::NotFound(format!("Post not found: {slug}")))?;

    // Check if the user is the author of the post
    if existing_post.user_id != Some(auth_user.0.user_id) && auth_user.0.role != "Admin" {
        return Err(ApiError::ValidationError(format!(
            "You don't have permission to delete post: {slug}"
        )));
    }

    // Delete the post
    state
        .repo
        .delete_post(existing_post.id.expect("Post ID should exist for deletion"))
        .await
        .map_err(|e| ApiError::DatabaseError(format!("Failed to delete post: {e}")))?;

    Ok(StatusCode::NO_CONTENT)
}

/// Get all tags
async fn get_all_tags(State(state): State<Arc<ApiState>>) -> ApiResult<Json<Vector<Tag>>> {
    let tags = state
        .repo
        .get_all_tags()
        .await
        .map_err(|e| ApiError::DatabaseError(format!("Failed to get tags: {e}")))?;

    let api_tags = tags.into_iter().map(repo_to_api_tag).collect();
    Ok(Json(api_tags))
}

/// Get published blog posts
async fn get_published_posts(
    State(state): State<Arc<ApiState>>,
) -> ApiResult<Json<Vector<BlogPost>>> {
    let posts = state
        .repo
        .get_published_posts()
        .await
        .map_err(|e| ApiError::DatabaseError(format!("Failed to get published posts: {e}")))?;

    let api_posts = posts.into_iter().map(repo_to_api_post).collect();
    Ok(Json(api_posts))
}

/// Get featured blog posts
async fn get_featured_posts(
    State(state): State<Arc<ApiState>>,
) -> ApiResult<Json<Vector<BlogPost>>> {
    let posts = state
        .repo
        .get_featured_posts()
        .await
        .map_err(|e| ApiError::DatabaseError(format!("Failed to get featured posts: {e}")))?;

    let api_posts = posts.into_iter().map(repo_to_api_post).collect();
    Ok(Json(api_posts))
}

/// Search blog posts
async fn search_posts(
    State(state): State<Arc<ApiState>>,
    Query(params): Query<HashMap<String, String>>,
) -> ApiResult<Json<Vector<BlogPost>>> {
    let query = params.get("q").cloned().unwrap_or_default();
    let _tag = params.get("tag").cloned();
    let published_only = params.get("published").map(|v| v == "true").unwrap_or(true);

    let posts = state
        .repo
        .search_posts(&query, published_only)
        .await
        .map_err(|e| ApiError::DatabaseError(format!("Failed to search posts: {e}")))?;

    let api_posts = posts.into_iter().map(repo_to_api_post).collect();
    Ok(Json(api_posts))
}

/// Get RSS feed
async fn get_rss_feed(State(state): State<Arc<ApiState>>) -> Result<impl IntoResponse, ApiError> {
    // Get published posts
    let posts = state
        .repo
        .get_published_posts()
        .await
        .map_err(|e| ApiError::DatabaseError(format!("Failed to get published posts: {e}")))?;

    // Get the current year for the copyright
    let current_year = chrono::Utc::now().year();

    // Create feed configuration
    let config = FeedConfig {
        title: "Blog".to_string(),
        description: "Latest blog posts".to_string(),
        link: "https://example.com/blog".to_string(),
        author: "Blog Author".to_string(),
        email: "author@example.com".to_string(),
        language: "en-us".to_string(),
        copyright: format!("Copyright {current_year}"),
        base_url: "https://example.com".to_string(),
    };

    // Convert repository posts to blog data posts
    let api_posts: Vector<BlogPost> = posts.into_iter().map(repo_to_api_post).collect();

    // Generate RSS feed
    let rss_feed = generate_rss_feed(&api_posts, &config)
        .map_err(|e| ApiError::InternalError(format!("Failed to generate RSS feed: {e}")))?;

    // Return the RSS feed with appropriate content type
    Ok((
        StatusCode::OK,
        [("Content-Type", "application/rss+xml")],
        rss_feed,
    ))
}

/// Get Atom feed
async fn get_atom_feed(State(state): State<Arc<ApiState>>) -> Result<impl IntoResponse, ApiError> {
    // Get published posts
    let posts = state
        .repo
        .get_published_posts()
        .await
        .map_err(|e| ApiError::DatabaseError(format!("Failed to get published posts: {e}")))?;

    // Get the current year for the copyright
    let current_year = chrono::Utc::now().year();

    // Create feed configuration
    let config = FeedConfig {
        title: "Blog".to_string(),
        description: "Latest blog posts".to_string(),
        link: "https://example.com/blog".to_string(),
        author: "Blog Author".to_string(),
        email: "author@example.com".to_string(),
        language: "en-us".to_string(),
        copyright: format!("Copyright {current_year}"),
        base_url: "https://example.com".to_string(),
    };

    // Convert repository posts to blog data posts
    let api_posts: Vector<BlogPost> = posts.into_iter().map(repo_to_api_post).collect();

    // Generate Atom feed
    let atom_feed = generate_atom_feed(&api_posts, &config)
        .map_err(|e| ApiError::InternalError(format!("Failed to generate Atom feed: {e}")))?;

    // Return the Atom feed with appropriate content type
    Ok((
        StatusCode::OK,
        [("Content-Type", "application/atom+xml")],
        atom_feed,
    ))
}

/// Get posts by tag
async fn get_posts_by_tag(
    State(state): State<Arc<ApiState>>,
    Path(tag_slug): Path<String>,
) -> ApiResult<Json<Vector<BlogPost>>> {
    let posts = state
        .repo
        .get_posts_by_tag(&tag_slug)
        .await
        .map_err(|e| ApiError::DatabaseError(format!("Failed to get posts by tag: {e}")))?;

    let api_posts = posts.into_iter().map(repo_to_api_post).collect();
    Ok(Json(api_posts))
}

/// Root handler
async fn root_handler() -> impl IntoResponse {
    // Redirect to the blog page
    (
        StatusCode::TEMPORARY_REDIRECT,
        [("Location", "/blog")],
        "Redirecting to blog",
    )
}

/// Create a session based on Git identity
async fn create_session(
    State(state): State<Arc<ApiState>>,
) -> Result<Json<AuthResponse>, ApiError> {
    state
        .auth_service
        .create_session()
        .await
        .map(Json)
        .map_err(|e| ApiError::InternalError(format!("Failed to create session: {e}")))
}

/// Create the blog API router
pub async fn create_simple_blog_api_router(
    db_path: PathBuf,
    jwt_secret: String,
    token_expiration: i64,
    dev_mode: bool,
) -> Result<Router, BlogError> {
    // Ensure database directory has correct permissions
    secure_db_permissions(&db_path)?;

    // Create database connection
    let db = Database::new(&db_path)?;

    // Create blog repository
    let repo = db.blog_repository();

    // Create Git identity service
    let git_identity_service = GitIdentityService::new();

    // Create configuration for SimpleAuthService
    let mut config = crate::unified_config::AppConfig {
        dev_mode,
        ..Default::default()
    };

    // Try to get Git identity for owner configuration
    if let Ok(identity) = git_identity_service.get_identity() {
        let owner = crate::unified_config::OwnerConfig {
            name: identity.name.clone(),
            github_username: identity.github_username.clone(),
            email: identity.email.clone(),
            display_name: None,
            bio: None,
            role: "Author".to_string(),
        };
        config.owner = Some(owner);
    }

    // Create simple auth service
    let auth_service = Arc::new(SimpleAuthService::new(
        &config,
        jwt_secret,
        token_expiration,
    ));

    // Create feature flags
    let feature_flags = FeatureFlags::new(crate::feature_flags::FeatureFlagConfig::default(), None);

    // Create rollback manager
    let rollback_manager = Arc::new(RollbackManager::new());

    // Create API state
    let state = Arc::new(ApiState {
        repo,
        auth_service: auth_service.clone(),
        feature_flags,
        rollback_manager,
        git_identity_service,
    });

    // Create rate limiter configuration
    let _rate_limiter_config = RateLimiterConfig {
        max_requests: 60,
        window_seconds: 60,
        include_headers: true,
        status_code: StatusCode::TOO_MANY_REQUESTS,
    };

    // Create image API router
    let image_api = create_image_api_router("/images");

    // Create API router
    let api_router = Router::new()
        // Public endpoints
        .route("/posts", get(get_all_posts))
        .route("/posts/{slug}", get(get_post_by_slug))
        .route("/tags", get(get_all_tags))
        .route("/published", get(get_published_posts))
        .route("/featured", get(get_featured_posts))
        .route("/search", get(search_posts))
        .route("/feed/rss", get(get_rss_feed))
        .route("/feed/atom", get(get_atom_feed))
        .route("/tags/{tag}", get(get_posts_by_tag))
        .route("/session", post(create_session))
        // Protected endpoints
        .route("/posts", post(create_post))
        .route("/posts/{slug}", put(update_post))
        .route("/posts/{slug}", delete(delete_post))
        // Note: Middleware is temporarily disabled due to compatibility issues
        // Will be properly implemented in a future update
        .with_state(state.clone());
    // Rate limiter temporarily disabled due to type compatibility issues
    // Will be properly implemented in a future update
    // .layer(create_rate_limiter_layer(rate_limiter_config));

    // Create static file server
    let static_dir = PathBuf::from("static");
    let static_service = ServeDir::new(static_dir);

    // Create main router
    let app = Router::new()
        .route("/", get(root_handler))
        .route("/health", get(health_handler))
        .route("/blog", get(blog_page_handler))
        .route("/cv", get(cv_page_handler))
        .route("/projects", get(projects_page_handler))
        .nest("/api", api_router)
        .nest("/images", image_api)
        .fallback_service(static_service);

    Ok(app)
}

/// Health check handler
async fn health_handler() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

/// Blog page handler
async fn blog_page_handler() -> impl IntoResponse {
    (
        StatusCode::OK,
        [("Content-Type", "text/html")],
        include_str!("../static/simple-blog-client.html"),
    )
}

/// CV page handler
async fn cv_page_handler() -> impl IntoResponse {
    (
        StatusCode::OK,
        [("Content-Type", "text/html")],
        include_str!("../static/cv.html"),
    )
}

/// Projects page handler
async fn projects_page_handler() -> impl IntoResponse {
    (
        StatusCode::OK,
        [("Content-Type", "text/html")],
        include_str!("../static/projects.html"),
    )
}
