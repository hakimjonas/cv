//! Asset processor module for optimizing and processing static assets
//!
//! This module provides functions for minifying, compressing, and processing
//! static assets like HTML, CSS, and JavaScript files. It also generates
//! server configuration files for different hosting environments.

use anyhow::{Context, Result};
use flate2::Compression;
use flate2::write::GzEncoder;
use im::Vector;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Minifies HTML content
///
/// # Arguments
///
/// * `content` - HTML content to minify
///
/// # Returns
///
/// A Result containing the minified HTML or an error
pub fn minify_html_content(content: &str) -> Result<Vec<u8>> {
    // Configure HTML minification
    let cfg = minify_html::Cfg {
        minify_css: true,
        minify_js: false, // Disabled JS minification to avoid panic in minify-js
        ..minify_html::Cfg::default()
    };

    // Minify the HTML
    let minified = minify_html::minify(content.as_bytes(), &cfg);
    Ok(minified)
}

/// Minifies CSS content
///
/// # Arguments
///
/// * `content` - CSS content to minify
///
/// # Returns
///
/// A Result containing the minified CSS or an error
pub fn minify_css_content(content: &str) -> Result<String> {
    // For now, we'll use a simple approach to minify CSS
    // This is not as effective as using lightningcss, but it will work for basic minification

    // Remove comments
    let re_comments = regex::Regex::new(r"/\*[\s\S]*?\*/").unwrap();
    let without_comments = re_comments.replace_all(content, "");

    // Remove whitespace
    let re_whitespace = regex::Regex::new(r"\s+").unwrap();
    let minified = re_whitespace.replace_all(&without_comments, " ");

    // Remove spaces around certain characters
    let re_spaces = regex::Regex::new(r"\s*([{};:,>+])\s*").unwrap();
    let minified = re_spaces.replace_all(&minified, "$1");

    // Remove trailing semicolons
    let re_semicolons = regex::Regex::new(r";\s*}").unwrap();
    let minified = re_semicolons.replace_all(&minified, "}");

    Ok(minified.to_string())
}

/// Writes gzipped content to a file
///
/// # Arguments
///
/// * `path` - Path where the gzipped content will be written
/// * `content` - Content to compress and write
///
/// # Returns
///
/// A Result indicating success or failure
pub fn write_gzipped_file(path: &str, content: &[u8]) -> Result<()> {
    // Create the file
    let file =
        fs::File::create(path).with_context(|| format!("Failed to create gzipped file: {path}"))?;

    // Create a gzip encoder
    let mut encoder = GzEncoder::new(file, Compression::best());

    // Write the content
    encoder
        .write_all(content)
        .with_context(|| format!("Failed to write gzipped content to {path}"))?;

    // Finish the compression
    encoder
        .finish()
        .with_context(|| format!("Failed to finish gzip compression for {path}"))?;

    Ok(())
}

/// Writes content to a file, with optional minification and compression in release mode
///
/// # Arguments
///
/// * `path` - Path where the content will be written
/// * `content` - Content to write to the file
///
/// # Returns
///
/// A Result indicating success or failure
pub fn write_file(path: &str, content: &str) -> Result<()> {
    // In debug mode, just write the file as is
    if cfg!(debug_assertions) {
        return fs::write(path, content).with_context(|| format!("Failed to write to {path}"));
    }

    // In release mode, apply optimizations based on file extension
    let path_obj = Path::new(path);
    let extension = path_obj
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");

    match extension {
        "html" => {
            // Minify HTML content
            let minified = minify_html_content(content)?;

            // Write minified HTML
            fs::write(path, &minified)
                .with_context(|| format!("Failed to write minified HTML to {path}"))?;

            // Also create a gzipped version
            write_gzipped_file(&format!("{path}.gz"), minified.as_slice())?;
        }
        "css" => {
            // Minify CSS content
            let minified = minify_css_content(content)?;

            // Write minified CSS
            fs::write(path, &minified)
                .with_context(|| format!("Failed to write minified CSS to {path}"))?;

            // Also create a gzipped version
            write_gzipped_file(&format!("{path}.gz"), minified.as_bytes())?;
        }
        "js" => {
            // For JS files, we'll just compress them without minification for now
            // (could add JS minification in the future)
            fs::write(path, content).with_context(|| format!("Failed to write to {path}"))?;

            // Create a gzipped version
            write_gzipped_file(&format!("{path}.gz"), content.as_bytes())?;
        }
        _ => {
            // For other file types, just write as is
            fs::write(path, content).with_context(|| format!("Failed to write to {path}"))?;
        }
    }

    Ok(())
}

