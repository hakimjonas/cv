use crate::blog_error::{BlogError, Result};
use crate::markdown_editor::utils::markdown_to_html;
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use im::Vector;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::path::Path;
use utoipa::ToSchema;

/// Represents a user role in the system
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, ToSchema)]
#[schema(description = "User role for authorization")]
pub enum UserRole {
    /// Administrator with full access
    Admin,
    /// Author who can create and edit their own posts
    Author,
    /// Editor who can edit but not create posts
    Editor,
    /// Viewer who can only view content
    Viewer,
}

impl Default for UserRole {
    fn default() -> Self {
        Self::Author
    }
}

/// Represents a user in the system
#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[schema(description = "A user account in the system")]
pub struct User {
    /// Unique identifier for the user (null for new users)
    #[schema(example = 1)]
    pub id: Option<i64>,

    /// Username for login
    #[schema(example = "johndoe")]
    pub username: String,

    /// Display name of the user
    #[schema(example = "John Doe")]
    pub display_name: String,

    /// Email address
    #[schema(example = "john.doe@example.com")]
    pub email: String,

    /// Password hash (not returned in API responses)
    #[serde(skip_serializing)]
    pub password_hash: String,

    /// User role for authorization
    pub role: UserRole,

    /// When the user was created
    pub created_at: String,

    /// When the user was last updated
    pub updated_at: String,
}

impl User {
    /// Creates a new user with default values
    pub fn new() -> Self {
        Self {
            id: None,
            username: String::new(),
            display_name: String::new(),
            email: String::new(),
            password_hash: String::new(),
            role: UserRole::default(),
            created_at: chrono::Local::now().to_rfc3339(),
            updated_at: chrono::Local::now().to_rfc3339(),
        }
    }

    /// Creates a new user with the given username, email, and password
    pub fn with_credentials(
        username: &str,
        display_name: &str,
        email: &str,
        password: &str,
    ) -> Result<Self> {
        let password_hash = Self::hash_password(password)?;

        Ok(Self {
            username: username.to_string(),
            display_name: display_name.to_string(),
            email: email.to_string(),
            password_hash,
            ..Self::new()
        })
    }

    /// Hashes a password using Argon2
    pub fn hash_password(password: &str) -> Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        argon2
            .hash_password(password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|e| BlogError::Internal(format!("Password hashing error: {e}")))
    }

    /// Verifies a password against the stored hash
    pub fn verify_password(&self, password: &str) -> Result<bool> {
        let parsed_hash = PasswordHash::new(&self.password_hash)
            .map_err(|e| BlogError::Internal(format!("Password hash parsing error: {e}")))?;

        Ok(Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }

    /// Returns a new user with updated username
    pub fn with_updated_username(self, username: &str) -> Self {
        Self {
            username: username.to_string(),
            updated_at: chrono::Local::now().to_rfc3339(),
            ..self
        }
    }

    /// Returns a new user with updated display name
    pub fn with_updated_display_name(self, display_name: &str) -> Self {
        Self {
            display_name: display_name.to_string(),
            updated_at: chrono::Local::now().to_rfc3339(),
            ..self
        }
    }

    /// Returns a new user with updated email
    pub fn with_updated_email(self, email: &str) -> Self {
        Self {
            email: email.to_string(),
            updated_at: chrono::Local::now().to_rfc3339(),
            ..self
        }
    }

    /// Returns a new user with updated password
    pub fn with_updated_password(self, password: &str) -> Result<Self> {
        let password_hash = Self::hash_password(password)?;

        Ok(Self {
            password_hash,
            updated_at: chrono::Local::now().to_rfc3339(),
            ..self
        })
    }

    /// Returns a new user with updated role
    pub fn with_updated_role(self, role: UserRole) -> Self {
        Self {
            role,
            updated_at: chrono::Local::now().to_rfc3339(),
            ..self
        }
    }
}

impl Default for User {
    fn default() -> Self {
        Self::new()
    }
}

/// Content format for blog posts
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, ToSchema)]
#[schema(description = "Format of the blog post content")]
pub enum ContentFormat {
    /// HTML content
    HTML,
    /// Markdown content
    Markdown,
}

impl Default for ContentFormat {
    fn default() -> Self {
        Self::HTML
    }
}

