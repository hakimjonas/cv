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

/// Optimize a CSS file by minifying it
pub fn optimize_css_file(input_path: &Path, output_path: &Path) -> Result<()> {
    let css_content = fs::read_to_string(input_path)
        .with_context(|| format!("Failed to read CSS file: {:?}", input_path))?;

    let minified = minify_css(&css_content);

    // Ensure output directory exists
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(output_path, minified)
        .with_context(|| format!("Failed to write minified CSS: {:?}", output_path))?;

    Ok(())
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
