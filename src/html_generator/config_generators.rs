//! Configuration file generators
//!
//! This module handles generation of various configuration files needed for
//! web deployment, including server configurations, PWA manifests, and SEO files.

use anyhow::{Context, Result};
use std::fs;

use super::utils::write_file;
use crate::site_config::FontConfig;

/// Generates an .htaccess file for Apache servers with optimized settings
///
/// # Arguments
///
/// * `path` - Path where the .htaccess file will be written
///
/// # Returns
///
/// A Result indicating success or failure
pub fn generate_htaccess(path: &str) -> Result<()> {
    let htaccess_content = r#"# Enable gzip compression
<IfModule mod_deflate.c>
  AddOutputFilterByType DEFLATE text/html text/plain text/xml text/css application/javascript application/json
  BrowserMatch ^Mozilla/4 gzip-only-text/html
  BrowserMatch ^Mozilla/4\.0[678] no-gzip
  BrowserMatch \bMSIE !no-gzip !gzip-only-text/html
</IfModule>

# Serve pre-compressed files if available
<IfModule mod_headers.c>
  <FilesMatch "\.(js|css|html)$">
    RewriteEngine On
    RewriteCond %{HTTP:Accept-Encoding} gzip
    RewriteCond %{REQUEST_FILENAME}.gz -f
    RewriteRule ^(.*)$ $1.gz [L]

    # Set proper content type for compressed files
    <Files *.js.gz>
      ForceType application/javascript
      Header set Content-Encoding gzip
    </Files>
    <Files *.css.gz>
      ForceType text/css
      Header set Content-Encoding gzip
    </Files>
    <Files *.html.gz>
      ForceType text/html
      Header set Content-Encoding gzip
    </Files>
  </FilesMatch>
</IfModule>

# Set cache headers
<IfModule mod_expires.c>
  ExpiresActive On

  # Images
  ExpiresByType image/jpeg "access plus 1 year"
  ExpiresByType image/png "access plus 1 year"
  ExpiresByType image/gif "access plus 1 year"
  ExpiresByType image/svg+xml "access plus 1 year"
  ExpiresByType image/webp "access plus 1 year"

  # CSS and JavaScript
  ExpiresByType text/css "access plus 1 month"
  ExpiresByType application/javascript "access plus 1 month"

  # HTML - shorter cache time
  ExpiresByType text/html "access plus 1 day"

  # Fonts
  ExpiresByType application/font-woff2 "access plus 1 year"
  ExpiresByType application/font-woff "access plus 1 year"
  ExpiresByType font/woff2 "access plus 1 year"
  ExpiresByType font/woff "access plus 1 year"
</IfModule>

# Security headers
<IfModule mod_headers.c>
  Header always set X-Content-Type-Options nosniff
  Header always set X-Frame-Options DENY
  Header always set X-XSS-Protection "1; mode=block"
  Header always set Strict-Transport-Security "max-age=31536000; includeSubDomains; preload"
</IfModule>
"#;

    write_file(path, htaccess_content)?;
    println!("Generated .htaccess file with Apache optimization settings");

    Ok(())
}

