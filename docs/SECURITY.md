# Security Features

This document describes the security features implemented in the blog API and how to use them.

## Overview

The blog API has been enhanced with several security features to protect against common web vulnerabilities:

1. **Rate Limiting**: Prevents abuse by limiting the number of requests a client can make in a given time period.
2. **Input Validation**: Ensures that user input is properly validated and sanitized to prevent injection attacks.
3. **CSRF Protection**: Protects against Cross-Site Request Forgery attacks by requiring a token for state-changing operations.
4. **Content Security Policy**: Prevents Cross-Site Scripting (XSS) attacks by restricting what resources can be loaded by the browser.
5. **Secure File Permissions**: Ensures that the SQLite database file has secure permissions to prevent unauthorized access.

## Rate Limiting

The blog API implements rate limiting to prevent abuse and ensure fair usage of the API. By default, clients are limited to 100 requests per minute. When the rate limit is exceeded, the API returns a 429 Too Many Requests status code.

The rate limiting configuration can be modified in the `create_blog_api_router` function in `blog_api.rs`:

```
// Configure rate limiting
let rate_limit_config = RateLimiterConfig {
    max_requests: 100,         // 100 requests per minute
    window_seconds: 60,        // 1 minute window
    include_headers: true,     // Include rate limit headers in response
    status_code: StatusCode::TOO_MANY_REQUESTS,
};
```

The API includes rate limit headers in the response:

- `X-RateLimit-Limit`: The maximum number of requests allowed in the current window
- `X-RateLimit-Remaining`: The number of requests remaining in the current window
- `X-RateLimit-Reset`: The number of seconds until the rate limit window resets

## Input Validation

The blog API implements comprehensive input validation and sanitization to prevent injection attacks. All user input is validated and sanitized before being processed.

The validation rules are defined in the `BlogPostValidation` struct in `blog_validation.rs`:

```
/// Validation rules for blog posts
#[derive(Debug, Validate)]
pub struct BlogPostValidation {
    /// Title must be between 3 and 200 characters
    #[validate(length(min = 3, max = 200, message = "Title must be between 3 and 200 characters"))]
    pub title: String,

    /// Slug must be between 3 and 100 characters and contain only lowercase letters, numbers, and hyphens
    #[validate(length(min = 3, max = 100, message = "Slug must be between 3 and 100 characters"))]
    #[validate(regex(path = "SLUG_REGEX", message = "Slug must contain only lowercase letters, numbers, and hyphens"))]
    pub slug: String,

    // ... other fields ...
}
```

The API also sanitizes HTML content to prevent XSS attacks. The sanitization rules are defined in the `sanitize_html` function in `blog_validation.rs`. The function uses the `ammonia` crate to sanitize HTML content by allowing only safe tags and attributes.

## CSRF Protection

The blog API implements CSRF protection to prevent Cross-Site Request Forgery attacks. CSRF protection is implemented using the `axum-csrf` crate.

The CSRF protection configuration can be modified in the `create_blog_api_router` function in `blog_api.rs`:

```
// Configure CSRF protection
let csrf_config = CsrfProtectionConfig {
    token_validity_seconds: 3600,  // 1 hour
    include_headers: true,
    cookie_name: "csrf_token".to_string(),
    cookie_path: "/".to_string(),
    cookie_secure: true,
    cookie_http_only: true,
    cookie_same_site: axum::http::header::SameSite::Strict,
};
```

To use the CSRF protection:

1. Make a GET request to any endpoint to get a CSRF token in the response headers (`X-CSRF-Token`).
2. Include the CSRF token in the `X-CSRF-Token` header for all state-changing requests (POST, PUT, DELETE).

## Content Security Policy

The blog API implements a Content Security Policy (CSP) to prevent Cross-Site Scripting (XSS) attacks. The CSP restricts what resources can be loaded by the browser.

The CSP configuration can be modified in the `create_csp_header_value` function in `content_security_policy.rs`:

```
let mut directives = vec![
    // Default policy: block everything not explicitly allowed
    "default-src 'self'",
    
    // Script sources: only allow scripts from the same origin and inline scripts with nonce
    "script-src 'self' 'unsafe-inline'",
    
    // Style sources: only allow styles from the same origin and inline styles
    "style-src 'self' 'unsafe-inline'",
    
    // ... other directives ...
];
```

The API also includes other security headers:

- `X-Content-Type-Options: nosniff`: Prevents MIME type sniffing
- `X-Frame-Options: SAMEORIGIN`: Prevents clickjacking
- `X-XSS-Protection: 1; mode=block`: Enables XSS filtering in browsers
- `Referrer-Policy: strict-origin-when-cross-origin`: Controls how much referrer information is sent
- `Permissions-Policy: camera=(), microphone=(), geolocation=()`: Controls which browser features can be used

## Secure File Permissions

The blog API implements secure file permissions for the SQLite database file to prevent unauthorized access. The database file is created with 0600 permissions (owner read/write only) and the parent directory is created with 0700 permissions (owner read/write/execute only).

The secure file permissions are implemented in the `secure_db_permissions` function in `check_db_permissions.rs`:

```
/// Sets secure permissions for a database file and its parent directory
///
/// This function sets the database file permissions to 0600 (read/write for owner only)
/// and the parent directory permissions to 0700 (read/write/execute for owner only).
pub fn secure_db_permissions(db_path: &Path) -> Result<()> {
    // ... implementation ...
}
```

## Testing Security Features

The security features can be tested using the `security_test` binary:

```bash
cargo run --bin security_test
```

This will run a series of tests to verify that all security features are working correctly:

1. Secure file permissions test
2. Rate limiting test
3. CSRF protection test
4. Content Security Policy test

## Security Best Practices

In addition to the security features implemented in the blog API, the following best practices should be followed:

1. **Keep dependencies up to date**: Regularly update dependencies to get the latest security patches.
2. **Use HTTPS**: Always use HTTPS in production to encrypt data in transit.
3. **Implement authentication and authorization**: Add proper authentication and authorization to control access to the API.
4. **Log security events**: Log security-related events for monitoring and auditing.
5. **Perform security testing**: Regularly perform security testing, including penetration testing and vulnerability scanning.
6. **Follow the principle of least privilege**: Give users and processes only the permissions they need to perform their tasks.
7. **Implement proper error handling**: Don't expose sensitive information in error messages.
8. **Use secure coding practices**: Follow secure coding practices to prevent security vulnerabilities.

## Reporting Security Issues

If you discover a security issue, please report it by sending an email to security@example.com. Please do not disclose security issues publicly until they have been handled by the security team.

## References

- [OWASP Top Ten](https://owasp.org/www-project-top-ten/)
- [OWASP API Security Top Ten](https://owasp.org/www-project-api-security/)
- [Content Security Policy](https://developer.mozilla.org/en-US/docs/Web/HTTP/CSP)
- [Cross-Site Request Forgery Prevention](https://cheatsheetseries.owasp.org/cheatsheets/Cross-Site_Request_Forgery_Prevention_Cheat_Sheet.html)
- [Input Validation](https://cheatsheetseries.owasp.org/cheatsheets/Input_Validation_Cheat_Sheet.html)
- [Rate Limiting](https://cheatsheetseries.owasp.org/cheatsheets/Denial_of_Service_Cheat_Sheet.html#rate-limiting)