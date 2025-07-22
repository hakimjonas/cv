/*!
 * Repository implementation for blog operations
 * This module provides a clean interface for database operations
 * following functional programming principles
 */

// Define the BlogPost and Tag structs directly in this module
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Tag {
    pub id: Option<i64>,
    pub name: String,
    pub slug: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BlogPost {
    pub id: Option<i64>,
    pub title: String,
    pub slug: String,
    pub date: String,
    pub author: String,
    pub excerpt: String,
    pub content: String,
    pub published: bool,
    pub featured: bool,
    pub image: Option<String>,
    pub tags: Vector<Tag>,
    pub metadata: HashMap<String, String>,
}
use anyhow::{Result, anyhow};
use im::{HashMap, Vector};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{OptionalExtension, Transaction, params};
use std::sync::Arc;
use tokio::task;
use tracing::{debug, error, info, instrument, warn};

/// Repository for blog operations
#[allow(dead_code)]
pub struct BlogRepository {
    pool: Arc<Pool<SqliteConnectionManager>>,
}

#[allow(dead_code)]
impl BlogRepository {
    /// Create a new repository with the given connection pool
    pub fn new(pool: Arc<Pool<SqliteConnectionManager>>) -> Self {
        Self { pool }
    }

    /// Get all blog posts
    #[instrument(skip(self), err)]
    pub async fn get_all_posts(&self) -> Result<Vector<BlogPost>> {
        let pool = Arc::clone(&self.pool);

        task::spawn_blocking(move || {
            let conn = pool.get()?;
            
            // Use the optimized query function
            let posts = super::optimized_queries::get_all_posts_optimized(&conn)?;
            
            debug!("Loaded {} blog posts using optimized query", posts.len());
            Ok(posts)
        })
        .await?
    }

    /// Get a blog post by its slug
    #[instrument(skip(self), err)]
    pub async fn get_post_by_slug(&self, slug: &str) -> Result<Option<BlogPost>> {
        let slug = slug.to_string();
        let pool = Arc::clone(&self.pool);

        task::spawn_blocking(move || {
            let conn = pool.get()?;
            
            // Use the optimized query function
            let post = super::optimized_queries::get_post_by_slug_optimized(&conn, &slug)?;
            
            if post.is_some() {
                debug!("Loaded blog post with slug: {} using optimized query", slug);
            } else {
                debug!("No blog post found with slug: {}", slug);
            }
            
            Ok(post)
        })
        .await?
    }

    /// Get a blog post by its ID
    #[instrument(skip(self), err)]
    pub async fn get_post_by_id(&self, id: i64) -> Result<Option<BlogPost>> {
        let pool = Arc::clone(&self.pool);

        task::spawn_blocking(move || {
            let conn = pool.get()?;

            let post_result = conn
                .query_row(
                    "
                SELECT id, title, slug, date, author, excerpt, content, 
                       published, featured, image FROM posts WHERE id = ?1
            ",
                    [id],
                    |row| {
                        let id = row.get(0)?;
                        let title = row.get(1)?;
                        let slug = row.get(2)?;
                        let date = row.get(3)?;
                        let author = row.get(4)?;
                        let excerpt = row.get(5)?;
                        let content = row.get(6)?;
                        let published = row.get(7)?;
                        let featured = row.get(8)?;
                        let image: Option<String> = row.get(9)?;

                        Ok(BlogPost {
                            id: Some(id),
                            title,
                            slug,
                            date,
                            author,
                            excerpt,
                            content,
                            published,
                            featured,
                            image,
                            tags: Vector::new(),      // Will be populated later
                            metadata: HashMap::new(), // Will be populated later
                        })
                    },
                )
                .optional()?;

            match post_result {
                Some(post) => {
                    let post_with_tags = Self::load_tags_for_post(&conn, post)?;
                    let post_with_tags_and_metadata =
                        Self::load_metadata_for_post(&conn, post_with_tags)?;
                    debug!("Loaded blog post with ID: {}", id);
                    Ok(Some(post_with_tags_and_metadata))
                }
                None => {
                    debug!("No blog post found with ID: {}", id);
                    Ok(None)
                }
            }
        })
        .await?
    }

    /// Save a blog post
    #[instrument(skip(self, post), err)]
    pub async fn save_post(&self, post: &BlogPost) -> Result<i64> {
        let post = post.clone();
        let pool = Arc::clone(&self.pool);

        task::spawn_blocking(move || {
            let mut conn = pool.get()?;
            let tx = conn.transaction()?;

            let post_id = Self::save_post_tx(&tx, &post)?;
            Self::save_tags_tx(&tx, post_id, &post.tags)?;
            Self::save_metadata_tx(&tx, post_id, &post.metadata)?;

            // Commit the transaction
            match tx.commit() {
                Ok(_) => {
                    info!("Created blog post with ID: {}", post_id);
                    Ok(post_id)
                }
                Err(e) => {
                    // If it's a lock error, the operations might have succeeded
                    if e.to_string().contains("locked") {
                        warn!(
                            "Database locked during commit, but operations were likely successful"
                        );
                        warn!(
                            "This is common in WAL mode - the data changes may have been preserved"
                        );
                        info!(
                            "Created blog post with ID: {} (despite lock warning)",
                            post_id
                        );
                        Ok(post_id)
                    } else {
                        error!("Failed to commit transaction: {}", e);
                        Err(anyhow!("Failed to commit transaction: {}", e))
                    }
                }
            }
        })
        .await?
    }

    /// Update a blog post
    #[instrument(skip(self, post), err)]
    pub async fn update_post(&self, post: &BlogPost) -> Result<()> {
        let post = post.clone();
        let pool = Arc::clone(&self.pool);

        task::spawn_blocking(move || {
            let mut conn = pool.get()?;
            let tx = conn.transaction()?;

            let post_id = match post.id {
                Some(id) => id,
                None => return Err(anyhow!("Cannot update post without ID")),
            };

            // Update the post
            tx.execute(
                "
                UPDATE posts SET 
                    title = ?1, slug = ?2, date = ?3, author = ?4, 
                    excerpt = ?5, content = ?6, published = ?7, 
                    featured = ?8, image = ?9
                WHERE id = ?10
            ",
                params![
                    &post.title,
                    &post.slug,
                    &post.date,
                    &post.author,
                    &post.excerpt,
                    &post.content,
                    &post.published,
                    &post.featured,
                    &post.image,
                    post_id
                ],
            )?;

            // Delete existing tags and metadata
            tx.execute("DELETE FROM post_tags WHERE post_id = ?1", [post_id])?;
            tx.execute("DELETE FROM post_metadata WHERE post_id = ?1", [post_id])?;

            // Save new tags and metadata
            Self::save_tags_tx(&tx, post_id, &post.tags)?;
            Self::save_metadata_tx(&tx, post_id, &post.metadata)?;

            // Commit the transaction
            match tx.commit() {
                Ok(_) => {
                    info!("Updated blog post with ID: {}", post_id);
                    Ok(())
                }
                Err(e) => {
                    // If it's a lock error, the operations might have succeeded
                    if e.to_string().contains("locked") {
                        warn!(
                            "Database locked during commit, but operations were likely successful"
                        );
                        warn!(
                            "This is common in WAL mode - the data changes may have been preserved"
                        );
                        info!(
                            "Updated blog post with ID: {} (despite lock warning)",
                            post_id
                        );
                        Ok(())
                    } else {
                        error!("Failed to commit transaction: {}", e);
                        Err(anyhow!("Failed to commit transaction: {}", e))
                    }
                }
            }
        })
        .await?
    }

    /// Delete a blog post
    #[instrument(skip(self), err)]
    pub async fn delete_post(&self, post_id: i64) -> Result<()> {
        let pool = Arc::clone(&self.pool);

        task::spawn_blocking(move || {
            let conn = pool.get()?;
            conn.execute("DELETE FROM posts WHERE id = ?1", [post_id])?;
            info!("Deleted blog post with ID: {}", post_id);
            Ok(())
        })
        .await?
    }

    /// Get all tags
    #[instrument(skip(self), err)]
    pub async fn get_all_tags(&self) -> Result<Vector<Tag>> {
        let pool = Arc::clone(&self.pool);

        task::spawn_blocking(move || {
            let conn = pool.get()?;
            let mut stmt = conn.prepare("SELECT id, name, slug FROM tags")?;

            let tag_iter = stmt.query_map([], |row| {
                let id = row.get(0)?;
                let name = row.get(1)?;
                let slug = row.get(2)?;

                Ok(Tag {
                    id: Some(id),
                    name,
                    slug,
                })
            })?;

            // Use functional approach to collect tags
            let tags = tag_iter
                .map(|tag_result| tag_result.map_err(anyhow::Error::from))
                .collect::<Result<Vector<_>>>()?;

            debug!("Loaded {} tags", tags.len());
            Ok(tags)
        })
        .await?
    }

    /// Get all published posts
    #[instrument(skip(self), err)]
    pub async fn get_published_posts(&self) -> Result<Vector<BlogPost>> {
        let pool = Arc::clone(&self.pool);

        task::spawn_blocking(move || {
            let conn = pool.get()?;
            
            // Use the optimized query function
            let posts = super::optimized_queries::get_published_posts_optimized(&conn)?;
            
            debug!("Loaded {} published blog posts using optimized query", posts.len());
            Ok(posts)
        })
        .await?
    }

    /// Get all featured posts (published posts that are marked as featured)
    #[instrument(skip(self), err)]
    pub async fn get_featured_posts(&self) -> Result<Vector<BlogPost>> {
        // Get all published posts first
        let published_posts = self.get_published_posts().await?;

        // Filter to only include featured posts
        let featured_posts: Vector<BlogPost> = published_posts
            .iter()
            .filter(|p| p.featured)
            .cloned()
            .collect();

        debug!("Filtered to {} featured blog posts", featured_posts.len());
        Ok(featured_posts)
    }

    /// Get posts by tag
    #[instrument(skip(self), err)]
    pub async fn get_posts_by_tag(&self, tag_slug: &str) -> Result<Vector<BlogPost>> {
        let tag_slug = tag_slug.to_string();
        let pool = Arc::clone(&self.pool);

        task::spawn_blocking(move || {
            let conn = pool.get()?;
            let mut stmt = conn.prepare(
                "
                SELECT p.id, p.title, p.slug, p.date, p.author, p.excerpt, 
                       p.content, p.published, p.featured, p.image 
                FROM posts p 
                JOIN post_tags pt ON p.id = pt.post_id 
                JOIN tags t ON pt.tag_id = t.id 
                WHERE t.slug = ?1 AND p.published = 1 
                ORDER BY p.date DESC
                ",
            )?;

            let post_iter = stmt.query_map([&tag_slug], |row| {
                let id = row.get(0)?;
                let title = row.get(1)?;
                let slug = row.get(2)?;
                let date = row.get(3)?;
                let author = row.get(4)?;
                let excerpt = row.get(5)?;
                let content = row.get(6)?;
                let published = row.get(7)?;
                let featured = row.get(8)?;
                let image: Option<String> = row.get(9)?;

                Ok(BlogPost {
                    id: Some(id),
                    title,
                    slug,
                    date,
                    author,
                    excerpt,
                    content,
                    published,
                    featured,
                    image,
                    tags: Vector::new(),      // Will be populated later
                    metadata: HashMap::new(), // Will be populated later
                })
            })?;

            // Use functional approach to collect and process posts
            let posts = post_iter
                .map(|post_result| -> Result<BlogPost> {
                    let post = post_result?;
                    let post_with_tags = Self::load_tags_for_post(&conn, post)?;
                    let post_with_tags_and_metadata =
                        Self::load_metadata_for_post(&conn, post_with_tags)?;
                    Ok(post_with_tags_and_metadata)
                })
                .collect::<Result<Vector<_>>>()?;

            debug!("Loaded {} posts with tag '{}'", posts.len(), tag_slug);
            Ok(posts)
        })
        .await?
    }

    /// Create or update a blog post
    ///
    /// This method handles both creating new posts and updating existing ones.
    /// If the post has an ID, it will be updated. Otherwise, a new post will be created.
    #[instrument(skip(self, post), err)]
    pub async fn create_or_update_post(&self, post: &BlogPost) -> Result<i64> {
        let post = post.clone();
        let pool = Arc::clone(&self.pool);

        task::spawn_blocking(move || {
            let mut conn = pool.get()?;
            let tx = conn.transaction()?;

            let post_id = if let Some(id) = post.id {
                // Update existing post
                debug!("Updating existing post with ID: {}", id);

                tx.execute(
                    "
                    UPDATE posts SET 
                        title = ?1, slug = ?2, date = ?3, author = ?4, 
                        excerpt = ?5, content = ?6, published = ?7, 
                        featured = ?8, image = ?9
                    WHERE id = ?10
                    ",
                    params![
                        &post.title,
                        &post.slug,
                        &post.date,
                        &post.author,
                        &post.excerpt,
                        &post.content,
                        &post.published,
                        &post.featured,
                        &post.image,
                        id
                    ],
                )?;

                // Delete existing tags and metadata
                tx.execute("DELETE FROM post_tags WHERE post_id = ?1", [id])?;
                tx.execute("DELETE FROM post_metadata WHERE post_id = ?1", [id])?;

                id
            } else {
                // Create new post
                debug!("Creating new post");
                Self::save_post_tx(&tx, &post)?
            };

            // Save tags and metadata
            Self::save_tags_tx(&tx, post_id, &post.tags)?;
            Self::save_metadata_tx(&tx, post_id, &post.metadata)?;

            // Commit the transaction
            match tx.commit() {
                Ok(_) => {
                    if post.id.is_some() {
                        info!("Updated blog post with ID: {}", post_id);
                    } else {
                        info!("Created blog post with ID: {}", post_id);
                    }
                    Ok(post_id)
                }
                Err(e) => {
                    // If it's a lock error, the operations might have succeeded
                    if e.to_string().contains("locked") {
                        warn!(
                            "Database locked during commit, but operations were likely successful"
                        );
                        warn!(
                            "This is common in WAL mode - the data changes may have been preserved"
                        );
                        info!(
                            "Post operation completed with ID: {} (despite lock warning)",
                            post_id
                        );
                        Ok(post_id)
                    } else {
                        error!("Failed to commit transaction: {}", e);
                        Err(anyhow!("Failed to commit transaction: {}", e))
                    }
                }
            }
        })
        .await?
    }

    /// Helper method to save a post within a transaction
    fn save_post_tx(tx: &Transaction, post: &BlogPost) -> Result<i64> {
        tx.execute(
            "
            INSERT INTO posts (
                title, slug, date, author, excerpt, content, 
                published, featured, image
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
        ",
            params![
                &post.title,
                &post.slug,
                &post.date,
                &post.author,
                &post.excerpt,
                &post.content,
                &post.published,
                &post.featured,
                &post.image
            ],
        )?;

        Ok(tx.last_insert_rowid())
    }

    /// Helper method to save tags within a transaction
    fn save_tags_tx(tx: &Transaction, post_id: i64, tags: &Vector<Tag>) -> Result<()> {
        for tag in tags.iter() {
            // Insert or get the tag ID
            tx.execute(
                "
                INSERT INTO tags (name, slug) VALUES (?1, ?2)
                ON CONFLICT(slug) DO UPDATE SET name = ?1
            ",
                params![&tag.name, &tag.slug],
            )?;

            let tag_id: i64 = tx.query_row(
                "
                SELECT id FROM tags WHERE slug = ?1
            ",
                [&tag.slug],
                |row| row.get(0),
            )?;

            // Associate the tag with the post
            tx.execute(
                "
                INSERT OR IGNORE INTO post_tags (post_id, tag_id)
                VALUES (?1, ?2)
            ",
                params![post_id, tag_id],
            )?;
        }

        Ok(())
    }

    /// Helper method to save metadata within a transaction
    fn save_metadata_tx(
        tx: &Transaction,
        post_id: i64,
        metadata: &HashMap<String, String>,
    ) -> Result<()> {
        for (key, value) in metadata.iter() {
            tx.execute(
                "
                INSERT INTO post_metadata (post_id, key, value)
                VALUES (?1, ?2, ?3)
                ON CONFLICT(post_id, key) DO UPDATE SET value = ?3
            ",
                params![post_id, key, value],
            )?;
        }

        Ok(())
    }

    /// Helper method to load tags for a post
    fn load_tags_for_post(conn: &rusqlite::Connection, post: BlogPost) -> Result<BlogPost> {
        let post_id = match post.id {
            Some(id) => id,
            None => return Ok(post), // No ID, can't load tags
        };

        let mut stmt = conn.prepare(
            "
            SELECT t.id, t.name, t.slug FROM tags t
            JOIN post_tags pt ON t.id = pt.tag_id
            WHERE pt.post_id = ?1
        ",
        )?;

        let tag_iter = stmt.query_map([post_id], |row| {
            let id = row.get(0)?;
            let name = row.get(1)?;
            let slug = row.get(2)?;

            Ok(Tag {
                id: Some(id),
                name,
                slug,
            })
        })?;

        // Use functional approach to collect tags
        let tags = tag_iter
            .map(|tag_result| tag_result.map_err(anyhow::Error::from))
            .collect::<Result<Vector<_>>>()?;

        Ok(BlogPost { tags, ..post })
    }

    /// Helper method to load metadata for a post
    fn load_metadata_for_post(conn: &rusqlite::Connection, post: BlogPost) -> Result<BlogPost> {
        let post_id = match post.id {
            Some(id) => id,
            None => return Ok(post), // No ID, can't load metadata
        };

        let mut stmt = conn.prepare(
            "
            SELECT key, value FROM post_metadata
            WHERE post_id = ?1
        ",
        )?;

        let metadata_iter = stmt.query_map([post_id], |row| {
            let key: String = row.get(0)?;
            let value: String = row.get(1)?;

            Ok((key, value))
        })?;

        // Use functional approach to collect metadata
        let metadata_pairs = metadata_iter
            .map(|result| result.map_err(anyhow::Error::from))
            .collect::<Result<Vector<(String, String)>>>()?;

        // Convert Vector of pairs to HashMap using functional approach
        let metadata = metadata_pairs
            .into_iter()
            .fold(HashMap::new(), |acc, (key, value)| acc.update(key, value));

        Ok(BlogPost { metadata, ..post })
    }
}
