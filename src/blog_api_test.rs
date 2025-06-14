use crate::blog_api::*;
use crate::blog_data::*;
use anyhow::Result;
use axum::{body::Body, http::{Request, StatusCode}, response::Response};
use im::Vector;
use std::{path::PathBuf, sync::Arc};
use tempfile::TempDir;
use tower::ServiceExt;

#[cfg(test)]
mod tests {
    use super::*;

    async fn create_test_app() -> Result<(axum::Router, TempDir)> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test.db");
        let blog_manager = Arc::new(BlogManager::new(&db_path)?);
        let state = Arc::new(ApiState { blog_manager });

        let app = axum::Router::new()
            .route("/api/blog", axum::routing::get(get_all_posts))
            .route("/api/blog", axum::routing::post(create_post))
            .route("/api/blog/:slug", axum::routing::get(get_post_by_slug))
            .route("/api/blog/:slug", axum::routing::put(update_post))
            .route("/api/blog/:slug", axum::routing::delete(delete_post))
            .with_state(state);

        Ok((app, temp_dir))
    }

    fn create_test_post() -> BlogPost {
        BlogPost {
            id: None,
            title: "Test API Post".to_string(),
            content: "Test API content".to_string(),
            excerpt: "Test API excerpt".to_string(),
            date: "2023-01-01".to_string(),
            author: "Test API Author".to_string(),
            image: Some("test-api-image.jpg".to_string()),
            slug: "test-api-post".to_string(),
            published: true,
            featured: false,
            tags: Vector::new(),
            metadata: im::HashMap::new(),
        }
    }

    #[tokio::test]
    async fn test_create_and_get_post() -> Result<()> {
        let (app, _temp_dir) = create_test_app().await?;
        let post = create_test_post();

        // Create post
        let create_request = Request::builder()
            .uri("/api/blog")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&post)?))?;

        let create_response = app.clone().oneshot(create_request).await?;
        assert_eq!(create_response.status(), StatusCode::OK);

        // Get post
        let get_request = Request::builder()
            .uri(format!("/api/blog/{}", post.slug))
            .method("GET")
            .body(Body::empty())?;

        let get_response = app.oneshot(get_request).await?;
        assert_eq!(get_response.status(), StatusCode::OK);

        Ok(())
    }

    #[tokio::test]
    async fn test_delete_post() -> Result<()> {
        let (app, _temp_dir) = create_test_app().await?;
        let post = create_test_post();

        // Create post
        let create_request = Request::builder()
            .uri("/api/blog")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&post)?))?;

        let create_response = app.clone().oneshot(create_request).await?;
        assert_eq!(create_response.status(), StatusCode::OK);

        // Delete post
        let delete_request = Request::builder()
            .uri(format!("/api/blog/{}", post.slug))
            .method("DELETE")
            .body(Body::empty())?;

        let delete_response = app.clone().oneshot(delete_request).await?;
        assert_eq!(delete_response.status(), StatusCode::OK);

        // Try to get deleted post
        let get_request = Request::builder()
            .uri(format!("/api/blog/{}", post.slug))
            .method("GET")
            .body(Body::empty())?;

        let get_response = app.oneshot(get_request).await?;
        assert_eq!(get_response.status(), StatusCode::NOT_FOUND);

        Ok(())
    }
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use axum::routing::get;
    use axum::Router;
    use serde_json::json;
    use std::sync::Arc;
    use tempfile::tempdir;
    use tower::ServiceExt;

    async fn create_test_app() -> Result<(Router, tempfile::TempDir)> {
        let temp_dir = tempdir()?;
        let db_path = temp_dir.path().join("test_blog.db");
        let router = create_blog_api_router(db_path.clone())?;
        Ok((router, temp_dir))
    }

    #[tokio::test]
    async fn test_get_all_posts_empty() -> Result<()> {
        let (app, _temp_dir) = create_test_app().await?;

        let response = app
            .oneshot(Request::builder().uri("/api/blog").body(Body::empty())?)
            .await?;

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await?
            .to_vec();
        let posts: Vector<BlogPost> = serde_json::from_slice(&body)?;
        assert_eq!(posts.len(), 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_create_and_get_post() -> Result<()> {
        let (app, _temp_dir) = create_test_app().await?;

        // Create a post
        let post_data = json!({
            "title": "Test Post",
            "slug": "test-post",
            "date": "2023-01-01",
            "author": "Test Author",
            "excerpt": "Test excerpt",
            "content": "Test content",
            "published": true,
            "featured": false,
            "tags": [{"name": "Test Tag", "slug": "test-tag"}]
        });

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/blog")
                    .header("Content-Type", "application/json")
                    .body(Body::from(post_data.to_string()))?
            )
            .await?;

        assert_eq!(response.status(), StatusCode::CREATED);

        // Get the post
        let response = app
            .oneshot(Request::builder().uri("/api/blog/test-post").body(Body::empty())?)
            .await?;

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await?
            .to_vec();
        let post: BlogPost = serde_json::from_slice(&body)?;
        assert_eq!(post.title, "Test Post");

        Ok(())
    }

    #[tokio::test]
    async fn test_get_nonexistent_post() -> Result<()> {
        let (app, _temp_dir) = create_test_app().await?;

        let response = app
            .oneshot(Request::builder().uri("/api/blog/nonexistent").body(Body::empty())?)
            .await?;

        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        Ok(())
    }

    #[tokio::test]
    async fn test_update_post() -> Result<()> {
        let (app, _temp_dir) = create_test_app().await?;

        // Create a post
        let post_data = json!({
            "title": "Test Post",
            "slug": "test-post",
            "date": "2023-01-01",
            "author": "Test Author",
            "excerpt": "Test excerpt",
            "content": "Test content",
            "published": true,
            "featured": false,
            "tags": [{"name": "Test Tag", "slug": "test-tag"}]
        });

        app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/blog")
                    .header("Content-Type", "application/json")
                    .body(Body::from(post_data.to_string()))?
            )
            .await?;

        // Update the post
        let updated_post_data = json!({
            "title": "Updated Post",
            "slug": "test-post",
            "date": "2023-01-01",
            "author": "Test Author",
            "excerpt": "Updated excerpt",
            "content": "Updated content",
            "published": true,
            "featured": false,
            "tags": [{"name": "Test Tag", "slug": "test-tag"}]
        });

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri("/api/blog/test-post")
                    .header("Content-Type", "application/json")
                    .body(Body::from(updated_post_data.to_string()))?
            )
            .await?;

        assert_eq!(response.status(), StatusCode::OK);

        // Get the updated post
        let response = app
            .oneshot(Request::builder().uri("/api/blog/test-post").body(Body::empty())?)
            .await?;

        let body = hyper::body::to_bytes(response.into_body()).await?
            .to_vec();
        let post: BlogPost = serde_json::from_slice(&body)?;
        assert_eq!(post.title, "Updated Post");
        assert_eq!(post.excerpt, "Updated excerpt");

        Ok(())
    }

    #[tokio::test]
    async fn test_delete_post() -> Result<()> {
        let (app, _temp_dir) = create_test_app().await?;

        // Create a post
        let post_data = json!({
            "title": "Test Post",
            "slug": "test-post",
            "date": "2023-01-01",
            "author": "Test Author",
            "excerpt": "Test excerpt",
            "content": "Test content",
            "published": true,
            "featured": false,
            "tags": [{"name": "Test Tag", "slug": "test-tag"}]
        });

        app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/blog")
                    .header("Content-Type", "application/json")
                    .body(Body::from(post_data.to_string()))?
            )
            .await?;