/// Ensures that the parent directory of a file path exists
///
/// # Arguments
///
/// * `file_path` - Path to a file whose parent directory should exist
///
/// # Returns
///
/// A Result indicating success or failure
pub fn ensure_parent_dir_exists(file_path: &str) -> Result<()> {
    Path::new(file_path)
        .parent()
        .map(fs::create_dir_all)
        .transpose()
        .context("Failed to create parent directory")?;

    Ok(())
}

/// Generates an .htaccess file for Apache servers with performance optimizations
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
  ExpiresByType font/ttf "access plus 1 year"
  ExpiresByType font/otf "access plus 1 year"
  ExpiresByType font/woff "access plus 1 year"
  ExpiresByType font/woff2 "access plus 1 year"

  # Default
  ExpiresDefault "access plus 1 month"
</IfModule>

# Add security headers
<IfModule mod_headers.c>
  # Protect against XSS attacks
  Header set X-XSS-Protection "1; mode=block"

  # Prevent MIME-type sniffing
  Header set X-Content-Type-Options "nosniff"

  # Referrer policy
  Header set Referrer-Policy "strict-origin-when-cross-origin"
</IfModule>
"#;

    // Write the .htaccess file
    fs::write(path, htaccess_content)
        .with_context(|| format!("Failed to write .htaccess file to {path}"))?;

    println!("Generated .htaccess file with performance optimizations");

    Ok(())
}

