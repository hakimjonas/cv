use anyhow::{Context, Result};
use cv::blog_utils::{create_test_database, run_blog_core_tests};

fn main() -> Result<()> {
    println!("Blog Core Tests");
    println!("===============\n");

    // Create database path
    let db_path = create_test_database()?;
    println!("Using database at: {:?}\n", db_path);

    // Run core blog functionality tests
    run_blog_core_tests(&db_path).context("Blog core tests failed")?;

    println!("\nAll blog core tests passed successfully!");

    Ok(())
}