#[cfg(test)]
mod tests {
    use anyhow::Result;
    use tempfile::tempdir;
    use std::path::Path;
    use std::sync::Arc;
    use crate::blog_api::*;
    use crate::blog_data::*;

    fn create_test_post() -> BlogPost {
        BlogPost {
            id: None,
            title: "Test Post".to_string(),
            content: "Test content".to_string(),
            excerpt: "Test excerpt".to_string(),
            date: "2025-06-12".to_string(),
            author: "Test Author".to_string(),
            slug: "test-post".to_string(),
            image: None,
            published: true,
            featured: false,
            tags: im::Vector::new(),
            metadata: im::HashMap::new(),
        }
    }

    #[test]
    fn test_blog_api_create_router() -> Result<()> {
        let temp_dir = tempdir()?;
        let db_path = temp_dir.path().join("test.db");

        // Test that router creation works
        let router = create_blog_api_router(db_path.clone())?;
        assert!(router.into_service().local_addr().is_some());

        Ok(())
    }
}
        // Delete the post
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri("/api/blog/test-post")
                    .body(Body::empty())?
            )
            .await?;

        assert_eq!(response.status(), StatusCode::NO_CONTENT);

        // Verify the post is deleted
        let response = app
            .oneshot(Request::builder().uri("/api/blog/test-post").body(Body::empty())?)
            .await?;

        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        Ok(())
    }
}
