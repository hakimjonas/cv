/// Optimized database queries for the blog repository
///
/// This module provides optimized query implementations for fetching blog posts
/// with tags and metadata, reducing the number of database roundtrips and
/// improving performance.
use anyhow::Result;
use im::{HashMap, Vector};
use rusqlite::Connection;
use std::collections::BTreeMap;
use tracing::{debug, instrument};

use super::repository::{BlogPost, Tag};

/// Type alias for the map used to process blog posts with tags and metadata
type PostsMap = BTreeMap<i64, (PostData, Vector<TagData>, HashMap<String, String>)>;

/// Fetches all blog posts with their tags and metadata in a single query
///
/// This function uses a single SQL query with JOINs to fetch all posts, tags, and metadata
/// in one database roundtrip. It then processes the results to construct the BlogPost objects.
///
/// ## Performance Optimization
///
/// This function solves the N+1 query problem that would occur with a naive approach:
/// 1. Without optimization: Fetch all posts (1 query) + fetch tags for each post (N queries) +
///    fetch metadata for each post (N queries) = 2N+1 queries
/// 2. With optimization: Fetch all posts with tags and metadata (1 query)
///
/// ## Algorithm
///
/// The algorithm works as follows:
/// 1. Execute a single SQL query with LEFT JOINs to fetch posts, tags, and metadata
/// 2. For each row in the result:
///    - Extract post data, tag data (if present), and metadata (if present)
///    - Use a map keyed by post ID to group related data
///    - For each post ID, maintain a collection of tags and metadata
/// 3. Convert the map to a Vector of BlogPost objects
///
/// This approach handles the many-to-many and one-to-many relationships efficiently,
/// even though the JOIN results in multiple rows per post (one for each tag and metadata entry).
///
/// ## Memory Considerations
///
/// The function uses a BTreeMap as an intermediate data structure to efficiently group
/// related data by post ID. This approach trades some memory usage for significant
/// performance gains by reducing database roundtrips.
///
/// # Arguments
///
/// * `conn` - A reference to a SQLite connection
///
/// # Returns
///
/// A Result containing a Vector of BlogPost objects
#[instrument(skip(conn), err)]
pub fn get_all_posts_optimized(conn: &Connection) -> Result<Vector<BlogPost>> {
    // Use a single query with LEFT JOINs to fetch posts, tags, and metadata
    let mut stmt = conn.prepare(
        "
        SELECT 
            p.id, p.title, p.slug, p.date, p.author, p.excerpt, p.content, 
            p.published, p.featured, p.image,
            t.id as tag_id, t.name as tag_name, t.slug as tag_slug,
            pm.key as meta_key, pm.value as meta_value
        FROM posts p
        LEFT JOIN post_tags pt ON p.id = pt.post_id
        LEFT JOIN tags t ON pt.tag_id = t.id
        LEFT JOIN post_metadata pm ON p.id = pm.post_id
        ORDER BY p.date DESC
        ",
    )?;

    let rows = stmt.query_map([], |row| {
        Ok((
            // Post data
            PostData {
                id: row.get(0)?,
                title: row.get(1)?,
                slug: row.get(2)?,
                date: row.get(3)?,
                author: row.get(4)?,
                excerpt: row.get(5)?,
                content: row.get(6)?,
                published: row.get(7)?,
                featured: row.get(8)?,
                image: row.get(9)?,
            },
            // Tag data (may be NULL)
            match row.get::<_, Option<i64>>(10)? {
                Some(tag_id) => Some(TagData {
                    id: tag_id,
                    name: row.get(11)?,
                    slug: row.get(12)?,
                }),
                None => None,
            },
            // Metadata (may be NULL)
            match row.get::<_, Option<String>>(13)? {
                Some(key) => Some((key, row.get::<_, String>(14)?)),
                None => None,
            },
        ))
    })?;

    // Process the rows to construct BlogPost objects
    let mut posts_map: BTreeMap<i64, (PostData, Vector<TagData>, HashMap<String, String>)> =
        BTreeMap::new();

    for row_result in rows {
        let (post_data, tag_data_opt, metadata_opt) = row_result?;

        // Get or insert the post entry
        let entry = posts_map
            .entry(post_data.id)
            .or_insert_with(|| (post_data, Vector::new(), HashMap::new()));

        // Add tag if present
        if let Some(tag_data) = tag_data_opt {
            // Only add if not already present
            if !entry.1.iter().any(|t| t.id == tag_data.id) {
                entry.1.push_back(tag_data);
            }
        }

        // Add metadata if present
        if let Some((key, value)) = metadata_opt {
            entry.2 = entry.2.update(key, value);
        }
    }

    // Convert the map to a Vector of BlogPost objects
    let posts: Vector<BlogPost> = posts_map
        .into_iter()
        .map(|(_, (post_data, tags_data, metadata))| {
            // Convert TagData to Tag
            let tags = tags_data
                .into_iter()
                .map(|tag_data| Tag {
                    id: Some(tag_data.id),
                    name: tag_data.name,
                    slug: tag_data.slug,
                })
                .collect();

            // Create the BlogPost
            BlogPost {
                id: Some(post_data.id),
                title: post_data.title,
                slug: post_data.slug,
                date: post_data.date,
                author: post_data.author,
                excerpt: post_data.excerpt,
                content: post_data.content,
                published: post_data.published,
                featured: post_data.featured,
                image: post_data.image,
                tags,
                metadata,
            }
        })
        .collect();

    debug!("Loaded {} blog posts with optimized query", posts.len());
    Ok(posts)
}

