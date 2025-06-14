use anyhow::{Context, Result};
use cv::blog_api::create_blog_api_router;
use cv::blog_utils::create_test_database;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use std::sync::Once;

// Initialize once to ensure we only set up global state once
static INIT: Once = Once::new();

// Configure SQLite for better concurrency
fn configure_sqlite() {
    INIT.call_once(|| {
        println!("Initializing SQLite global configuration...");
        // Set global SQLite configuration for better concurrent access
        if let Err(e) = rusqlite::Connection::open_in_memory().and_then(|conn| {
            conn.execute_batch("
                PRAGMA journal_mode = WAL;
                PRAGMA synchronous = NORMAL;
                PRAGMA busy_timeout = 120000;
                PRAGMA temp_store = MEMORY;
                PRAGMA cache_size = 10000;
                PRAGMA locking_mode = NORMAL;
                PRAGMA mmap_size = 30000000;
                PRAGMA page_size = 4096;
                PRAGMA max_page_count = 2147483646;
            ")
        }) {
            println!("Warning: Failed to set global SQLite configuration: {}", e);
        }
        println!("Configuring SQLite for better concurrency");
    });
}

#[tokio::main]
async fn main() -> Result<()> {
    // Configure SQLite for better concurrency
    configure_sqlite();

    // Create a test database
    let db_path = create_test_database()?;
    println!("Using database at: {:?}\n", db_path);

    // Create the API router
    let app = create_blog_api_router(db_path)?;

    // Try a range of ports starting from 3000
    let mut port = 3000;
    let max_port = 3010; // Try up to port 3010
    let mut listener = None;

    println!("Attempting to start server on ports {}-{}", port, max_port);

    while port <= max_port {
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        match TcpListener::bind(addr).await {
            Ok(l) => {
                println!("Blog API server running at http://{}", addr);
                listener = Some((l, addr));
                break;
            },
            Err(e) => {
                if e.kind() == std::io::ErrorKind::AddrInUse {
                    println!("Port {} is already in use, trying next port...", port);
                    port += 1;
                } else {
                    return Err(e.into());
                }
            }
        }
    }

    // If we didn't find an available port
    if listener.is_none() {
        return Err(anyhow::anyhow!("Could not find an available port between {} and {}", 3000, max_port));
    }

    let (listener, _addr) = listener.unwrap();

    axum::serve(listener, app).await.context("Server error")?;

    Ok(())
}