/// Represents a blog post tag
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, ToSchema)]
#[schema(description = "A tag for categorizing blog posts")]
pub struct Tag {
    /// Unique identifier for the tag (null for new tags)
    #[schema(example = 1)]
    pub id: Option<i64>,

    /// Display name of the tag
    #[schema(example = "Technology")]
    pub name: String,

    /// URL-friendly version of the name
    #[schema(example = "technology")]
    pub slug: String,
}

/// Represents a blog post with immutable data
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BlogPost {
    pub id: Option<i64>,
    pub title: String,
    pub slug: String,
    pub date: String,
    /// ID of the user who authored the post
    pub user_id: Option<i64>,
    /// Display name of the author (for backward compatibility and display purposes)
    pub author: String,
    pub excerpt: String,
    pub content: String,
    /// Format of the content (HTML or Markdown)
    pub content_format: ContentFormat,
    pub published: bool,
    pub featured: bool,
    pub image: Option<String>,
    pub tags: Vector<Tag>,
    pub metadata: im::HashMap<String, String>,
}

#[allow(dead_code)]
impl BlogPost {
    /// Creates a new empty blog post with default values
    pub fn new() -> Self {
        Self {
            id: None,
            title: String::new(),
            slug: String::new(),
            date: chrono::Local::now().format("%Y-%m-%d").to_string(),
            user_id: None,
            author: String::new(),
            excerpt: String::new(),
            content: String::new(),
            content_format: ContentFormat::default(),
            published: false,
            featured: false,
            image: None,
            tags: Vector::new(),
            metadata: im::HashMap::new(),
        }
    }

    /// Generates a slug from the title
    pub fn generate_slug_from_title(&self) -> String {
        self.title
            .to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join("-")
    }

    /// Creates a new blog post with the specified title and autogenerated slug
    pub fn with_title(title: &str) -> Self {
        let title_str = title.to_string();
        let post = Self {
            title: title_str,
            ..Self::new()
        };
        Self {
            slug: post.generate_slug_from_title(),
            ..post
        }
    }

    /// Returns a new blog post with updated title and slug
    pub fn with_updated_title(self, title: &str) -> Self {
        let post = Self {
            title: title.to_string(),
            ..self
        };
        Self {
            slug: post.generate_slug_from_title(),
            ..post
        }
    }

    /// Returns a new blog post with updated content
    pub fn with_updated_content(self, content: &str) -> Self {
        Self {
            content: content.to_string(),
            ..self
        }
    }

    /// Returns a new blog post with updated content and format
    pub fn with_updated_content_and_format(self, content: &str, format: ContentFormat) -> Self {
        Self {
            content: content.to_string(),
            content_format: format,
            ..self
        }
    }

    /// Returns a new blog post with updated content format
    pub fn with_updated_content_format(self, format: ContentFormat) -> Self {
        Self {
            content_format: format,
            ..self
        }
    }

    /// Returns a new blog post with updated excerpt
    pub fn with_updated_excerpt(self, excerpt: &str) -> Self {
        Self {
            excerpt: excerpt.to_string(),
            ..self
        }
    }

    /// Returns a new blog post with updated date
    pub fn with_updated_date(self, date: &str) -> Self {
        Self {
            date: date.to_string(),
            ..self
        }
    }

    /// Returns a new blog post with updated author
    pub fn with_updated_author(self, author: &str) -> Self {
        Self {
            author: author.to_string(),
            ..self
        }
    }

    /// Returns a new blog post with updated user ID and author name
    pub fn with_updated_user(self, user_id: i64, display_name: &str) -> Self {
        Self {
            user_id: Some(user_id),
            author: display_name.to_string(),
            ..self
        }
    }

    /// Returns a new blog post with updated user ID
    pub fn with_updated_user_id(self, user_id: i64) -> Self {
        Self {
            user_id: Some(user_id),
            ..self
        }
    }

    /// Returns a new blog post with updated image
    pub fn with_updated_image(self, image: Option<String>) -> Self {
        Self { image, ..self }
    }

    /// Returns a new blog post with updated published state
    pub fn with_updated_published(self, published: bool) -> Self {
        Self { published, ..self }
    }

    /// Returns a new blog post with updated featured state
    pub fn with_updated_featured(self, featured: bool) -> Self {
        Self { featured, ..self }
    }

