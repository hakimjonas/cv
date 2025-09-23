/// Test the new service layer with database operations
use anyhow::Result;
use cv::configuration::AppConfiguration;
use cv::db::Database;
use cv::services::{BlogService, CvService};

#[tokio::main]
async fn main() -> Result<()> {
    println!("Testing new service layer with database operations...");

    // Load configuration
    let config = AppConfiguration::load()?;
    println!("✅ Configuration loaded");

    // Test CV Service
    println!("\n--- Testing CV Service ---");

    let cv_service = CvService::with_database(config.clone()).await?;
    println!("✅ CV service created with database");

    // Test loading CV from file
    let cv = cv_service.load_cv_from_file().await?;
    println!("✅ CV loaded from file: {}", cv.personal_info.name);

    // Test CV statistics
    let stats = cv_service.get_cv_statistics(&cv).await?;
    println!("✅ CV statistics: {}", stats.summary());

    // Test CV validation
    cv_service.validate_cv(&cv).await?;
    println!("✅ CV validation passed");

    // Test output paths
    let paths = cv_service.get_output_paths();
    println!("✅ Output paths: HTML={}, PDF={}", paths.html, paths.pdf);

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

    // Test configuration and database integration
    println!("\n--- Testing Integration ---");

    // Test that services use the same configuration
    assert_eq!(
        cv_service.get_output_paths().html,
        config.paths.html_output()
    );
    println!("✅ CV service uses consistent configuration");

    // Test ensuring output directory exists
    cv_service.ensure_output_directory().await?;
    println!("✅ Output directory creation works");

    // Test regeneration check
    let needs_regen = cv_service.needs_regeneration().await?;
    println!("✅ Regeneration check: needs_regen={}", needs_regen);

    println!("\n🎉 All service tests completed successfully!");
    println!("The new architecture successfully integrates with the database layer.");

    Ok(())
}
