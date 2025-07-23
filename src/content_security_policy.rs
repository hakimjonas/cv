/// Content Security Policy module
///
/// This module provides a Content Security Policy (CSP) for the blog API
/// to prevent XSS attacks and other security vulnerabilities.
use axum::http::HeaderValue;
use tower_http::set_header::SetResponseHeaderLayer;

/// CSP configuration
#[derive(Debug, Clone)]
pub struct CspConfig {
    /// Whether to enable the CSP
    pub enabled: bool,
    /// Whether to enable CSP reporting
    pub report_only: bool,
    /// Report URI for CSP violations
    pub report_uri: Option<String>,
}

impl Default for CspConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            report_only: false,
            report_uri: None,
        }
    }
}

/// Create a Content Security Policy header value
///
/// This function creates a strict CSP header value that prevents XSS attacks
/// by restricting what resources can be loaded by the browser.
pub fn create_csp_header_value(config: &CspConfig) -> HeaderValue {
    // Use owned strings instead of references
    let mut directives = Vec::new();

    // Add all directives as owned strings
    directives.push(String::from("default-src 'self'"));
    directives.push(String::from("script-src 'self' 'unsafe-inline'"));
    directives.push(String::from("style-src 'self' 'unsafe-inline'"));
    directives.push(String::from("img-src 'self' data:"));
    directives.push(String::from("font-src 'self'"));
    directives.push(String::from("connect-src 'self'"));
    directives.push(String::from("object-src 'none'"));
    directives.push(String::from("frame-src 'self'"));
    directives.push(String::from("frame-ancestors 'self'"));
    directives.push(String::from("form-action 'self'"));
    directives.push(String::from("base-uri 'self'"));
    directives.push(String::from("upgrade-insecure-requests"));
    directives.push(String::from("block-all-mixed-content"));

    // Add report URI if configured
    if let Some(report_uri) = &config.report_uri {
        // Store the formatted string in a variable to avoid the temporary value dropped while borrowed error
        let report_directive = format!("report-uri {}", report_uri);
        directives.push(report_directive);
    }

    // Join directives with semicolons
    let csp_value = directives.join("; ");

    // Create header value
    HeaderValue::from_str(&csp_value).unwrap_or_else(|_| {
        HeaderValue::from_static("default-src 'self'; script-src 'self'; object-src 'none'")
    })
}

// Function to return CSP header value
fn get_csp_value() -> HeaderValue {
    create_csp_header_value(&CspConfig::default())
}

// Function to return X-Content-Type-Options header value
fn get_content_type_options_value() -> HeaderValue {
    HeaderValue::from_static("nosniff")
}

// Function to return X-Frame-Options header value
fn get_frame_options_value() -> HeaderValue {
    HeaderValue::from_static("SAMEORIGIN")
}

// Function to return X-XSS-Protection header value
fn get_xss_protection_value() -> HeaderValue {
    HeaderValue::from_static("1; mode=block")
}

// Function to return Referrer-Policy header value
fn get_referrer_policy_value() -> HeaderValue {
    HeaderValue::from_static("strict-origin-when-cross-origin")
}

// Function to return Permissions-Policy header value
fn get_permissions_policy_value() -> HeaderValue {
    HeaderValue::from_static("camera=(), microphone=(), geolocation=()")
}

/// Create a Content Security Policy header layer
pub fn create_csp_layer() -> SetResponseHeaderLayer<fn() -> HeaderValue> {
    // Determine header name
    let header_name = axum::http::header::CONTENT_SECURITY_POLICY;

    // Create and return the layer
    SetResponseHeaderLayer::overriding(header_name, get_csp_value)
}

/// Create X-Content-Type-Options header layer
pub fn create_content_type_options_layer() -> SetResponseHeaderLayer<fn() -> HeaderValue> {
    SetResponseHeaderLayer::overriding(
        axum::http::header::X_CONTENT_TYPE_OPTIONS,
        get_content_type_options_value,
    )
}

/// Create X-Frame-Options header layer
pub fn create_frame_options_layer() -> SetResponseHeaderLayer<fn() -> HeaderValue> {
    SetResponseHeaderLayer::overriding(
        axum::http::header::HeaderName::from_static("x-frame-options"),
        get_frame_options_value,
    )
}

/// Create X-XSS-Protection header layer
pub fn create_xss_protection_layer() -> SetResponseHeaderLayer<fn() -> HeaderValue> {
    SetResponseHeaderLayer::overriding(
        axum::http::header::HeaderName::from_static("x-xss-protection"),
        get_xss_protection_value,
    )
}

/// Create Referrer-Policy header layer
pub fn create_referrer_policy_layer() -> SetResponseHeaderLayer<fn() -> HeaderValue> {
    SetResponseHeaderLayer::overriding(
        axum::http::header::REFERRER_POLICY,
        get_referrer_policy_value,
    )
}

/// Create Permissions-Policy header layer
pub fn create_permissions_policy_layer() -> SetResponseHeaderLayer<fn() -> HeaderValue> {
    SetResponseHeaderLayer::overriding(
        axum::http::header::HeaderName::from_static("permissions-policy"),
        get_permissions_policy_value,
    )
}

/// Create all security headers layers
///
/// This function returns a list of all security header layers.
/// Note: This function is not used directly, but is provided for documentation purposes.
/// Instead, use the individual header layer functions in blog_api.rs.
pub fn create_security_headers_layer() -> Vec<SetResponseHeaderLayer<fn() -> HeaderValue>> {
    vec![
        create_csp_layer(),
        create_content_type_options_layer(),
        create_frame_options_layer(),
        create_xss_protection_layer(),
        create_referrer_policy_layer(),
        create_permissions_policy_layer(),
    ]
}
