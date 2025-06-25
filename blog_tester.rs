use anyhow::{Context, Result};
use cv::blog_utils::{create_test_database, run_blog_core_tests};
use cv::check_db_permissions::check_db_permissions;

fn main() -> Result<()> {
    println!("Blog Tester");
    println!("==========\n");

    // Create database path
    let db_path = create_test_database()?;
    println!("Using database at: {:?}\n", db_path);

    // Explicitly check database permissions before running tests
    check_db_permissions(&db_path).context("Failed to verify database permissions")?;
    println!("Database permissions check passed\n");

    // Run core blog functionality tests
    run_blog_core_tests(&db_path).context("Blog core tests failed")?;

    println!("\nAll blog tests passed successfully!");

    Ok(())
}
