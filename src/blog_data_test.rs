use crate::blog_data::*;
use tempfile::TempDir;
use anyhow::Result;

#[cfg(test)]
mod tests {
    use super::*;
    use im::vector;

    #[test]
    fn test_blog_post_new() {
        let post = BlogPost::new();
        assert_eq!(post.title, "");
        assert_eq!(post.slug, "");
        assert_eq!(post.published, false);
        assert_eq!(post.featured, false);
        assert!(post.tags.is_empty());
        assert!(post.metadata.is_empty());
    }

    #[test]
    fn test_generate_slug_from_title() {
        let post = BlogPost {
            id: None,
            title: "This is a Test Post!".to_string(),
            slug: "".to_string(),
            date: "2023-01-01".to_string(),
            author: "Test Author".to_string(),
            excerpt: "Test excerpt".to_string(),
            content: "Test content".to_string(),
            published: false,
            featured: false,
            image: None,
            tags: Vector::new(),
            metadata: im::HashMap::new(),
        };

        let slug = post.generate_slug_from_title();
        assert_eq!(slug, "this-is-a-test-post-");
    }

    #[test]
    fn test_with_added_tag() {
        let post = BlogPost {
            id: None,
            title: "Test Post".to_string(),
            slug: "test-post".to_string(),
            date: "2023-01-01".to_string(),
            author: "Test Author".to_string(),
            excerpt: "Test excerpt".to_string(),
            content: "Test content".to_string(),
            published: false,
            featured: false,
            image: None,
            tags: Vector::new(),
            metadata: im::HashMap::new(),
        };

        let tag = Tag {
            id: None,
            name: "Test Tag".to_string(),
            slug: "test-tag".to_string(),
        };

        let updated_post = post.with_added_tag(tag.clone());

        // Original post should be unchanged
        assert_eq!(post.tags.len(), 0);

        // Updated post should have the new tag
        assert_eq!(updated_post.tags.len(), 1);
        assert_eq!(updated_post.tags[0], tag);
    }

    #[test]
    fn test_with_removed_tag() {
        let tag1 = Tag {
            id: None,
            name: "Tag 1".to_string(),
            slug: "tag-1".to_string(),
        };

        let tag2 = Tag {
            id: None,
            name: "Tag 2".to_string(),
            slug: "tag-2".to_string(),
        };

        let post = BlogPost {
            id: None,
            title: "Test Post".to_string(),
            slug: "test-post".to_string(),
            date: "2023-01-01".to_string(),
            author: "Test Author".to_string(),
            excerpt: "Test excerpt".to_string(),
            content: "Test content".to_string(),
            published: false,
            featured: false,
            image: None,
            tags: vector![tag1.clone(), tag2.clone()],
            metadata: im::HashMap::new(),
        };

        let updated_post = post.with_removed_tag(&tag1.name);

        // Original post should be unchanged
        assert_eq!(post.tags.len(), 2);

        // Updated post should have one tag removed
        assert_eq!(updated_post.tags.len(), 1);
        assert_eq!(updated_post.tags[0], tag2);
    }

    #[test]
    fn test_with_added_metadata() {
        let post = BlogPost {
            id: None,
            title: "Test Post".to_string(),
            slug: "test-post".to_string(),
            date: "2023-01-01".to_string(),
            author: "Test Author".to_string(),
            excerpt: "Test excerpt".to_string(),
            content: "Test content".to_string(),
            published: false,
            featured: false,
            image: None,
            tags: Vector::new(),
            metadata: im::HashMap::new(),
        };

        let updated_post = post.with_added_metadata("key", "value");

        // Original post should be unchanged
        assert!(post.metadata.is_empty());

        // Updated post should have the new metadata
        assert_eq!(updated_post.metadata.len(), 1);
        assert_eq!(updated_post.metadata.get("key").unwrap(), "value");
    }