    /// Returns a new blog post with updated tags
    pub fn with_updated_tags(self, tags: Vector<Tag>) -> Self {
        Self { tags, ..self }
    }

    /// Returns a new blog post with updated metadata
    pub fn with_updated_metadata(self, metadata: im::HashMap<String, String>) -> Self {
        Self { metadata, ..self }
    }

    /// Returns a new blog post with added tag
    pub fn with_added_tag(self, tag: Tag) -> Self {
        // Create a new vector with the tag added
        let mut new_tags = self.tags.clone();
        new_tags.push_back(tag);
        Self {
            tags: new_tags,
            ..self
        }
    }

    /// Returns a new blog post with removed tag
    pub fn with_removed_tag(self, tag_name: &str) -> Self {
        Self {
            tags: self
                .tags
                .into_iter()
                .filter(|tag| tag.name != tag_name)
                .collect(),
            ..self
        }
    }

    /// Returns a new blog post with added metadata
    pub fn with_added_metadata(self, key: &str, value: &str) -> Self {
        Self {
            metadata: self.metadata.update(key.to_string(), value.to_string()),
            ..self
        }
    }

    /// Returns a new blog post with removed metadata
    pub fn with_removed_metadata(self, key: &str) -> Self {
        let new_metadata = self.metadata.without(key);
        Self {
            metadata: new_metadata,
            ..self
        }
    }

    /// Renders the content based on its format
    ///
    /// If the content is in Markdown format, it will be converted to HTML.
    /// If the content is already in HTML format, it will be returned as is.
    ///
    /// # Returns
    ///
    /// A Result containing the rendered HTML content
    pub fn render_content(&self) -> Result<String> {
        match self.content_format {
            ContentFormat::HTML => {
                // Content is already in HTML format
                Ok(self.content.clone())
            }
            ContentFormat::Markdown => {
                // Convert Markdown to HTML
                markdown_to_html(&self.content)
                    .map_err(|e| BlogError::Internal(format!("Failed to render Markdown: {e}")))
            }
        }
    }
}

impl Default for BlogPost {
    fn default() -> Self {
        Self::new()
    }
}

/// Manages blog posts using a repository pattern
use crate::blog_converters;
use crate::db::{BlogRepository, Database};
use tokio::runtime::Runtime;

pub struct BlogManager {
    repository: BlogRepository,
    runtime: Runtime,
}

#[allow(dead_code)]
impl BlogManager {
    /// Creates a new BlogManager with the given SQLite database path
    pub fn new(db_path: &Path) -> Result<Self> {
        // Create a database instance
        let db = Database::new(db_path)?;

        // Get a repository from the database
        let repository = db.blog_repository();

        // Create a runtime for executing async code in a synchronous context
        let runtime = Runtime::new()
            .map_err(|e| BlogError::Internal(format!("Failed to create runtime: {e}")))?;

        Ok(Self {
            repository,
            runtime,
        })
    }

    /// Initializes the database with required tables
    fn initialize_db(conn: &Connection) -> Result<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS blog_posts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                slug TEXT NOT NULL UNIQUE,
                date TEXT NOT NULL,
                author TEXT NOT NULL,
                excerpt TEXT NOT NULL,
                content TEXT NOT NULL,
                published BOOLEAN NOT NULL,
                featured BOOLEAN NOT NULL,
                image TEXT
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS tags (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                slug TEXT NOT NULL UNIQUE
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS post_tags (
                post_id INTEGER NOT NULL,
                tag_id INTEGER NOT NULL,
                PRIMARY KEY (post_id, tag_id),
                FOREIGN KEY (post_id) REFERENCES blog_posts(id) ON DELETE CASCADE,
                FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS post_metadata (
                post_id INTEGER NOT NULL,
                key TEXT NOT NULL,
                value TEXT NOT NULL,
                PRIMARY KEY (post_id, key),
                FOREIGN KEY (post_id) REFERENCES blog_posts(id) ON DELETE CASCADE
            )",
            [],
        )?;

        Ok(())
    }

    /// Gets all blog posts
    pub fn get_all_posts(&self) -> Result<Vector<BlogPost>> {
        // Use the runtime to execute the async method synchronously
        let repo_posts = self
            .runtime
            .block_on(self.repository.get_all_posts())
            .map_err(|e| BlogError::Internal(format!("Failed to get posts: {e}")))?;

        // Convert repository posts to blog_data posts
        let posts = blog_converters::convert_posts_to_data(&repo_posts);

        Ok(posts)
    }

