use anyhow::{Context, Result};
use cv::blog_data::{BlogPost, Tag};
use cv::db::{BlogRepository, Database, error::DatabaseError};
use cv::logging;
use im::{HashMap, Vector};
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, error, info, instrument, warn};

// Function to create a test blog post
fn create_test_post() -> BlogPost {
    let tag = Tag {
        id: None,
        name: "Test".to_string(),
        slug: "test".to_string(),
    };

    // Create a unique slug to avoid conflicts
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let unique_slug = format!("test-post-{timestamp}");

    // Create a simple metadata map
    let mut metadata = HashMap::new();
    metadata.insert("test_key".to_string(), "test_value".to_string());

    BlogPost {
        id: None,
        title: format!("Test Post {timestamp}"),
        slug: unique_slug,
        date: chrono::Local::now().format("%Y-%m-%d").to_string(),
        user_id: None,
        author: "Test Author".to_string(),
        excerpt: "This is a test excerpt".to_string(),
        content: "This is the full content of the test post.".to_string(),
        content_format: cv::blog_data::ContentFormat::HTML,
        published: true,
        featured: false,
        image: None,
        tags: Vector::from(vec![tag]),
        metadata,
    }
}

// Convert from blog_data::BlogPost to repository::BlogPost
fn api_to_repo_post(api_post: &BlogPost) -> cv::db::repository::BlogPost {
    cv::db::repository::BlogPost {
        id: api_post.id,
        title: api_post.title.clone(),
        slug: api_post.slug.clone(),
        date: api_post.date.clone(),
        user_id: api_post.user_id,
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

// Convert from blog_data::Tag to repository::Tag
fn api_to_repo_tag(api_tag: &Tag) -> cv::db::repository::Tag {
    cv::db::repository::Tag {
        id: api_tag.id,
        name: api_tag.name.clone(),
        slug: api_tag.slug.clone(),
    }
}

// Convert from repository::BlogPost to blog_data::BlogPost
fn repo_to_api_post(repo_post: cv::db::repository::BlogPost) -> BlogPost {
    // Determine content format from metadata or default to HTML
    let content_format = if let Some(format) = repo_post.metadata.get("content_format") {
        if format == "markdown" {
            cv::blog_data::ContentFormat::Markdown
        } else {
            cv::blog_data::ContentFormat::HTML
        }
    } else {
        cv::blog_data::ContentFormat::HTML
    };

    BlogPost {
        id: repo_post.id,
        title: repo_post.title,
        slug: repo_post.slug,
        date: repo_post.date,
        user_id: repo_post.user_id,
        author: repo_post.author,
        excerpt: repo_post.excerpt,
        content: repo_post.content,
        content_format,
        published: repo_post.published,
        featured: repo_post.featured,
        image: repo_post.image,
        tags: repo_post.tags.into_iter().map(repo_to_api_tag).collect(),
        metadata: repo_post.metadata.into_iter().collect(),
    }
}

// Convert from repository::Tag to blog_data::Tag
fn repo_to_api_tag(repo_tag: cv::db::repository::Tag) -> Tag {
    Tag {
        id: repo_tag.id,
        name: repo_tag.name,
        slug: repo_tag.slug,
    }
}

// Retry function with exponential backoff
async fn retry_with_backoff<F, Fut, T>(
    operation: F,
    max_retries: u32,
    initial_delay_ms: u64,
    operation_name: &str,
) -> Result<T>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let mut retry_count = 0;
    let mut delay_ms = initial_delay_ms;

    loop {
        match operation().await {
            Ok(result) => {
                if retry_count > 0 {
                    info!(
                        "Operation '{}' succeeded after {} retries",
                        operation_name, retry_count
                    );
                } else {
                    debug!("Operation '{}' succeeded on first attempt", operation_name);
                }
                return Ok(result);
            }
            Err(e) => {
                retry_count += 1;

                // Check if we've reached the maximum number of retries
                if retry_count >= max_retries {
                    error!(
                        "Operation '{}' failed after {} retries: {}",
                        operation_name, max_retries, e
                    );
                    return Err(e);
                }

                // Check if the error is a database locking error
                let is_db_lock = if let Some(db_err) = e.downcast_ref::<DatabaseError>() {
                    matches!(db_err, DatabaseError::Locking(_))
                } else {
                    e.to_string().contains("locked") || e.to_string().contains("busy")
                };

                // Calculate backoff with jitter
                let jitter = rand::random::<u64>() % 500;
                delay_ms = if is_db_lock {
                    // Use longer delays for database locks
                    std::cmp::min(delay_ms * 2, 10000) + jitter
                } else {
                    // Standard exponential backoff with jitter
                    std::cmp::min(delay_ms * 2, 5000) + jitter
                };

                warn!(
                    "Operation '{}' failed (attempt {}/{}), retrying in {}ms: {}",
                    operation_name, retry_count, max_retries, delay_ms, e
                );

                sleep(Duration::from_millis(delay_ms)).await;
            }
        }
    }
}

// Test creating a blog post
#[instrument(skip(repo), err)]
async fn test_create_post(repo: &BlogRepository) -> Result<BlogPost> {
    info!("Testing blog post creation");

    let test_post = create_test_post();
    debug!("Created test post with slug: {}", test_post.slug);

    let repo_post = api_to_repo_post(&test_post);

    let post_id = retry_with_backoff(
        || async { repo.save_post(&repo_post).await },
        5,
        100,
        "create_post",
    )
    .await?;

    info!("Successfully created post with ID: {}", post_id);

    // Retrieve the created post
    let created_post = retry_with_backoff(
        || async {
            match repo.get_post_by_id(post_id).await? {
                Some(post) => Ok(post),
                None => Err(anyhow::anyhow!("Post with ID {} not found", post_id)),
            }
        },
        3,
        100,
        "get_created_post",
    )
    .await?;

    let api_post = repo_to_api_post(created_post);
    info!("Retrieved created post: {}", api_post.title);

    Ok(api_post)
}

// Test retrieving all blog posts
#[instrument(skip(repo), err)]
async fn test_get_all_posts(repo: &BlogRepository) -> Result<Vector<BlogPost>> {
    info!("Testing retrieval of all blog posts");

    let repo_posts = retry_with_backoff(
        || async { repo.get_all_posts().await },
        3,
        100,
        "get_all_posts",
    )
    .await?;

    let posts: Vector<BlogPost> = repo_posts.into_iter().map(repo_to_api_post).collect();
    info!("Retrieved {} blog posts", posts.len());

    Ok(posts)
}

// Test retrieving a blog post by slug
#[instrument(skip(repo), err)]
async fn test_get_post_by_slug(repo: &BlogRepository, slug: &str) -> Result<BlogPost> {
    info!("Testing retrieval of blog post by slug: {}", slug);

    let repo_post = retry_with_backoff(
        || async {
            match repo.get_post_by_slug(slug).await? {
                Some(post) => Ok(post),
                None => Err(anyhow::anyhow!("Post with slug '{}' not found", slug)),
            }
        },
        3,
        100,
        "get_post_by_slug",
    )
    .await?;

    let post = repo_to_api_post(repo_post);
    info!("Retrieved blog post: {}", post.title);

    Ok(post)
}

// Test updating a blog post
#[instrument(skip(repo), err)]
async fn test_update_post(repo: &BlogRepository, post: &BlogPost) -> Result<BlogPost> {
    info!("Testing blog post update for post with slug: {}", post.slug);

    // Create an updated version of the post
    let mut updated_post = post.clone();
    updated_post.title = format!("{} (Updated)", post.title);
    updated_post.content = format!("{}\n\nThis content has been updated.", post.content);

    let repo_post = api_to_repo_post(&updated_post);

    // Update the post
    retry_with_backoff(
        || async { repo.update_post(&repo_post).await },
        5,
        100,
        "update_post",
    )
    .await?;

    info!("Successfully updated post");

    // Retrieve the updated post
    let updated_repo_post = retry_with_backoff(
        || async {
            match repo.get_post_by_slug(&updated_post.slug).await? {
                Some(post) => Ok(post),
                None => Err(anyhow::anyhow!(
                    "Updated post with slug '{}' not found",
                    updated_post.slug
                )),
            }
        },
        3,
        100,
        "get_updated_post",
    )
    .await?;

    let api_updated_post = repo_to_api_post(updated_repo_post);
    info!("Retrieved updated post: {}", api_updated_post.title);

    Ok(api_updated_post)
}

// Test deleting a blog post
#[instrument(skip(repo), err)]
async fn test_delete_post(repo: &BlogRepository, post: &BlogPost) -> Result<()> {
    let post_id = post.id.ok_or_else(|| anyhow::anyhow!("Post has no ID"))?;
    info!("Testing blog post deletion for post with ID: {}", post_id);

    retry_with_backoff(
        || async { repo.delete_post(post_id).await },
        5,
        100,
        "delete_post",
    )
    .await?;

    info!("Successfully deleted post");

    // Verify the post is deleted
    match repo.get_post_by_id(post_id).await? {
        Some(_) => {
            warn!("Post with ID {} still exists after deletion", post_id);
            Err(anyhow::anyhow!("Post still exists after deletion"))
        }
        None => {
            info!("Verified post with ID {} no longer exists", post_id);
            Ok(())
        }
    }
}

// Initialize logging with tracing
fn init_logging() {
    // Set up a logging configuration for the blog tester
    let config = logging::LoggingConfig {
        app_name: "blog_tester".to_string(),
        level: tracing::Level::DEBUG,
        log_spans: true,
        ..Default::default()
    };

    // Initialize logging with the configuration
    let _guard = logging::init_logging(config);
    info!("Logging initialized with tracing");
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    init_logging();

    info!("Blog API Tester");
    info!("==============");

    // Get the database path
    let db_path = PathBuf::from("blog.db");
    info!("Using database at: {:?}", db_path);

    // Create the database connection
    let db = Database::new(&db_path).context("Failed to create database connection")?;
    let blog_repo = db.blog_repository();

    info!("Successfully connected to database");

    // Run the tests
    let created_post = match test_create_post(&blog_repo).await {
        Ok(post) => {
            info!("✅ Post creation test passed");
            post
        }
        Err(e) => {
            error!("❌ Post creation test failed: {}", e);
            return Err(e);
        }
    };

    match test_get_all_posts(&blog_repo).await {
        Ok(posts) => {
            info!("✅ Get all posts test passed, found {} posts", posts.len());
        }
        Err(e) => {
            error!("❌ Get all posts test failed: {}", e);
            return Err(e);
        }
    }

    match test_get_post_by_slug(&blog_repo, &created_post.slug).await {
        Ok(_) => {
            info!("✅ Get post by slug test passed");
        }
        Err(e) => {
            error!("❌ Get post by slug test failed: {}", e);
            return Err(e);
        }
    }

    let updated_post = match test_update_post(&blog_repo, &created_post).await {
        Ok(post) => {
            info!("✅ Post update test passed");
            post
        }
        Err(e) => {
            error!("❌ Post update test failed: {}", e);
            return Err(e);
        }
    };

    match test_delete_post(&blog_repo, &updated_post).await {
        Ok(_) => {
            info!("✅ Post deletion test passed");
        }
        Err(e) => {
            error!("❌ Post deletion test failed: {}", e);
            return Err(e);
        }
    }

    info!("🎉 All tests passed successfully!");

    Ok(())
}
