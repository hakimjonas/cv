/// Utility functions for Typst markup generation
use im::Vector;

/// Splits a full name into first name and last name
///
/// # Arguments
///
/// * `full_name` - The full name to split
///
/// # Returns
///
/// A tuple containing (first_name, last_name)
pub fn split_name(full_name: &str) -> (String, String) {
    let name_parts: Vec<&str> = full_name.split_whitespace().collect();

    if name_parts.is_empty() {
        return (String::new(), String::new());
    }

    if name_parts.len() == 1 {
        return (name_parts[0].to_string(), String::new());
    }

    let first_name = name_parts[0..name_parts.len() - 1].join(" ");

    let last_name = name_parts[name_parts.len() - 1].to_string();

    (first_name.to_string(), last_name)
}

/// Formats an email address for Typst (replacing @ with " at ")
///
/// # Arguments
///
/// * `email` - The email address to format
///
/// # Returns
///
/// The formatted email address
pub fn format_email_for_typst(email: &str) -> String {
    email.replace('@', " at ")
}

/// Joins a vector of strings with a separator
///
/// # Arguments
///
/// * `items` - The vector of strings to join
/// * `separator` - The separator to use
///
/// # Returns
///
/// The joined string
pub fn join_strings(items: &Vector<String>, separator: &str) -> String {
    items
        .iter()
        .map(|s| s.as_str())
        .collect::<Vec<_>>()
        .join(separator)
}

/// Appends a string to a Typst markup string
///
/// # Arguments
///
/// * `markup` - The Typst markup string to append to
/// * `content` - The content to append
///
/// # Returns
///
/// The updated Typst markup string
pub fn append_markup(markup: String, content: &str) -> String {
    let mut result = markup;
    result.push_str(content);
    result
}

/// Appends a line to a Typst markup string (adds a newline)
///
/// # Arguments
///
/// * `markup` - The Typst markup string to append to
/// * `content` - The content to append
///
/// # Returns
///
/// The updated Typst markup string
pub fn append_line(markup: String, content: &str) -> String {
    let mut result = markup;
    result.push_str(content);
    result.push('\n');
    result
}

/// Appends multiple lines to a Typst markup string
///
/// # Arguments
///
/// * `markup` - The Typst markup string to append to
/// * `content` - The content to append
///
/// # Returns
///
/// The updated Typst markup string
pub fn append_lines(markup: String, content: &str) -> String {
    let mut result = markup;
    result.push_str(content);
    result.push_str("\n\n");
    result
}
