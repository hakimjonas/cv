/// Pure async blog service
///
/// This service replaces the problematic BlogManager with a clean async implementation.
/// No runtime creation, no sync wrappers - just pure async operations.
use anyhow::Result;
use im::Vector;

use crate::db::BlogRepository;
use crate::domain::blog::{BlogPost, PostId, Tag, TagId, UserId};

/// Async blog service for all blog-related operations
///
/// This service uses dependency injection and pure async patterns.
/// It operates on domain entities and delegates persistence to repositories.
pub struct BlogService {
    repository: BlogRepository,
}

impl BlogService {
    /// Create a new blog service with the given repository
    pub fn new(repository: BlogRepository) -> Self {
        Self { repository }
    }

    /// Get all blog posts
    pub async fn get_all_posts(&self) -> Result<Vector<BlogPost>> {
        let repo_posts = self.repository.get_all_posts().await?;
        let domain_posts = repo_posts
            .into_iter()
            .map(|post| self.convert_repo_to_domain(post))
            .collect();
        Ok(domain_posts)
    }

    /// Get a blog post by ID
    pub async fn get_post_by_id(&self, post_id: PostId) -> Result<Option<BlogPost>> {
        match self.repository.get_post_by_id(post_id.0).await? {
            Some(repo_post) => Ok(Some(self.convert_repo_to_domain(repo_post))),
            None => Ok(None),
        }
    }

    /// Get a blog post by slug
    pub async fn get_post_by_slug(&self, slug: &str) -> Result<Option<BlogPost>> {
        match self.repository.get_post_by_slug(slug).await? {
            Some(repo_post) => Ok(Some(self.convert_repo_to_domain(repo_post))),
            None => Ok(None),
        }
    }

    /// Get all published posts
    pub async fn get_published_posts(&self) -> Result<Vector<BlogPost>> {
        let repo_posts = self.repository.get_published_posts().await?;
        let domain_posts = repo_posts
            .into_iter()
            .map(|post| self.convert_repo_to_domain(post))
            .collect();
        Ok(domain_posts)
    }

    /// Get all featured posts
    pub async fn get_featured_posts(&self) -> Result<Vector<BlogPost>> {
        let repo_posts = self.repository.get_featured_posts().await?;
        let domain_posts = repo_posts
            .into_iter()
            .map(|post| self.convert_repo_to_domain(post))
            .collect();
        Ok(domain_posts)
    }

    /// Get posts by tag slug
    pub async fn get_posts_by_tag(&self, tag_slug: &str) -> Result<Vector<BlogPost>> {
        let repo_posts = self.repository.get_posts_by_tag(tag_slug).await?;
        let domain_posts = repo_posts
            .into_iter()
            .map(|post| self.convert_repo_to_domain(post))
            .collect();
        Ok(domain_posts)
    }

    /// Get all tags
    pub async fn get_all_tags(&self) -> Result<Vector<Tag>> {
        let repo_tags = self.repository.get_all_tags().await?;
        let domain_tags = repo_tags
            .into_iter()
            .map(|tag| self.convert_repo_tag_to_domain(tag))
            .collect();
        Ok(domain_tags)
    }

    /// Create or update a blog post
    pub async fn save_post(&self, post: &BlogPost) -> Result<PostId> {
        // Validate the post before saving
        post.validate()?;

        let repo_post = self.convert_domain_to_repo(post);
        let post_id = self.repository.create_or_update_post(&repo_post).await?;
        Ok(PostId(post_id))
    }

    /// Delete a blog post
    pub async fn delete_post(&self, post_id: PostId) -> Result<()> {
        self.repository.delete_post(post_id.0).await
    }

    /// Create a new blog post with validation
    pub async fn create_post(
        &self,
        title: &str,
        content: &str,
        author: &str,
        user_id: Option<UserId>,
    ) -> Result<BlogPost> {
        let mut post = BlogPost::with_title(title)
            .with_content(content)
            .with_author(author);

        if let Some(uid) = user_id {
            post = post.with_user_id(uid);
        }

        // Validate before creating
        post.validate()?;

        Ok(post)
    }

    /// Publish a post (set published = true)
    pub async fn publish_post(&self, post_id: PostId) -> Result<()> {
        if let Some(post) = self.get_post_by_id(post_id).await? {
            let updated_post = post.with_published(true);
            self.save_post(&updated_post).await?;
        }
        Ok(())
    }

    /// Unpublish a post (set published = false)
    pub async fn unpublish_post(&self, post_id: PostId) -> Result<()> {
        if let Some(post) = self.get_post_by_id(post_id).await? {
            let updated_post = post.with_published(false);
            self.save_post(&updated_post).await?;
        }
        Ok(())
    }

    /// Feature a post (set featured = true)
    pub async fn feature_post(&self, post_id: PostId) -> Result<()> {
        if let Some(post) = self.get_post_by_id(post_id).await? {
            let updated_post = post.with_featured(true);
            self.save_post(&updated_post).await?;
        }
        Ok(())
    }

