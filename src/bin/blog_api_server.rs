use anyhow::{Context, Result};
use cv::blog_api::create_blog_api_router;
use cv::blog_utils::create_test_database;
use cv::logging;
use cv::unified_config::AppConfig;
use std::net::SocketAddr;
use std::sync::Once;
use tokio::net::TcpListener;
use tracing::{debug, error, info, warn};

// Initialize once to ensure we only set up global state once
static INIT: Once = Once::new();

// Initialize logging with tracing
fn init_logging() {
    // Set up a logging configuration for the blog API server
    let config = logging::LoggingConfig {
        app_name: "blog_api_server".to_string(),
        level: tracing::Level::INFO,
        log_spans: true,
        ..Default::default()
    };

    // Initialize logging with the configuration
    let _guard = logging::init_logging(config);
    info!("Logging initialized with tracing");
}

// Configure SQLite for better concurrency
fn configure_sqlite() {
    INIT.call_once(|| {
        info!("Initializing SQLite global configuration...");
        // Set global SQLite configuration for better concurrent access
        if let Err(e) = rusqlite::Connection::open_in_memory().and_then(|conn| {
            conn.execute_batch(
                "
                PRAGMA journal_mode = WAL;
                PRAGMA synchronous = NORMAL;
                PRAGMA busy_timeout = 120000;
                PRAGMA temp_store = MEMORY;
                PRAGMA cache_size = 10000;
                PRAGMA locking_mode = NORMAL;
                PRAGMA mmap_size = 30000000;
                PRAGMA page_size = 4096;
                PRAGMA max_page_count = 2147483646;
            ",
            )
        }) {
            warn!("Failed to set global SQLite configuration: {}", e);
        }
        info!("Configuring SQLite for better concurrency");
    });
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    init_logging();

    // Configure SQLite for better concurrency
    configure_sqlite();

    // Load configuration from all available sources
    let config = AppConfig::load().context("Failed to load configuration")?;
    debug!("Loaded configuration: {:?}", config);

    // Create a test database
    let db_path = create_test_database()?;
    info!("Using database at: {:?}", db_path);

    // Create the API router
    let app = create_blog_api_router(db_path)?;

    // Try a range of ports starting from the configured port
    let mut port = config.api_port;
    let max_port = config.api_max_port;
    let mut listener = None;

    info!("Attempting to start server on ports {}-{}", port, max_port);

    while port <= max_port {
        let addr = SocketAddr::from(([0, 0, 0, 0], port));
        match TcpListener::bind(addr).await {
            Ok(l) => {
                info!("Blog API server running at http://{}", addr);
                listener = Some((l, addr));
                break;
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::AddrInUse {
                    info!("Port {} is already in use, trying next port...", port);
                    port += 1;
                } else {
                    error!("Failed to bind to port {}: {}", port, e);
                    return Err(e.into());
                }
            }
        }
    }

    // If we didn't find an available port
    if listener.is_none() {
        let err_msg = format!(
            "Could not find an available port between {} and {}",
            config.api_port, max_port
        );
        error!("{}", err_msg);
        return Err(anyhow::anyhow!(err_msg));
    }

    let (listener, addr) = listener.unwrap();
    info!("Starting server on http://{}", addr);

    axum::serve(listener, app).await.context("Server error")?;

    Ok(())
}