    #[test]
    fn test_with_removed_metadata() {
        let mut metadata = im::HashMap::new();
        metadata.insert("key1".to_string(), "value1".to_string());
        metadata.insert("key2".to_string(), "value2".to_string());

        let post = BlogPost {
            id: None,
            title: "Test Post".to_string(),
            slug: "test-post".to_string(),
            date: "2023-01-01".to_string(),
            author: "Test Author".to_string(),
            excerpt: "Test excerpt".to_string(),
            content: "Test content".to_string(),
            published: false,
            featured: false,
            image: None,
            tags: Vector::new(),
            metadata: metadata.clone(),
        };

        let updated_post = post.with_removed_metadata("key1");

        // Original post should be unchanged
        assert_eq!(post.metadata.len(), 2);

        // Updated post should have one metadata removed
        assert_eq!(updated_post.metadata.len(), 1);
        assert!(updated_post.metadata.get("key1").is_none());
        assert_eq!(updated_post.metadata.get("key2").unwrap(), "value2");
    }

    #[test]
    fn test_comprehensive_blog_post_operations() {
        // Create a test database connection
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_blog.db");

        // Initialize the database
        let blog_db = BlogData::new(db_path.to_str().unwrap()).unwrap();

        // Test post creation
        let mut post = BlogPost::new();
        post.title = "Comprehensive Test Post".to_string();
        post.slug = "comprehensive-test".to_string();
        post.date = "2023-01-01".to_string();
        post.author = "Test Author".to_string();
        post.excerpt = "Test excerpt".to_string();
        post.content = "Test content with **markdown**".to_string();
        post.published = true;

        // Add tags
        let tag1 = Tag {
            id: None,
            name: "Testing".to_string(),
            slug: "testing".to_string(),
        };

        let tag2 = Tag {
            id: None,
            name: "Rust".to_string(),
            slug: "rust".to_string(),
        };

        post = post.with_added_tag(tag1.clone());
        post = post.with_added_tag(tag2.clone());

        // Add metadata
        post = post.with_added_metadata("test-key", "test-value");

        // Save the post
        let post_id = blog_db.save_post(&post).unwrap();
        assert!(post_id > 0, "Post ID should be positive");

        // Retrieve the post
        let retrieved_post = blog_db.get_post_by_slug(&post.slug).unwrap();
        assert_eq!(retrieved_post.title, post.title);
        assert_eq!(retrieved_post.tags.len(), 2);
        assert_eq!(retrieved_post.metadata.len(), 1);
        assert_eq!(retrieved_post.metadata.get("test-key").unwrap(), "test-value");

        // Update the post
        let updated_post = retrieved_post
            .with_title("Updated Test Post")
            .with_removed_tag(&tag1.name)
            .with_added_metadata("new-key", "new-value");

        blog_db.update_post(&updated_post).unwrap();

        // Retrieve the updated post
        let retrieved_updated_post = blog_db.get_post_by_slug(&post.slug).unwrap();
        assert_eq!(retrieved_updated_post.title, "Updated Test Post");
        assert_eq!(retrieved_updated_post.tags.len(), 1);
        assert_eq!(retrieved_updated_post.tags[0].name, "Rust");
        assert_eq!(retrieved_updated_post.metadata.len(), 2);

        // Test getting all posts
        let all_posts = blog_db.get_all_posts().unwrap();
        assert_eq!(all_posts.len(), 1);

        // Test getting posts by tag
        let posts_by_tag = blog_db.get_posts_by_tag(&tag2.slug).unwrap();
        assert_eq!(posts_by_tag.len(), 1);

        let posts_by_nonexistent_tag = blog_db.get_posts_by_tag("nonexistent").unwrap();
        assert_eq!(posts_by_nonexistent_tag.len(), 0);

        // Test getting all tags
        let all_tags = blog_db.get_all_tags().unwrap();
        assert_eq!(all_tags.len(), 1);

        // Test deleting the post
        blog_db.delete_post(post_id).unwrap();

        // Verify the post is deleted
        let posts_after_delete = blog_db.get_all_posts().unwrap();
        assert_eq!(posts_after_delete.len(), 0);
    }
}
