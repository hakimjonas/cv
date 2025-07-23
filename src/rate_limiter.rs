use std::collections::HashMap;
/// Rate limiting middleware for the blog API
///
/// This module provides a configurable rate limiting middleware for the blog API
/// to prevent abuse and ensure fair usage of the API.
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;

use axum::extract::ConnectInfo;
use axum::http::{Request, Response, StatusCode};
use axum::middleware::Next;
use axum::response::IntoResponse;
use tokio::sync::Mutex;
use tokio::time::Instant;
use tracing::warn;

/// Configuration for the rate limiter
#[derive(Debug, Clone)]
pub struct RateLimiterConfig {
    /// Maximum number of requests per window
    pub max_requests: u64,
    /// Time window in seconds
    pub window_seconds: u64,
    /// Whether to include the rate limit headers in the response
    pub include_headers: bool,
    /// Status code to return when rate limit is exceeded
    pub status_code: StatusCode,
}

impl Default for RateLimiterConfig {
    fn default() -> Self {
        Self {
            max_requests: 100,
            window_seconds: 60,
            include_headers: true,
            status_code: StatusCode::TOO_MANY_REQUESTS,
        }
    }
}

/// Rate limiter state for tracking requests by IP address
#[derive(Debug, Default)]
pub struct RateLimiterState {
    /// Map of IP addresses to their request counts and timestamps
    requests: Mutex<HashMap<IpAddr, Vec<Instant>>>,
}

impl RateLimiterState {
    /// Create a new rate limiter state
    pub fn new() -> Self {
        Self {
            requests: Mutex::new(HashMap::new()),
        }
    }

    /// Check if a request from the given IP address is allowed
    /// Returns the number of remaining requests in the current window
    pub async fn check_rate_limit(
        &self,
        ip: IpAddr,
        config: &RateLimiterConfig,
    ) -> (bool, u64, u64) {
        let now = Instant::now();
        let window = Duration::from_secs(config.window_seconds);
        let max_requests = config.max_requests;

        let mut requests = self.requests.lock().await;

        // Get or create the request timestamps for this IP
        let timestamps = requests.entry(ip).or_insert_with(Vec::new);

        // Remove timestamps that are outside the current window
        timestamps.retain(|&timestamp| now.duration_since(timestamp) < window);

        // Count the number of requests in the current window
        let request_count = timestamps.len() as u64;

        // Check if the request is allowed
        let is_allowed = request_count < max_requests;

        // If allowed, add the current timestamp
        if is_allowed {
            timestamps.push(now);
        }

        // Calculate remaining requests
        let remaining = if is_allowed {
            max_requests - request_count - 1
        } else {
            0
        };

        // Calculate reset time in seconds
        let reset = if timestamps.is_empty() {
            config.window_seconds
        } else {
            let oldest = timestamps[0];
            let elapsed = now.duration_since(oldest).as_secs();
            config.window_seconds.saturating_sub(elapsed)
        };

        (is_allowed, remaining, reset)
    }
}

/// Rate limiter middleware for Axum
pub async fn rate_limiter(
    ConnectInfo(addr): ConnectInfo<std::net::SocketAddr>,
    state: Arc<RateLimiterState>,
    config: Arc<RateLimiterConfig>,
    request: Request<axum::body::Body>,
    next: Next,
) -> impl IntoResponse {
    let ip = addr.ip();

    // Check if the request is allowed
    let (is_allowed, remaining, reset) = state.check_rate_limit(ip, &config).await;

    if is_allowed {
        // Process the request
        let mut response = next.run(request).await;

        // Add rate limit headers if configured
        if config.include_headers {
            let headers = response.headers_mut();
            headers.insert("X-RateLimit-Limit", config.max_requests.into());
            headers.insert("X-RateLimit-Remaining", remaining.into());
            headers.insert("X-RateLimit-Reset", reset.into());
        }

        response
    } else {
        // Log rate limit exceeded
        warn!("Rate limit exceeded for IP: {}", ip);

        // Return rate limit exceeded response
        let mut response = Response::builder()
            .status(config.status_code)
            .body(axum::body::Body::empty())
            .unwrap();

        // Add rate limit headers if configured
        if config.include_headers {
            let headers = response.headers_mut();
            headers.insert("X-RateLimit-Limit", config.max_requests.into());
            headers.insert("X-RateLimit-Remaining", remaining.into());
            headers.insert("X-RateLimit-Reset", reset.into());
        }

        response
    }
}

/// Create a rate limiter middleware with the given configuration
///
/// NOTE: This function is temporarily commented out due to type compatibility issues
/// as part of the issue resolution process. It will be properly implemented in a future update.
pub fn create_rate_limiter_layer(_config: RateLimiterConfig) -> ((), Arc<RateLimiterState>) {
    // Create a state that can be returned
    let state = Arc::new(RateLimiterState::new());

    // Return a placeholder tuple with an empty value for the layer
    // This is a temporary solution to get the code to compile
    ((), state)
}