/// Generates a web.config file for IIS servers
///
/// # Arguments
///
/// * `path` - Path where the web.config file will be written
///
/// # Returns
///
/// A Result indicating success or failure
pub fn generate_web_config(path: &str) -> Result<()> {
    let web_config_content = r##"<?xml version="1.0" encoding="UTF-8"?>
<configuration>
  <system.webServer>
    <!-- Enable compression -->
    <httpCompression>
      <dynamicTypes>
        <add mimeType="text/html" enabled="true" />
        <add mimeType="text/css" enabled="true" />
        <add mimeType="application/javascript" enabled="true" />
        <add mimeType="application/json" enabled="true" />
      </dynamicTypes>
      <staticTypes>
        <add mimeType="text/css" enabled="true" />
        <add mimeType="application/javascript" enabled="true" />
      </staticTypes>
    </httpCompression>

    <!-- Set cache headers -->
    <staticContent>
      <clientCache cacheControlMode="UseMaxAge" cacheControlMaxAge="31536000.00:00:00" />

      <!-- Font MIME types -->
      <mimeMap fileExtension=".woff2" mimeType="font/woff2" />
      <mimeMap fileExtension=".woff" mimeType="font/woff" />
    </staticContent>

    <!-- Security headers -->
    <httpProtocol>
      <customHeaders>
        <add name="X-Content-Type-Options" value="nosniff" />
        <add name="X-Frame-Options" value="DENY" />
        <add name="X-XSS-Protection" value="1; mode=block" />
        <add name="Strict-Transport-Security" value="max-age=31536000; includeSubDomains; preload" />
      </customHeaders>
    </httpProtocol>

    <!-- URL Rewrite rules -->
    <rewrite>
      <rules>
        <!-- Redirect to HTTPS -->
        <rule name="Redirect to HTTPS" stopProcessing="true">
          <match url=".*" />
          <conditions>
            <add input="{HTTPS}" pattern="off" ignoreCase="true" />
          </conditions>
          <action type="Redirect" url="https://{HTTP_HOST}/{R:0}" redirectType="Permanent" />
        </rule>
      </rules>
    </rewrite>
  </system.webServer>
</configuration>
"##;

    write_file(path, web_config_content)?;
    println!("Generated web.config file with IIS optimization settings");

    Ok(())
}