/// Fetches a blog post by slug with its tags and metadata in a single query
///
/// This function uses a single SQL query with JOINs to fetch a post, its tags, and metadata
/// in one database roundtrip. It then processes the results to construct the BlogPost object.
///
/// ## Performance Optimization
///
/// Similar to `get_all_posts_optimized`, this function solves the N+1 query problem:
/// 1. Without optimization: Fetch post by slug (1 query) + fetch tags (1 query) +
///    fetch metadata (1 query) = 3 queries
/// 2. With optimization: Fetch post with tags and metadata (1 query)
///
/// ## Algorithm
///
/// The algorithm is similar to `get_all_posts_optimized` but optimized for a single post:
/// 1. Execute a single SQL query with LEFT JOINs to fetch the post, tags, and metadata
/// 2. For each row in the result:
///    - Extract post data (same for all rows)
///    - Extract tag data if present and add to a collection
///    - Extract metadata if present and add to a collection
/// 3. Construct a BlogPost object with the collected data
///
/// This approach is particularly efficient for retrieving a single post with all its
/// related data, which is a common operation in blog systems (e.g., viewing a single post).
///
/// # Arguments
///
/// * `conn` - A reference to a SQLite connection
/// * `slug` - The slug of the post to fetch
///
/// # Returns
///
/// A Result containing an Option with the BlogPost object if found
#[instrument(skip(conn), err)]
pub fn get_post_by_slug_optimized(conn: &Connection, slug: &str) -> Result<Option<BlogPost>> {
    // Use a single query with LEFT JOINs to fetch the post, tags, and metadata
    let mut stmt = conn.prepare(
        "
        SELECT 
            p.id, p.title, p.slug, p.date, p.author, p.excerpt, p.content, 
            p.published, p.featured, p.image,
            t.id as tag_id, t.name as tag_name, t.slug as tag_slug,
            pm.key as meta_key, pm.value as meta_value
        FROM posts p
        LEFT JOIN post_tags pt ON p.id = pt.post_id
        LEFT JOIN tags t ON pt.tag_id = t.id
        LEFT JOIN post_metadata pm ON p.id = pm.post_id
        WHERE p.slug = ?1
        ",
    )?;

    let rows = stmt.query_map([slug], |row| {
        Ok((
            // Post data
            PostData {
                id: row.get(0)?,
                title: row.get(1)?,
                slug: row.get(2)?,
                date: row.get(3)?,
                author: row.get(4)?,
                excerpt: row.get(5)?,
                content: row.get(6)?,
                published: row.get(7)?,
                featured: row.get(8)?,
                image: row.get(9)?,
            },
            // Tag data (may be NULL)
            match row.get::<_, Option<i64>>(10)? {
                Some(tag_id) => Some(TagData {
                    id: tag_id,
                    name: row.get(11)?,
                    slug: row.get(12)?,
                }),
                None => None,
            },
            // Metadata (may be NULL)
            match row.get::<_, Option<String>>(13)? {
                Some(key) => Some((key, row.get::<_, String>(14)?)),
                None => None,
            },
        ))
    })?;

    // Process the rows to construct the BlogPost object
    let mut post_data: Option<PostData> = None;
    let mut tags_data: Vector<TagData> = Vector::new();
    let mut metadata: HashMap<String, String> = HashMap::new();

    for row_result in rows {
        let (current_post_data, tag_data_opt, metadata_opt) = row_result?;

        // Set post data if not already set
        if post_data.is_none() {
            post_data = Some(current_post_data.clone());
        }

        // Add tag if present
        if let Some(tag_data) = tag_data_opt {
            // Only add if not already present
            if !tags_data.iter().any(|t| t.id == tag_data.id) {
                tags_data.push_back(tag_data);
            }
        }

        // Add metadata if present
        if let Some((key, value)) = metadata_opt {
            metadata = metadata.update(key, value);
        }
    }

    // If no post was found, return None
    let post_data = match post_data {
        Some(data) => data,
        None => return Ok(None),
    };

    // Convert TagData to Tag
    let tags = tags_data
        .into_iter()
        .map(|tag_data| Tag {
            id: Some(tag_data.id),
            name: tag_data.name,
            slug: tag_data.slug,
        })
        .collect();

    // Create the BlogPost
    let post = BlogPost {
        id: Some(post_data.id),
        title: post_data.title,
        slug: post_data.slug,
        date: post_data.date,
        author: post_data.author,
        excerpt: post_data.excerpt,
        content: post_data.content,
        published: post_data.published,
        featured: post_data.featured,
        image: post_data.image,
        tags,
        metadata,
    };

    debug!(
        "Loaded blog post with slug '{}' using optimized query",
        slug
    );
    Ok(Some(post))
}

