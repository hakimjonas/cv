use crate::blog_data_async::*;
use anyhow::Result;
use tempfile::TempDir;

#[cfg(test)]
mod tests {
    use super::*;
    use im::vector;

    async fn create_test_blog_manager() -> Result<(BlogManager, TempDir)> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test_blog.db");
        let blog_manager = BlogManager::new(&db_path).await?;
        Ok((blog_manager, temp_dir))
    }

    fn create_test_post() -> BlogPost {
        let tag = Tag {
            id: None,
            name: "Test Tag".to_string(),
            slug: "test-tag".to_string(),
        };

        BlogPost {
            id: None,
            title: "Test Post".to_string(),
            slug: "test-post".to_string(),
            date: "2023-01-01".to_string(),
            author: "Test Author".to_string(),
            excerpt: "Test excerpt".to_string(),
            content: "Test content".to_string(),
            published: true,
            featured: false,
            image: None,
            tags: vector![tag],
            metadata: im::HashMap::new(),
        }
    }

    #[tokio::test]
    async fn test_create_and_get_post() -> Result<()> {
        let (blog_manager, _temp_dir) = create_test_blog_manager().await?;
        let post = create_test_post();

        // Create the post
        let post_id = blog_manager.create_or_update_post(&post).await?;
        assert!(post_id > 0);

        // Get the post by slug
        let retrieved_post = blog_manager.get_post_by_slug(&post.slug).await?.unwrap();
        assert_eq!(retrieved_post.title, post.title);
        assert_eq!(retrieved_post.slug, post.slug);
        assert_eq!(retrieved_post.date, post.date);
        assert_eq!(retrieved_post.author, post.author);
        assert_eq!(retrieved_post.excerpt, post.excerpt);
        assert_eq!(retrieved_post.content, post.content);
        assert_eq!(retrieved_post.published, post.published);
        assert_eq!(retrieved_post.featured, post.featured);
        assert_eq!(retrieved_post.tags.len(), 1);
        assert_eq!(retrieved_post.tags[0].name, "Test Tag");

        Ok(())
    }

    #[tokio::test]
    async fn test_get_all_posts() -> Result<()> {
        let (blog_manager, _temp_dir) = create_test_blog_manager().await?;
        let post1 = create_test_post();
        let post2 = BlogPost {
            title: "Another Post".to_string(),
            slug: "another-post".to_string(),
            ..create_test_post()
        };

        // Create the posts
        blog_manager.create_or_update_post(&post1).await?;
        blog_manager.create_or_update_post(&post2).await?;

        // Get all posts
        let posts = blog_manager.get_all_posts().await?;
        assert_eq!(posts.len(), 2);

        Ok(())
    }

    #[tokio::test]
    async fn test_update_post() -> Result<()> {
        let (blog_manager, _temp_dir) = create_test_blog_manager().await?;
        // Add a small delay to ensure any previous connections are closed
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        let post = create_test_post();

        // Create the post
        let post_id = blog_manager.create_or_update_post(&post).await?;

        // Update the post
        let updated_post = BlogPost {
            id: Some(post_id),
            title: "Updated Post".to_string(),
            ..post.clone()
        };
        blog_manager.create_or_update_post(&updated_post).await?;

        // Get the updated post
        let retrieved_post = blog_manager.get_post_by_slug(&post.slug).await?.unwrap();
        assert_eq!(retrieved_post.title, "Updated Post");

        Ok(())
    }

    #[tokio::test]
    async fn test_delete_post() -> Result<()> {
        let (blog_manager, _temp_dir) = create_test_blog_manager().await?;
        let post = create_test_post();

        // Create the post
        let post_id = blog_manager.create_or_update_post(&post).await?;

        // Delete the post
        blog_manager.delete_post(post_id).await?;

        // Verify the post is deleted
        let retrieved_post = blog_manager.get_post_by_slug(&post.slug).await?;
        assert!(retrieved_post.is_none());

        Ok(())
    }

    #[tokio::test]
    async fn test_get_published_posts() -> Result<()> {
        let (blog_manager, _temp_dir) = create_test_blog_manager().await?;

        // Create a published post
        let published_post = create_test_post();
        blog_manager.create_or_update_post(&published_post).await?;

        // Create an unpublished post
        let unpublished_post = BlogPost {
            slug: "unpublished-post".to_string(),
            title: "Unpublished Post".to_string(),
            published: false,
            ..create_test_post()
        };
        blog_manager.create_or_update_post(&unpublished_post).await?;

        // Get published posts
        let published_posts = blog_manager.get_published_posts().await?;
        assert_eq!(published_posts.len(), 1);
        assert_eq!(published_posts[0].slug, published_post.slug);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_posts_by_tag() -> Result<()> {
        let (blog_manager, _temp_dir) = create_test_blog_manager().await?;

        // Create a post with tag "Test Tag"
        let post1 = create_test_post();
        blog_manager.create_or_update_post(&post1).await?;

        // Create a post with tag "Another Tag"
        let tag = Tag {
            id: None,
            name: "Another Tag".to_string(),
            slug: "another-tag".to_string(),
        };
        let post2 = BlogPost {
            slug: "another-post".to_string(),
            title: "Another Post".to_string(),
            tags: vector![tag],
            ..create_test_post()
        };
        blog_manager.create_or_update_post(&post2).await?;

        // Get posts by tag
        let tagged_posts = blog_manager.get_posts_by_tag("test-tag").await?;
        assert_eq!(tagged_posts.len(), 1);
        assert_eq!(tagged_posts[0].slug, post1.slug);

        Ok(())
    }
}