    /// Gets a blog post by ID
    pub fn get_post_by_id(&self, post_id: i64) -> Result<Option<BlogPost>> {
        // Use the runtime to execute the async method synchronously
        let repo_post = self
            .runtime
            .block_on(self.repository.get_post_by_id(post_id))
            .map_err(|e| BlogError::Internal(format!("Failed to get post by ID {post_id}: {e}")))?;

        // Convert repository post to blog_data post if found
        Ok(repo_post.map(|post| blog_converters::repo_to_data(&post)))
    }

    pub fn get_post_by_slug(&self, slug: &str) -> Result<Option<BlogPost>> {
        // Use the runtime to execute the async method synchronously
        let repo_post = self
            .runtime
            .block_on(self.repository.get_post_by_slug(slug))
            .map_err(|e| {
                BlogError::Internal(format!("Failed to get post by slug '{slug}': {e}"))
            })?;

        // Convert repository post to blog_data post if found
        Ok(repo_post.map(|post| blog_converters::repo_to_data(&post)))
    }

    /// Gets all tags
    pub fn get_all_tags(&self) -> Result<Vector<Tag>> {
        // Use the runtime to execute the async method synchronously
        let repo_tags = self
            .runtime
            .block_on(self.repository.get_all_tags())
            .map_err(|e| BlogError::Internal(format!("Failed to get all tags: {e}")))?;

        // Convert repository tags to blog_data tags
        let tags = blog_converters::convert_tags_to_data(&repo_tags);

        Ok(tags)
    }

    /// Creates or updates a blog post
    pub fn create_or_update_post(&self, post: &BlogPost) -> Result<i64> {
        println!("Starting create_or_update_post operation...");

        // Convert blog_data post to repository post
        let repo_post = blog_converters::data_to_repo(post);

        // Use the runtime to execute the async method synchronously
        let post_id = self
            .runtime
            .block_on(self.repository.create_or_update_post(&repo_post))
            .map_err(|e| BlogError::Internal(format!("Failed to create or update post: {e}")))?;

        println!("Post operation completed with ID: {post_id}");
        Ok(post_id)
    }

    /// Deletes a blog post
    pub fn delete_post(&self, post_id: i64) -> Result<()> {
        // Use the runtime to execute the async method synchronously
        self.runtime
            .block_on(self.repository.delete_post(post_id))
            .map_err(|e| {
                BlogError::Internal(format!("Failed to delete post with ID {post_id}: {e}"))
            })?;

        println!("Post with ID {post_id} deleted successfully");
        Ok(())
    }

    /// Gets all published blog posts
    pub fn get_published_posts(&self) -> Result<Vector<BlogPost>> {
        // Use the runtime to execute the async method synchronously
        let repo_posts = self
            .runtime
            .block_on(self.repository.get_published_posts())
            .map_err(|e| BlogError::Internal(format!("Failed to get published posts: {e}")))?;

        // Convert repository posts to blog_data posts
        let posts = blog_converters::convert_posts_to_data(&repo_posts);

        Ok(posts)
    }

    /// Gets all featured blog posts
    pub fn get_featured_posts(&self) -> Result<Vector<BlogPost>> {
        // Use the runtime to execute the async method synchronously
        let repo_posts = self
            .runtime
            .block_on(self.repository.get_featured_posts())
            .map_err(|e| BlogError::Internal(format!("Failed to get featured posts: {e}")))?;

        // Convert repository posts to blog_data posts
        let posts = blog_converters::convert_posts_to_data(&repo_posts);

        Ok(posts)
    }

    /// Gets posts by tag
    pub fn get_posts_by_tag(&self, tag_slug: &str) -> Result<Vector<BlogPost>> {
        // Use the runtime to execute the async method synchronously
        let repo_posts = self
            .runtime
            .block_on(self.repository.get_posts_by_tag(tag_slug))
            .map_err(|e| {
                BlogError::Internal(format!("Failed to get posts by tag '{tag_slug}': {e}"))
            })?;

        // Convert repository posts to blog_data posts
        let posts = blog_converters::convert_posts_to_data(&repo_posts);

        Ok(posts)
    }
}
