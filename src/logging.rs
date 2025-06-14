use std::path::Path;
use tracing::Level;
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    prelude::*,
    EnvFilter,
};
use tracing_appender::{self, non_blocking, rolling};
use tracing_log::LogTracer;

/// Logging configuration options
pub struct LoggingConfig {
    /// The name of the application
    pub app_name: String,
    /// The minimum log level to display
    pub level: Level,
    /// Whether to log to a file
    pub log_to_file: bool,
    /// The directory to store log files in
    pub log_dir: Option<String>,
    /// Whether to use JSON formatting for logs
    pub json_format: bool,
    /// Whether to log spans (entry/exit of functions)
    pub log_spans: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            app_name: "cv".to_string(),
            level: Level::INFO,
            log_to_file: false,
            log_dir: None,
            json_format: false,
            log_spans: false,
        }
    }
}

/// Initialize logging with the given configuration
///
/// # Arguments
///
/// * `config` - The logging configuration
///
/// # Returns
///
/// A guard that must be kept alive for the duration of the program
pub fn init_logging(config: LoggingConfig) -> Option<non_blocking::WorkerGuard> {
    // Initialize LogTracer to convert log crate records to tracing events
    // Ignore errors if it's already been initialized
    let _ = LogTracer::init();

    // Create a filter based on the configured level
    let filter = EnvFilter::from_default_env()
        .add_directive(format!("{}={}", config.app_name, config.level).parse().unwrap())
        .add_directive("tower_http=info".parse().unwrap())
        .add_directive("axum=info".parse().unwrap());

    // Determine if we should log spans
    let span_events = if config.log_spans {
        FmtSpan::ACTIVE
    } else {
        FmtSpan::NONE
    };

    // Set up the subscriber
    let subscriber = tracing_subscriber::registry().with(filter);

    // Configure file logging if enabled
    if config.log_to_file {
        if let Some(log_dir) = config.log_dir.as_ref() {
            // Create the log directory if it doesn't exist
            std::fs::create_dir_all(log_dir).ok();

            // Set up a rolling file appender
            let file_appender = rolling::daily(log_dir, format!("{}.log", config.app_name));
            let (file_writer, guard) = non_blocking(file_appender);

            // Configure the file layer
            let file_layer = fmt::Layer::new()
                .with_writer(file_writer)
                .with_span_events(span_events.clone())
                .with_ansi(false);

            // Add JSON formatting if configured
            let file_layer = if config.json_format {
                file_layer.json().boxed()
            } else {
                file_layer.boxed()
            };

            // Set up the console layer
            let console_layer = fmt::Layer::new()
                .with_writer(std::io::stdout)
                .with_span_events(span_events);

            // Try to register both layers, but don't panic if it fails
            // This allows multiple binaries to initialize logging
            match subscriber
                .with(file_layer)
                .with(console_layer)
                .try_init() {
                    Ok(_) => return Some(guard),
                    Err(_) => {
                        // Subscriber already set, just return the guard to keep the file writer alive
                        return Some(guard);
                    }
                }
        }
    }

    // Set up console-only logging
    let console_layer = fmt::Layer::new()
        .with_writer(std::io::stdout)
        .with_span_events(span_events);

    // Add JSON formatting if configured
    let console_layer = if config.json_format {
        console_layer.json().boxed()
    } else {
        console_layer.boxed()
    };

    // Try to register the console layer, but don't panic if it fails
    // This allows multiple binaries to initialize logging
    let _ = subscriber
        .with(console_layer)
        .try_init();

    None
}

/// Initialize logging with default configuration
///
/// # Returns
///
/// A guard that must be kept alive for the duration of the program
pub fn init_default_logging() -> Option<non_blocking::WorkerGuard> {
    init_logging(LoggingConfig::default())
}

/// Initialize logging with file output
///
/// # Arguments
///
/// * `app_name` - The name of the application
/// * `log_dir` - The directory to store log files in
///
/// # Returns
///
/// A guard that must be kept alive for the duration of the program
pub fn init_file_logging(app_name: &str, log_dir: &Path) -> Option<non_blocking::WorkerGuard> {
    let config = LoggingConfig {
        app_name: app_name.to_string(),
        log_to_file: true,
        log_dir: Some(log_dir.to_string_lossy().to_string()),
        ..Default::default()
    };

    init_logging(config)
}