/// Generates a web.config file for IIS servers with performance optimizations
///
/// # Arguments
///
/// * `path` - Path where the web.config file will be written
///
/// # Returns
///
/// A Result indicating success or failure
pub fn generate_web_config(path: &str) -> Result<()> {
    let web_config_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<configuration>
  <system.webServer>
    <!-- Enable gzip compression -->
    <urlCompression doStaticCompression="true" doDynamicCompression="true" />

    <!-- Set static content caching -->
    <staticContent>
      <clientCache cacheControlMode="UseMaxAge" cacheControlMaxAge="30.00:00:00" />

      <!-- Add proper MIME types if needed -->
      <remove fileExtension=".woff" />
      <remove fileExtension=".woff2" />
      <mimeMap fileExtension=".woff" mimeType="application/font-woff" />
      <mimeMap fileExtension=".woff2" mimeType="application/font-woff2" />
    </staticContent>

    <!-- Configure custom HTTP response headers -->
    <httpProtocol>
      <customHeaders>
        <add name="X-Content-Type-Options" value="nosniff" />
        <add name="X-XSS-Protection" value="1; mode=block" />
        <add name="Referrer-Policy" value="strict-origin-when-cross-origin" />
      </customHeaders>
    </httpProtocol>

    <!-- Serve pre-compressed files if available -->
    <rewrite>
      <rules>
        <rule name="Serve gzipped CSS" enabled="true">
          <match url="^(.*)\.css$" ignoreCase="true" />
          <conditions>
            <add input="{HTTP_ACCEPT_ENCODING}" pattern="gzip" />
            <add input="{REQUEST_FILENAME}.gz" matchType="IsFile" />
          </conditions>
          <action type="Rewrite" url="{R:1}.css.gz" />
        </rule>
        <rule name="Serve gzipped JS" enabled="true">
          <match url="^(.*)\.js$" ignoreCase="true" />
          <conditions>
            <add input="{HTTP_ACCEPT_ENCODING}" pattern="gzip" />
            <add input="{REQUEST_FILENAME}.gz" matchType="IsFile" />
          </conditions>
          <action type="Rewrite" url="{R:1}.js.gz" />
        </rule>
        <rule name="Serve gzipped HTML" enabled="true">
          <match url="^(.*)\.html$" ignoreCase="true" />
          <conditions>
            <add input="{HTTP_ACCEPT_ENCODING}" pattern="gzip" />
            <add input="{REQUEST_FILENAME}.gz" matchType="IsFile" />
          </conditions>
          <action type="Rewrite" url="{R:1}.html.gz" />
        </rule>
      </rules>
      <outboundRules>
        <rule name="Add gzip content encoding for CSS">
          <match serverVariable="RESPONSE_CONTENT_ENCODING" pattern=".*" />
          <conditions>
            <add input="{REQUEST_URI}" pattern="\.css\.gz$" />
          </conditions>
          <action type="Rewrite" value="gzip" />
        </rule>
        <rule name="Add gzip content encoding for JS">
          <match serverVariable="RESPONSE_CONTENT_ENCODING" pattern=".*" />
          <conditions>
            <add input="{REQUEST_URI}" pattern="\.js\.gz$" />
          </conditions>
          <action type="Rewrite" value="gzip" />
        </rule>
        <rule name="Add gzip content encoding for HTML">
          <match serverVariable="RESPONSE_CONTENT_ENCODING" pattern=".*" />
          <conditions>
            <add input="{REQUEST_URI}" pattern="\.html\.gz$" />
          </conditions>
          <action type="Rewrite" value="gzip" />
        </rule>
        <rule name="Set correct content type for gzipped CSS">
          <match serverVariable="RESPONSE_CONTENT_TYPE" pattern=".*" />
          <conditions>
            <add input="{REQUEST_URI}" pattern="\.css\.gz$" />
          </conditions>
          <action type="Rewrite" value="text/css" />
        </rule>
        <rule name="Set correct content type for gzipped JS">
          <match serverVariable="RESPONSE_CONTENT_TYPE" pattern=".*" />
          <conditions>
            <add input="{REQUEST_URI}" pattern="\.js\.gz$" />
          </conditions>
          <action type="Rewrite" value="application/javascript" />
        </rule>
        <rule name="Set correct content type for gzipped HTML">
          <match serverVariable="RESPONSE_CONTENT_TYPE" pattern=".*" />
          <conditions>
            <add input="{REQUEST_URI}" pattern="\.html\.gz$" />
          </conditions>
          <action type="Rewrite" value="text/html" />
        </rule>
      </outboundRules>
    </rewrite>
  </system.webServer>
</configuration>
"#;

    // Write the web.config file
    fs::write(path, web_config_content)
        .with_context(|| format!("Failed to write web.config file to {path}"))?;

    println!("Generated web.config file with performance optimizations");

    Ok(())
}

