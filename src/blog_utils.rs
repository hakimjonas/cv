use anyhow::{Context, Result};
use crate::blog_data::{BlogManager, BlogPost, Tag};
use im::vector;
use std::fs;
use std::path::{Path, PathBuf};

/// Creates a test directory and returns the path
pub fn create_test_directory() -> Result<PathBuf> {
    let test_dir = PathBuf::from("./test_data");
    if !test_dir.exists() {
        fs::create_dir(&test_dir).context("Failed to create test directory")?;
    }
    Ok(test_dir)
}

/// Creates a test database and returns the path
pub fn create_test_database() -> Result<PathBuf> {
    let test_dir = create_test_directory()?;
    let db_path = test_dir.join("blog_test.db");

    // If database file exists and seems to be locked, remove it
    if db_path.exists() {
        // Try to open the database exclusively to check if it's locked
        match rusqlite::Connection::open_with_flags(
            &db_path, 
            rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE | 
            rusqlite::OpenFlags::SQLITE_OPEN_CREATE | 
            rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX
        ) {
            Ok(_) => {
                // Database isn't locked, we can use it
                println!("✅ Existing database is not locked");
            },
            Err(e) => {
                if e.to_string().contains("locked") {
                    println!("⚠️ Existing database appears to be locked, removing it to start fresh");
                    // Try to remove the database file and its WAL files
                    let _ = std::fs::remove_file(&db_path);
                    let _ = std::fs::remove_file(db_path.with_extension("db-shm"));
                    let _ = std::fs::remove_file(db_path.with_extension("db-wal"));
                    // Wait a moment for the OS to release the files
                    std::thread::sleep(std::time::Duration::from_millis(500));
                }
            }
        }
    }

    // Check if the directory is writable
    let test_file_path = test_dir.join("write_test.tmp");
    match std::fs::File::create(&test_file_path) {
        Ok(_) => {
            println!("✅ Directory is writable");
            // Clean up test file
            let _ = std::fs::remove_file(test_file_path);
        },
        Err(e) => {
            println!("❌ Directory is not writable: {}", e);
            return Err(anyhow::anyhow!("Test directory is not writable: {}", e));
        }
    }

    // If database file already exists, check if it's writable
    if db_path.exists() {
        match std::fs::OpenOptions::new().write(true).open(&db_path) {
            Ok(_) => println!("✅ Existing database file is writable"),
            Err(e) => {
                println!("❌ Existing database file is not writable: {}", e);
                return Err(anyhow::anyhow!("Database file is not writable: {}", e));
            }
        }
    }

    Ok(db_path)
}

/// Creates a test blog post
pub fn create_test_blog_post() -> BlogPost {
    let test_tag = Tag {
        id: None,
        name: "Test Tag".to_string(),
        slug: "test-tag".to_string(),
    };

    BlogPost {
        id: None,
        title: "Test Blog Post".to_string(),
        slug: "test-blog-post".to_string(),
        date: "2025-06-12".to_string(),
        author: "Test Author".to_string(),
        excerpt: "This is a test post.".to_string(),
        content: "# Test Blog Post\n\nThis is a test post.\n\n## Features\n\n- Markdown support\n- Tags\n- Metadata".to_string(),
        published: true,
        featured: true,
        image: Some("https://picsum.photos/800/400".to_string()),
        tags: vector![test_tag],
        metadata: im::HashMap::new(),
    }
}