/// Fetches all published posts with their tags and metadata in a single query
///
/// This function uses a single SQL query with JOINs to fetch all published posts, tags, and metadata
/// in one database roundtrip. It then processes the results to construct the BlogPost objects.
///
/// ## Performance Optimization
///
/// This function uses the same optimization approach as `get_all_posts_optimized` but with
/// an additional WHERE clause to filter for published posts only. It solves the N+1 query problem:
/// 1. Without optimization: Fetch published posts (1 query) + fetch tags for each post (N queries) +
///    fetch metadata for each post (N queries) = 2N+1 queries
/// 2. With optimization: Fetch published posts with tags and metadata (1 query)
///
/// ## Algorithm
///
/// The algorithm is identical to `get_all_posts_optimized`:
/// 1. Execute a single SQL query with LEFT JOINs and a WHERE clause to fetch published posts, tags, and metadata
/// 2. For each row in the result:
///    - Extract post data, tag data (if present), and metadata (if present)
///    - Use a map keyed by post ID to group related data
///    - For each post ID, maintain a collection of tags and metadata
/// 3. Convert the map to a Vector of BlogPost objects
///
/// This approach is commonly used for displaying blog post listings on the public-facing
/// website, where only published posts should be shown.
///
/// # Arguments
///
/// * `conn` - A reference to a SQLite connection
///
/// # Returns
///
/// A Result containing a Vector of BlogPost objects
#[instrument(skip(conn), err)]
pub fn get_published_posts_optimized(conn: &Connection) -> Result<Vector<BlogPost>> {
    // Use a single query with LEFT JOINs to fetch posts, tags, and metadata
    let mut stmt = conn.prepare(
        "
        SELECT 
            p.id, p.title, p.slug, p.date, p.author, p.excerpt, p.content, 
            p.published, p.featured, p.image,
            t.id as tag_id, t.name as tag_name, t.slug as tag_slug,
            pm.key as meta_key, pm.value as meta_value
        FROM posts p
        LEFT JOIN post_tags pt ON p.id = pt.post_id
        LEFT JOIN tags t ON pt.tag_id = t.id
        LEFT JOIN post_metadata pm ON p.id = pm.post_id
        WHERE p.published = 1
        ORDER BY p.date DESC
        ",
    )?;

    let rows = stmt.query_map([], |row| {
        Ok((
            // Post data
            PostData {
                id: row.get(0)?,
                title: row.get(1)?,
                slug: row.get(2)?,
                date: row.get(3)?,
                author: row.get(4)?,
                excerpt: row.get(5)?,
                content: row.get(6)?,
                published: row.get(7)?,
                featured: row.get(8)?,
                image: row.get(9)?,
            },
            // Tag data (may be NULL)
            match row.get::<_, Option<i64>>(10)? {
                Some(tag_id) => Some(TagData {
                    id: tag_id,
                    name: row.get(11)?,
                    slug: row.get(12)?,
                }),
                None => None,
            },
            // Metadata (may be NULL)
            match row.get::<_, Option<String>>(13)? {
                Some(key) => Some((key, row.get::<_, String>(14)?)),
                None => None,
            },
        ))
    })?;

    // Process the rows to construct BlogPost objects
    let mut posts_map: BTreeMap<i64, (PostData, Vector<TagData>, HashMap<String, String>)> =
        BTreeMap::new();

    for row_result in rows {
        let (post_data, tag_data_opt, metadata_opt) = row_result?;

        // Get or insert the post entry
        let entry = posts_map
            .entry(post_data.id)
            .or_insert_with(|| (post_data, Vector::new(), HashMap::new()));

        // Add tag if present
        if let Some(tag_data) = tag_data_opt {
            // Only add if not already present
            if !entry.1.iter().any(|t| t.id == tag_data.id) {
                entry.1.push_back(tag_data);
            }
        }

        // Add metadata if present
        if let Some((key, value)) = metadata_opt {
            entry.2 = entry.2.update(key, value);
        }
    }

    // Convert the map to a Vector of BlogPost objects
    let posts: Vector<BlogPost> = posts_map
        .into_iter()
        .map(|(_, (post_data, tags_data, metadata))| {
            // Convert TagData to Tag
            let tags = tags_data
                .into_iter()
                .map(|tag_data| Tag {
                    id: Some(tag_data.id),
                    name: tag_data.name,
                    slug: tag_data.slug,
                })
                .collect();

            // Create the BlogPost
            BlogPost {
                id: Some(post_data.id),
                title: post_data.title,
                slug: post_data.slug,
                date: post_data.date,
                author: post_data.author,
                excerpt: post_data.excerpt,
                content: post_data.content,
                published: post_data.published,
                featured: post_data.featured,
                image: post_data.image,
                tags,
                metadata,
            }
        })
        .collect();

    debug!(
        "Loaded {} published blog posts with optimized query",
        posts.len()
    );
    Ok(posts)
}

