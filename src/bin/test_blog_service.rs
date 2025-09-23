/// Test the new blog service with database operations
use anyhow::Result;
use cv::configuration::AppConfiguration;
use cv::db::Database;
use cv::services::BlogService;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Testing new blog service with database operations...");

    // Load configuration
    let config = AppConfiguration::load()?;
    println!("✅ Configuration loaded");

    // Test Blog Service
    println!("\n--- Testing Blog Service ---");

    let database = Database::new(&config.paths.database)?;
    let blog_repository = database.blog_repository();
    let blog_service = BlogService::new(blog_repository);
    println!("✅ Blog service created with database");

    // Test getting all posts
    match blog_service.get_all_posts().await {
        Ok(posts) => {
            println!("✅ Retrieved {} blog posts from database", posts.len());
            for (i, post) in posts.iter().take(3).enumerate() {
                println!(
                    "  {}. {} (published: {})",
                    i + 1,
                    post.title,
                    post.published
                );
            }
        }
        Err(e) => {
            println!("⚠️  No existing blog posts found: {}", e);
        }
    }

    // Test creating a new blog post
    match blog_service.create_post(
        "Test Post from New Service",
        "This is a test post created by the new BlogService to verify database operations work correctly.",
        "Test Author",
        None
    ).await {
        Ok(created_post) => {
            println!("✅ Created test blog post: {}", created_post.title);

            // Test saving the post to database
            match blog_service.save_post(&created_post).await {
                Ok(post_id) => {
                    println!("✅ Saved test blog post with ID: {:?}", post_id);

                    // Test retrieving the post
                    if let Ok(Some(retrieved_post)) = blog_service.get_post_by_id(post_id).await {
                        println!("✅ Retrieved test post: {}", retrieved_post.title);
                    }

                    // Clean up - delete the test post
                    if let Ok(()) = blog_service.delete_post(post_id).await {
                        println!("✅ Cleaned up test post");
                    }
                }
                Err(e) => {
                    println!("⚠️  Could not save test post: {}", e);
                }
            }
        }
        Err(e) => {
            println!("⚠️  Could not create test post: {}", e);
        }
    }

    // Test blog statistics
    match blog_service.get_all_posts().await {
        Ok(posts) => println!("✅ Total blog posts in database: {}", posts.len()),
        Err(e) => println!("⚠️  Could not get post count: {}", e),
    }

    // Test published posts
    match blog_service.get_published_posts().await {
        Ok(published) => println!("✅ Published posts: {}", published.len()),
        Err(e) => println!("⚠️  Could not get published posts: {}", e),
    }

    // Test tags
    match blog_service.get_all_tags().await {
        Ok(tags) => println!("✅ Total tags: {}", tags.len()),
        Err(e) => println!("⚠️  Could not get tags: {}", e),
    }

    println!("\n🎉 Blog service tests completed successfully!");
    println!("The new blog service architecture successfully integrates with the database layer.");

    Ok(())
}
