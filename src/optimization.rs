//! Build optimization module for CSS and JS minification

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Simple CSS minification - removes comments, whitespace, and line breaks
pub fn minify_css(css_content: &str) -> String {
    css_content
        // Remove CSS comments
        .split("/*")
        .enumerate()
        .map(|(i, part)| {
            if i == 0 {
                part.to_string()
            } else {
                part.split("*/").skip(1).collect::<Vec<_>>().join("")
            }
        })
        .collect::<String>()
        // Remove extra whitespace and line breaks
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
        // Remove spaces around CSS operators
        .replace(" { ", "{")
        .replace(" } ", "}")
        .replace(" { ", "{")
        .replace("; ", ";")
        .replace(": ", ":")
        .replace(", ", ",")
}

/// Simple JavaScript minification - removes comments and unnecessary whitespace
pub fn minify_js(js_content: &str) -> String {
    js_content
        // Remove single-line comments (basic implementation)
        .lines()
        .map(|line| {
            if let Some(comment_pos) = line.find("//") {
                // Check if // is inside a string
                let before_comment = &line[..comment_pos];
                let single_quotes = before_comment.matches('\'').count();
                let double_quotes = before_comment.matches('"').count();

                // Simple heuristic: if odd number of quotes, // is likely in a string
                if single_quotes % 2 == 1 || double_quotes % 2 == 1 {
                    line.to_string()
                } else {
                    before_comment.to_string()
                }
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
        // Remove multi-line comments (basic)
        .split("/*")
        .enumerate()
        .map(|(i, part)| {
            if i == 0 {
                part.to_string()
            } else {
                part.split("*/").skip(1).collect::<Vec<_>>().join("")
            }
        })
        .collect::<String>()
        // Remove extra whitespace
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Bundle and optimize a CSS file by resolving @imports and minifying
pub fn optimize_css_file(input_path: &Path, output_path: &Path) -> Result<()> {
    let bundled_css = bundle_css_imports(input_path)?;
    let minified = minify_css(&bundled_css);

    // Ensure output directory exists
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(output_path, minified).with_context(|| {
        format!(
            "Failed to write bundled and minified CSS: {:?}",
            output_path
        )
    })?;

    Ok(())
}

/// Bundle CSS by resolving @import statements into a single file
pub fn bundle_css_imports(css_path: &Path) -> Result<String> {
    let css_content = fs::read_to_string(css_path)
        .with_context(|| format!("Failed to read CSS file: {:?}", css_path))?;

    let css_dir = css_path.parent().unwrap_or_else(|| Path::new("."));
    let mut bundled_content = String::new();

    for line in css_content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("@import") {
            // Extract the imported file path
            if let Some(import_path) = extract_import_path(trimmed) {
                let full_import_path = css_dir.join(&import_path);

                if full_import_path.exists() {
                    // Recursively bundle imported file
                    match bundle_css_imports(&full_import_path) {
                        Ok(imported_content) => {
                            bundled_content
                                .push_str(&format!("\n/* Bundled from {} */\n", import_path));
                            bundled_content.push_str(&imported_content);
                            bundled_content.push('\n');
                        }
                        Err(e) => {
                            eprintln!("Warning: Failed to bundle {}: {}", import_path, e);
                            // Keep the original @import line as fallback
                            bundled_content.push_str(line);
                            bundled_content.push('\n');
                        }
                    }
                } else {
                    // Keep external imports (like Google Fonts)
                    bundled_content.push_str(line);
                    bundled_content.push('\n');
                }
            } else {
                // Keep the original line if we can't parse it
                bundled_content.push_str(line);
                bundled_content.push('\n');
            }
        } else {
            // Keep non-import lines
            bundled_content.push_str(line);
            bundled_content.push('\n');
        }
    }

    Ok(bundled_content)
}

/// Extract file path from @import statement
fn extract_import_path(import_line: &str) -> Option<String> {
    // Handle: @import "path/file.css";
    // Handle: @import url("path/file.css");

    if let Some(start) = import_line.find('"') {
        if let Some(end) = import_line[start + 1..].find('"') {
            let path = &import_line[start + 1..start + 1 + end];
            // Skip external URLs
            if path.starts_with("http") {
                return None;
            }
            return Some(path.to_string());
        }
    }

    None
}

/// Optimize a JavaScript file by minifying it
pub fn optimize_js_file(input_path: &Path, output_path: &Path) -> Result<()> {
    let js_content = fs::read_to_string(input_path)
        .with_context(|| format!("Failed to read JS file: {:?}", input_path))?;

    let minified = minify_js(&js_content);

    // Ensure output directory exists
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(output_path, minified)
        .with_context(|| format!("Failed to write minified JS: {:?}", output_path))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_css_minification() {
        let css = r#"
/* This is a comment */
.class {
    color: red;
    background: blue;
}

.another-class {
    margin: 10px;
}
        "#;

        let minified = minify_css(css);
        assert!(!minified.contains("/*"));
        assert!(!minified.contains("This is a comment"));
        assert!(minified.contains(".class{"));
        assert!(minified.contains("color:red"));
    }

    #[test]
    fn test_js_minification() {
        let js = r#"
// This is a comment
function test() {
    console.log("Hello"); // End comment
    return true;
}

/* Multi-line
   comment */
const x = 5;
        "#;

        let minified = minify_js(js);
        assert!(!minified.contains("// This is a comment"));
        assert!(!minified.contains("/* Multi-line"));
        assert!(minified.contains("function test()"));
        assert!(minified.contains("console.log(\"Hello\");"));
    }
}