/// Helper struct to store post data from a database row
///
/// This struct is used as an intermediate representation when processing query results
/// from the optimized JOIN queries. It contains only the post fields without any related
/// data (tags or metadata), which are processed separately and combined later.
///
/// The struct mirrors the columns in the posts table and is used to efficiently extract
/// post data from each row of the query result before aggregating related data.
#[derive(Debug, Clone)]
struct PostData {
    /// Primary key of the post
    id: i64,
    /// Title of the post
    title: String,
    /// URL-friendly version of the title (unique)
    slug: String,
    /// Publication date in ISO format
    date: String,
    /// Author of the post
    author: String,
    /// Short summary of the post
    excerpt: String,
    /// Full content of the post
    content: String,
    /// Whether the post is published (true) or draft (false)
    published: bool,
    /// Whether the post is featured (true) or not (false)
    featured: bool,
    /// Optional URL or path to a featured image
    image: Option<String>,
}

/// Helper struct to store tag data from a database row
///
/// This struct is used as an intermediate representation when processing query results
/// from the optimized JOIN queries. It contains only the tag fields, which are extracted
/// from each row of the query result that contains tag data.
///
/// The struct mirrors the columns in the tags table and is used to efficiently extract
/// tag data before aggregating it with the corresponding post.
#[derive(Debug, Clone)]
struct TagData {
    /// Primary key of the tag
    id: i64,
    /// Display name of the tag
    name: String,
    /// URL-friendly version of the name (unique)
    slug: String,
}