/// Generates a _headers file for Netlify hosting with performance optimizations
///
/// # Arguments
///
/// * `path` - Path where the _headers file will be written
///
/// # Returns
///
/// A Result indicating success or failure
pub fn generate_netlify_headers(path: &str) -> Result<()> {
    let headers_content = r#"# Global headers for all pages
/*
  X-Content-Type-Options: nosniff
  X-XSS-Protection: 1; mode=block
  Referrer-Policy: strict-origin-when-cross-origin
  X-Frame-Options: DENY
  Content-Security-Policy: default-src 'self'; style-src 'self' 'unsafe-inline' https://cdnjs.cloudflare.com; script-src 'self' 'unsafe-inline'; font-src 'self' https://cdnjs.cloudflare.com; img-src 'self' data: https://avatars.githubusercontent.com;

# Cache control for HTML files - shorter cache time
/*.html
  Cache-Control: public, max-age=86400

# Cache control for CSS and JS files
/*.css
  Cache-Control: public, max-age=2592000
/*.js
  Cache-Control: public, max-age=2592000

# Cache control for images and fonts - longer cache time
/*.jpg
  Cache-Control: public, max-age=31536000
/*.jpeg
  Cache-Control: public, max-age=31536000
/*.png
  Cache-Control: public, max-age=31536000
/*.gif
  Cache-Control: public, max-age=31536000
/*.svg
  Cache-Control: public, max-age=31536000
/*.webp
  Cache-Control: public, max-age=31536000
/*.woff
  Cache-Control: public, max-age=31536000
/*.woff2
  Cache-Control: public, max-age=31536000
/*.ttf
  Cache-Control: public, max-age=31536000
/*.otf
  Cache-Control: public, max-age=31536000

# Brotli and Gzip pre-compressed files
/*.br
  Content-Encoding: br
/*.gz
  Content-Encoding: gzip
"#;

    // Write the _headers file
    fs::write(path, headers_content)
        .with_context(|| format!("Failed to write _headers file to {path}"))?;

    println!("Generated _headers file for Netlify with performance optimizations");

    Ok(())
}

/// Generates a _redirects file for Netlify hosting with common redirects
///
/// # Arguments
///
/// * `path` - Path where the _redirects file will be written
///
/// # Returns
///
/// A Result indicating success or failure
pub fn generate_netlify_redirects(path: &str) -> Result<()> {
    let redirects_content = r#"# Redirect HTTP to HTTPS
http://:hostname/* https://:hostname/:splat 301!

# Handle 404 errors
/* /index.html 404

# Redirect www to non-www
https://www.:hostname/* https://:hostname/:splat 301!
"#;

    // Write the _redirects file
    fs::write(path, redirects_content)
        .with_context(|| format!("Failed to write _redirects file to {path}"))?;

    println!("Generated _redirects file for Netlify with common redirects");

    Ok(())
}

/// Generates a robots.txt file with SEO-friendly rules
///
/// # Arguments
///
/// * `path` - Path where the robots.txt file will be written
///
/// # Returns
///
/// A Result indicating success or failure
pub fn generate_robots_txt(path: &str) -> Result<()> {
    let robots_content = r#"# Allow all crawlers
User-agent: *
Allow: /

# Sitemap location (uncomment and update if you have a sitemap)
# Sitemap: https://yourdomain.com/sitemap.xml

# Disallow certain paths if needed
# Disallow: /private/
# Disallow: /admin/
"#;

    // Write the robots.txt file
    fs::write(path, robots_content)
        .with_context(|| format!("Failed to write robots.txt file to {path}"))?;

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
    let sw_content = r#"// Service Worker for CV Portfolio PWA
const CACHE_NAME = "cv-portfolio-cache-v1";
const ASSETS_TO_CACHE = [
  "/",
  "/index.html",
  "/cv.html",
  "/style.css",
  "/manifest.json",
  "/img/icon-192.png",
  "/img/icon-512.png",
  // Add other assets you want to cache
];

// Install event - cache assets
self.addEventListener("install", (event) => {
  event.waitUntil(
    caches.open(CACHE_NAME)
      .then((cache) => {
        console.log("Opened cache");
        return cache.addAll(ASSETS_TO_CACHE);
      })
      .then(() => self.skipWaiting())
  );
});

// Activate event - clean up old caches
self.addEventListener("activate", (event) => {
  const cacheWhitelist = [CACHE_NAME];

  event.waitUntil(
    caches.keys().then((cacheNames) => {
      return Promise.all(
        cacheNames.map((cacheName) => {
          if (cacheWhitelist.indexOf(cacheName) === -1) {
            // Delete old caches
            return caches.delete(cacheName);
          }
        })
      );
    }).then(() => self.clients.claim())
  );
});

// Fetch event - serve from cache or network
self.addEventListener("fetch", (event) => {
  event.respondWith(
    caches.match(event.request)
      .then((response) => {
        // Cache hit - return response
        if (response) {
          return response;
        }

        // Clone the request
        const fetchRequest = event.request.clone();

        return fetch(fetchRequest).then(
          (response) => {
            // Check if valid response
            if (!response || response.status !== 200 || response.type !== "basic") {
              return response;
            }

            // Clone the response
            const responseToCache = response.clone();

            // Cache the fetched response
            caches.open(CACHE_NAME)
              .then((cache) => {
                cache.put(event.request, responseToCache);
              });

            return response;
          }
        ).catch(() => {
          // If fetch fails (offline), try to serve the fallback
          if (event.request.mode === "navigate") {
            return caches.match("/index.html");
          }
        });
      })
  );
});
"#;

    // Write the service-worker.js file
    fs::write(path, sw_content)
        .with_context(|| format!("Failed to write service-worker.js file to {path}"))?;

    println!("Generated service-worker.js file for offline support");

    Ok(())
}

/// Generates all server configuration files for a given output directory
///
/// # Arguments
///
/// * `output_dir` - Path to the output directory
///
/// # Returns
///
/// A Result indicating success or failure
pub fn generate_server_configs(output_dir: &str) -> Result<()> {
    let output_path = Path::new(output_dir);

    // Generate .htaccess file for Apache servers
    let htaccess_path = output_path
        .join(".htaccess")
        .to_str()
        .context("Failed to convert path to string")?
        .to_string();
    generate_htaccess(&htaccess_path)?;

    // Generate web.config file for IIS servers
    let web_config_path = output_path
        .join("web.config")
        .to_str()
        .context("Failed to convert path to string")?
        .to_string();
    generate_web_config(&web_config_path)?;

    // Generate _headers file for Netlify
    let netlify_headers_path = output_path
        .join("_headers")
        .to_str()
        .context("Failed to convert path to string")?
        .to_string();
    generate_netlify_headers(&netlify_headers_path)?;

    // Generate _redirects file for Netlify
    let netlify_redirects_path = output_path
        .join("_redirects")
        .to_str()
        .context("Failed to convert path to string")?
        .to_string();
    generate_netlify_redirects(&netlify_redirects_path)?;

    // Generate robots.txt file
    let robots_path = output_path
        .join("robots.txt")
        .to_str()
        .context("Failed to convert path to string")?
        .to_string();
    generate_robots_txt(&robots_path)?;

    // Generate manifest.json file for PWA support
    let manifest_path = output_path
        .join("manifest.json")
        .to_str()
        .context("Failed to convert path to string")?
        .to_string();
    generate_manifest_json(&manifest_path)?;

    // Generate service worker for offline support
    let sw_path = output_path
        .join("service-worker.js")
        .to_str()
        .context("Failed to convert path to string")?
        .to_string();
    generate_service_worker(&sw_path)?;

    Ok(())
}

/// Copy static assets to the output directory, excluding specified files
///
/// # Arguments
///
/// * `static_dir` - Directory containing static assets
/// * `output_dir` - Directory where assets will be copied
/// * `exclude` - Array of filenames to exclude from copying
///
/// # Returns
///
/// A Result indicating success or failure
pub fn copy_static_assets(static_dir: &str, output_dir: &str, exclude: &[&str]) -> Result<()> {
    // Ensure the output directory exists
    fs::create_dir_all(output_dir).context("Failed to create output directory")?;

    // Recursively copy the static directory, excluding specified files
    copy_dir_recursively(static_dir, output_dir, exclude)
}

/// Represents a file system entry (file or directory)
#[derive(Debug, Clone)]
enum FsEntry {
    File(PathBuf),
    Directory(PathBuf),
}

/// Recursively copy a directory and its contents, excluding specified files
///
/// # Arguments
///
/// * `src` - Source directory
/// * `dst` - Destination directory
/// * `exclude` - Array of filenames to exclude from copying
///
/// # Returns
///
/// A Result indicating success or failure
fn copy_dir_recursively(src: &str, dst: &str, exclude: &[&str]) -> Result<()> {
    // Get all entries in the source directory
    let entries = list_directory_entries(src)?;

    // Process each entry using functional patterns
    entries.iter().try_for_each(|entry| match entry {
        FsEntry::File(path) => {
            // Check if the file should be excluded
            let file_name = path
                .file_name()
                .context("Failed to get file name")?
                .to_str()
                .context("Failed to convert file name to string")?;

            if exclude.contains(&file_name) {
                println!("Skipping excluded file: {file_name}");
                Ok(())
            } else {
                copy_file(path, dst)
            }
        }
        FsEntry::Directory(path) => copy_directory(path, dst, exclude),
    })
}

/// Lists all entries in a directory
///
/// # Arguments
///
/// * `dir_path` - Path to the directory
///
/// # Returns
///
/// A Result containing a Vector of FsEntry or an error
fn list_directory_entries(dir_path: &str) -> Result<Vector<FsEntry>> {
    let entries =
        fs::read_dir(dir_path).with_context(|| format!("Failed to read directory: {dir_path}"))?;

    // Convert DirEntry stream to Vector<FsEntry> using functional patterns
    let result = entries
        .filter_map(|entry_result| {
            entry_result
                .map_err(|e| anyhow::anyhow!("Failed to read directory entry: {}", e))
                .ok()
                .and_then(|entry| {
                    let path = entry.path();
                    if path.is_file() {
                        Some(FsEntry::File(path))
                    } else if path.is_dir() {
                        Some(FsEntry::Directory(path))
                    } else {
                        // Skip other types of entries (symlinks, etc.)
                        None
                    }
                })
        })
        .collect::<Vector<_>>();

    Ok(result)
}

/// Copies a file to a destination directory, with optional minification and compression in release mode
///
/// # Arguments
///
/// * `src_path` - Source file path
/// * `dst_dir` - Destination directory
///
/// # Returns
///
/// A Result indicating success or failure
fn copy_file(src_path: &Path, dst_dir: &str) -> Result<()> {
    let file_name = src_path
        .file_name()
        .context("Failed to get file name")?
        .to_str()
        .context("Failed to convert file name to string")?;

    let dst_path = Path::new(dst_dir).join(file_name);
    let dst_path_str = dst_path
        .to_str()
        .context("Failed to convert path to string")?;

    // In debug mode, just copy the file as is
    if cfg!(debug_assertions) {
        return fs::copy(src_path, &dst_path)
            .with_context(|| {
                format!(
                    "Failed to copy {} to {}",
                    src_path.display(),
                    dst_path.display()
                )
            })
            .map(|_| ());
    }

    // In release mode, apply optimizations based on file extension
    let extension = src_path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");

    match extension {
        "css" => {
            // Read the CSS file
            let content = fs::read_to_string(src_path)
                .with_context(|| format!("Failed to read CSS file: {}", src_path.display()))?;

            // Minify and write the CSS file
            let minified = minify_css_content(&content)?;
            fs::write(&dst_path, &minified).with_context(|| {
                format!("Failed to write minified CSS to {}", dst_path.display())
            })?;

            // Also create a gzipped version
            write_gzipped_file(&format!("{dst_path_str}.gz"), minified.as_bytes())?;

            println!("Optimized CSS file: {file_name}");
        }
        "js" => {
            // Read the JS file
            let content = fs::read_to_string(src_path)
                .with_context(|| format!("Failed to read JS file: {}", src_path.display()))?;

            // For now, we'll just compress JS files without minification
            // (could add JS minification in the future)
            fs::write(&dst_path, &content)
                .with_context(|| format!("Failed to write JS to {}", dst_path.display()))?;

            // Also create a gzipped version
            write_gzipped_file(&format!("{dst_path_str}.gz"), content.as_bytes())?;

            println!("Compressed JS file: {file_name}");
        }
        _ => {
            // For other file types, just copy as is
            fs::copy(src_path, &dst_path).with_context(|| {
                format!(
                    "Failed to copy {} to {}",
                    src_path.display(),
                    dst_path.display()
                )
            })?;
        }
    }

    Ok(())
}

/// Copies a directory to a destination directory, excluding specified files
///
/// # Arguments
///
/// * `src_path` - Source directory path
/// * `dst_dir` - Destination directory
/// * `exclude` - Array of filenames to exclude from copying
///
/// # Returns
///
/// A Result indicating success or failure
fn copy_directory(src_path: &Path, dst_dir: &str, exclude: &[&str]) -> Result<()> {
    let dir_name = src_path
        .file_name()
        .context("Failed to get directory name")?
        .to_str()
        .context("Failed to convert directory name to string")?;

    let dst_path = Path::new(dst_dir).join(dir_name);

    // Create the destination directory
    fs::create_dir_all(&dst_path)
        .with_context(|| format!("Failed to create directory: {}", dst_path.display()))?;

    // Recursively copy the subdirectory
    copy_dir_recursively(
        src_path
            .to_str()
            .context("Failed to convert path to string")?,
        dst_path
            .to_str()
            .context("Failed to convert path to string")?,
        exclude,
    )
}