    /// Unfeature a post (set featured = false)
    pub async fn unfeature_post(&self, post_id: PostId) -> Result<()> {
        if let Some(post) = self.get_post_by_id(post_id).await? {
            let updated_post = post.with_featured(false);
            self.save_post(&updated_post).await?;
        }
        Ok(())
    }

    /// Add a tag to a post
    pub async fn add_tag_to_post(&self, post_id: PostId, tag: Tag) -> Result<()> {
        if let Some(post) = self.get_post_by_id(post_id).await? {
            let updated_post = post.with_added_tag(tag);
            self.save_post(&updated_post).await?;
        }
        Ok(())
    }

    /// Search posts by title or content
    pub async fn search_posts(&self, query: &str) -> Result<Vector<BlogPost>> {
        let all_posts = self.get_all_posts().await?;
        let query_lower = query.to_lowercase();

        let matching_posts = all_posts
            .into_iter()
            .filter(|post| {
                post.title.to_lowercase().contains(&query_lower)
                    || post.content.to_lowercase().contains(&query_lower)
                    || post.excerpt.to_lowercase().contains(&query_lower)
            })
            .collect();

        Ok(matching_posts)
    }

    /// Get posts by author
    pub async fn get_posts_by_author(&self, author: &str) -> Result<Vector<BlogPost>> {
        let all_posts = self.get_all_posts().await?;
        let author_posts = all_posts
            .into_iter()
            .filter(|post| post.author == author)
            .collect();
        Ok(author_posts)
    }

    /// Get posts by user ID
    pub async fn get_posts_by_user_id(&self, user_id: UserId) -> Result<Vector<BlogPost>> {
        let all_posts = self.get_all_posts().await?;
        let user_posts = all_posts
            .into_iter()
            .filter(|post| post.user_id == Some(user_id))
            .collect();
        Ok(user_posts)
    }

    /// Get recent posts (limit the number returned)
    pub async fn get_recent_posts(&self, limit: usize) -> Result<Vector<BlogPost>> {
        let posts = self.get_published_posts().await?;

        // Sort by date (newest first) - simplified sorting
        let mut posts_vec: Vec<_> = posts.into_iter().collect();
        posts_vec.sort_by(|a, b| b.date.cmp(&a.date));

        // Take only the requested number
        posts_vec.truncate(limit);

        Ok(posts_vec.into_iter().collect())
    }

    // Conversion methods between domain and repository models
    fn convert_repo_to_domain(&self, repo_post: crate::db::repository::BlogPost) -> BlogPost {
        BlogPost {
            id: repo_post.id.map(PostId),
            title: repo_post.title,
            slug: repo_post.slug,
            date: repo_post.date,
            user_id: repo_post.user_id.map(UserId),
            author: repo_post.author,
            excerpt: repo_post.excerpt,
            content: repo_post.content,
            content_format: crate::domain::blog::ContentFormat::HTML, // Default for now
            published: repo_post.published,
            featured: repo_post.featured,
            image: repo_post.image,
            tags: repo_post
                .tags
                .into_iter()
                .map(|tag| self.convert_repo_tag_to_domain(tag))
                .collect(),
            metadata: repo_post.metadata,
        }
    }

    fn convert_domain_to_repo(&self, domain_post: &BlogPost) -> crate::db::repository::BlogPost {
        crate::db::repository::BlogPost {
            id: domain_post.id.map(|id| id.0),
            title: domain_post.title.clone(),
            slug: domain_post.slug.clone(),
            date: domain_post.date.clone(),
            user_id: domain_post.user_id.map(|id| id.0),
            author: domain_post.author.clone(),
            excerpt: domain_post.excerpt.clone(),
            content: domain_post.content.clone(),
            published: domain_post.published,
            featured: domain_post.featured,
            image: domain_post.image.clone(),
            tags: domain_post
                .tags
                .iter()
                .map(|tag| self.convert_domain_tag_to_repo(tag))
                .collect(),
            metadata: domain_post.metadata.clone(),
        }
    }

    fn convert_repo_tag_to_domain(&self, repo_tag: crate::db::repository::Tag) -> Tag {
        Tag {
            id: repo_tag.id.map(TagId),
            name: repo_tag.name,
            slug: repo_tag.slug,
        }
    }

    fn convert_domain_tag_to_repo(&self, domain_tag: &Tag) -> crate::db::repository::Tag {
        crate::db::repository::Tag {
            id: domain_tag.id.map(|id| id.0),
            name: domain_tag.name.clone(),
            slug: domain_tag.slug.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    // Test imports needed for blog service testing
    use crate::domain::blog::{BlogPost, Tag};

    #[tokio::test]
    async fn test_create_post() {
        // This would require a mock repository for testing
        // For now, just test the creation logic
        let post = BlogPost::with_title("Test Post")
            .with_content("Test content")
            .with_author("Test Author");

        assert_eq!(post.title, "Test Post");
        assert_eq!(post.slug, "test-post");
        assert_eq!(post.content, "Test content");
        assert_eq!(post.author, "Test Author");
        assert!(!post.published);
    }

    #[test]
    fn test_tag_conversion() {
        let domain_tag = Tag::new("Technology");
        assert_eq!(domain_tag.name, "Technology");
        assert_eq!(domain_tag.slug, "technology");
    }
}
