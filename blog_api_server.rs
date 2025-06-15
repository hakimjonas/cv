use anyhow::{Context, Result};
use cv::blog_api::create_blog_api_router;
use cv::blog_utils::create_test_database;
use std::net::SocketAddr;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<()> {
    // Create a test database
    let db_path = create_test_database()?;
    println!("Using database at: {:?}\n", db_path);

    // Create the API router
    let app = create_blog_api_router(db_path)?;

    // Note: Static file serving is already configured in create_blog_api_router

    // Start the server
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let listener = TcpListener::bind(addr).await?;
    println!("Blog API server running at http://localhost:3000");

    axum::serve(listener, app).await.context("Server error")?;

    Ok(())
}