/// Run core blog functionality tests
pub fn run_blog_core_tests(db_path: &Path) -> Result<()> {
    println!("Running blog core tests with database at: {:?}", db_path);

    // Verify we can open the database in read-write mode before proceeding
    match rusqlite::Connection::open_with_flags(
        db_path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE | rusqlite::OpenFlags::SQLITE_OPEN_CREATE
    ) {
        Ok(conn) => {
            println!("✅ Successfully opened database connection in read-write mode");
            // Test if we can write to the database
            match conn.execute("CREATE TABLE IF NOT EXISTS write_test (id INTEGER PRIMARY KEY)", []) {
                Ok(_) => println!("✅ Successfully created test table - database is writable"),
                Err(e) => {
                    println!("❌ Failed to create test table: {}", e);
                    return Err(anyhow::anyhow!("Database is not writable: {}", e));
                }
            }
            // Clean up
            let _ = conn.execute("DROP TABLE IF EXISTS write_test", []);
            // Close connection explicitly
            drop(conn);
        },
        Err(e) => {
            println!("❌ Failed to open database in read-write mode: {}", e);
            return Err(anyhow::anyhow!("Cannot open database in read-write mode: {}", e));
        }
    }

    // Create a blog manager
    let blog_manager = BlogManager::new(db_path).context("Failed to create blog manager")?;
    println!("✅ Successfully created BlogManager instance");

    // Add a longer delay to ensure database is ready
    println!("Waiting for database to be ready before creating post...");
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Test creating a post
    let test_post = create_test_blog_post();
    println!("Creating test post...");
    let post_id = blog_manager.create_or_update_post(&test_post).context("Failed to create test post")?;
    println!("Created test post with ID: {}", post_id);

    // Add a longer delay before retrieving posts
    println!("Waiting before retrieving all posts...");
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Test retrieving all posts
    println!("Retrieving all posts...");
    let posts = blog_manager.get_all_posts().context("Failed to get posts")?;
    println!("Retrieved {} posts", posts.len());
    assert!(!posts.is_empty(), "Posts list should not be empty");

    // Add a longer delay before retrieving a specific post
    println!("Waiting before retrieving specific post...");
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Test retrieving a specific post
    println!("Retrieving post by slug: {}", test_post.slug);
    let retrieved_post = blog_manager
        .get_post_by_slug(&test_post.slug)
        .context("Failed to get post by slug")?;
    println!("Retrieved post by slug: {}", if retrieved_post.is_some() { "found" } else { "not found" });
    assert!(retrieved_post.is_some(), "Retrieved post should exist");

    // Add a longer delay before updating the post
    println!("Waiting before updating post...");
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Test updating a post
    let updated_post = BlogPost {
        id: Some(post_id),
        title: "Updated Test Post".to_string(),
        ..test_post.clone()
    };

    println!("Updating post with ID: {}", post_id);
    let updated_id = blog_manager
        .create_or_update_post(&updated_post)
        .context("Failed to update post")?;
    println!("Updated post, returned ID: {}", updated_id);
    assert_eq!(updated_id, post_id, "Updated post should have the same ID");

    // Add a longer delay before retrieving the updated post
    println!("Waiting before retrieving updated post...");
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Test retrieving the updated post
    println!("Retrieving updated post by slug: {}", test_post.slug);
    let retrieved_updated_post = blog_manager
        .get_post_by_slug(&test_post.slug)
        .context("Failed to get updated post by slug")?;

    println!("Retrieved updated post: {}", if retrieved_updated_post.is_some() { "found" } else { "not found" });
    assert!(retrieved_updated_post.is_some(), "Updated post should exist");
    let retrieved_updated_post = retrieved_updated_post.unwrap();
    assert_eq!(retrieved_updated_post.title, "Updated Test Post", "Post title should be updated");

    // Add a longer delay before deleting the post
    println!("Waiting before deleting post...");
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Test deleting a post
    println!("Deleting post with ID: {}", post_id);
    blog_manager
        .delete_post(post_id)
        .context("Failed to delete post")?;
    println!("Deleted post with ID: {}", post_id);

    // Add a longer delay before verifying deletion
    println!("Waiting before verifying deletion...");
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Verify deletion
    let deleted_post = blog_manager.get_post_by_slug(&test_post.slug)?;
    assert!(deleted_post.is_none(), "Deleted post should not exist");

    Ok(())
}