/// Generates Netlify headers file for deployment optimization
///
/// # Arguments
///
/// * `path` - Path where the _headers file will be written
///
/// # Returns
///
/// A Result indicating success or failure
pub fn generate_netlify_headers(path: &str) -> Result<()> {
    let headers_content = r#"# Security headers
/*
  X-Frame-Options: DENY
  X-XSS-Protection: 1; mode=block
  X-Content-Type-Options: nosniff
  Referrer-Policy: strict-origin-when-cross-origin
  Strict-Transport-Security: max-age=31536000; includeSubDomains; preload

# Cache headers for static assets
/css/*
  Cache-Control: public, max-age=31536000, immutable

/js/*
  Cache-Control: public, max-age=31536000, immutable

/img/*
  Cache-Control: public, max-age=31536000, immutable

/fonts/*
  Cache-Control: public, max-age=31536000, immutable

# Shorter cache for HTML files
/*.html
  Cache-Control: public, max-age=86400

# PWA manifest
/manifest.json
  Cache-Control: public, max-age=86400
  Content-Type: application/manifest+json

# Service worker
/service-worker.js
  Cache-Control: no-cache
"#;

    write_file(path, headers_content)?;
    println!("Generated Netlify _headers file with cache and security headers");

    Ok(())
}

/// Generates Netlify redirects file
///
/// # Arguments
///
/// * `path` - Path where the _redirects file will be written
///
/// # Returns
///
/// A Result indicating success or failure
pub fn generate_netlify_redirects(path: &str) -> Result<()> {
    let redirects_content = r#"# Redirect index.html to root
/index.html / 301

# SPA fallback for client-side routing (if needed in future)
# /* /index.html 200
"#;

    write_file(path, redirects_content)?;
    println!("Generated Netlify _redirects file");

    Ok(())
}

/// Generates a robots.txt file for SEO
///
/// # Arguments
///
/// * `path` - Path where the robots.txt file will be written
///
/// # Returns
///
/// A Result indicating success or failure
pub fn generate_robots_txt(path: &str) -> Result<()> {
    let robots_content = r#"User-agent: *
Allow: /

# Sitemap location (update when sitemap is implemented)
# Sitemap: https://yourdomain.com/sitemap.xml

# Disallow crawling of certain paths
Disallow: /service-worker.js
Disallow: /*.gz$
"#;

    write_file(path, robots_content)?;
    println!("Generated robots.txt file with SEO-friendly rules");

    Ok(())
}

/// Generates a manifest.json file for Progressive Web App (PWA) support
///
/// # Arguments
///
/// * `path` - Path where the manifest.json file will be written
///
/// # Returns
///
/// A Result indicating success or failure
pub fn generate_manifest_json(path: &str) -> Result<()> {
    let manifest_content = r##"{
  "name": "Professional CV & Portfolio",
  "short_name": "CV Portfolio",
  "description": "Professional CV and portfolio showcasing skills and projects",
  "start_url": "/",
  "display": "standalone",
  "background_color": "#faf4ed",
  "theme_color": "#286983",
  "icons": [
    {
      "src": "img/icon-192.png",
      "sizes": "192x192",
      "type": "image/png",
      "purpose": "any maskable"
    },
    {
      "src": "img/icon-512.png",
      "sizes": "512x512",
      "type": "image/png",
      "purpose": "any maskable"
    }
  ]
}"##;

    // Write the manifest.json file
    fs::write(path, manifest_content)
        .with_context(|| format!("Failed to write manifest.json file to {path}"))?;

    println!("Generated manifest.json file for PWA support ");

    Ok(())
}

/// Generates a service worker for offline support
///
/// # Arguments
///
/// * `path` - Path where the service-worker.js file will be written
///
/// # Returns
///
/// A Result indicating success or failure
pub fn generate_service_worker(path: &str) -> Result<()> {
    let sw_content = r##"// Service Worker for CV Portfolio
// Provides basic caching for offline support

const CACHE_NAME = 'cv-portfolio-v1';
const urlsToCache = [
  '/',
  '/cv.html',
  '/css/main.css',
  '/js/scripts.js',
  '/manifest.json'
];

// Install event - cache resources
self.addEventListener('install', event => {
  event.waitUntil(
    caches.open(CACHE_NAME)
      .then(cache => {
        console.log('Opened cache');
        return cache.addAll(urlsToCache);
      })
  );
});

// Fetch event - serve cached content when offline
self.addEventListener('fetch', event => {
  event.respondWith(
    caches.match(event.request)
      .then(response => {
        // Return cached version or fetch from network
        return response || fetch(event.request);
      }
    )
  );
});

// Activate event - clean up old caches
self.addEventListener('activate', event => {
  event.waitUntil(
    caches.keys().then(cacheNames => {
      return Promise.all(
        cacheNames.map(cacheName => {
          if (cacheName !== CACHE_NAME) {
            console.log('Deleting old cache:', cacheName);
            return caches.delete(cacheName);
          }
        })
      );
    })
  );
});
"##;

    write_file(path, sw_content)?;
    println!("Generated service worker for offline support");

    Ok(())
}

/// Generates CSS for font configuration
///
/// # Arguments
///
/// * `font_config` - Font configuration from site config
/// * `path` - Path where the fonts.css file will be written
///
/// # Returns
///
/// A Result indicating success or failure
pub fn generate_font_css(font_config: &FontConfig, path: &str) -> Result<()> {
    let mut css_content = String::new();

    // Add font imports based on source
    match font_config.source.as_deref() {
        Some("nerd-fonts") => {
            css_content
                .push_str("@import url('https://www.nerdfonts.com/assets/css/webfont.css');\n\n");
        }
        Some("google-fonts") => {
            if let Some(primary) = &font_config.primary {
                css_content.push_str(&format!(
                    "@import url('https://fonts.googleapis.com/css2?family={}:wght@{}..{}&display=swap');\n\n",
                    primary.replace(' ', "+"),
                    font_config.weight_regular.unwrap_or(400),
                    font_config.weight_bold.unwrap_or(700)
                ));
            }
        }
        _ => {
            // Default to system fonts
            css_content.push_str("/* Using system fonts */\n\n");
        }
    }

    // Generate CSS variables for font configuration
    css_content.push_str(":root {\n");

    if let Some(primary) = &font_config.primary {
        let fallback = font_config.fallback.as_deref().unwrap_or("monospace");
        css_content.push_str(&format!(
            "  --font-family-primary: \"{}\", {};\n",
            primary, fallback
        ));
    }

    if let Some(base_size) = &font_config.base_size {
        css_content.push_str(&format!("  --font-size-base: {};\n", base_size));
    }

    if let Some(weight_regular) = font_config.weight_regular {
        css_content.push_str(&format!("  --font-weight-regular: {};\n", weight_regular));
    }

    if let Some(weight_bold) = font_config.weight_bold {
        css_content.push_str(&format!("  --font-weight-bold: {};\n", weight_bold));
    }

    css_content.push_str("}\n");

    // Write the CSS file
    fs::write(path, css_content)
        .with_context(|| format!("Failed to write font CSS file to {path}"))?;

    println!("Generated dynamic font CSS: {path}");

    Ok(())
}
