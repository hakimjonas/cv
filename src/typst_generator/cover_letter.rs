//! Cover letter Typst markup generation
//!
//! Generates Typst markup for cover letters using the grotesk-cv template.
//! Reuses the same metadata structure as the CV for consistent styling.

use super::utils::{append_line, append_lines, split_name};
use crate::cover_letter::CoverLetter;
use crate::cv_data::PersonalInfo;
use crate::site_config::TypstConfig;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use std::process::Command;

// Extension trait to enable method chaining with pipe (same as markup.rs)
trait Pipe: Sized {
    fn pipe<F, R>(self, f: F) -> R
    where
        F: FnOnce(Self) -> R,
    {
        f(self)
    }
}

impl Pipe for String {}

/// Generate a cover letter PDF
pub fn generate_cover_letter_pdf(
    letter: &CoverLetter,
    personal_info: &PersonalInfo,
    typst_config: &TypstConfig,
    temp_path: &str,
    output_path: &str,
) -> Result<()> {
    let typst_markup = generate_cover_letter_markup(letter, personal_info, typst_config);

    // Ensure output directory exists
    if let Some(parent) = Path::new(output_path).parent() {
        fs::create_dir_all(parent).context("Failed to create output directory")?;
    }

    // Write Typst markup to temporary file
    fs::write(temp_path, &typst_markup)
        .with_context(|| format!("Failed to write Typst markup to {temp_path}"))?;

    // Compile Typst to PDF
    let output = Command::new("typst")
        .arg("compile")
        .arg(temp_path)
        .arg(output_path)
        .output()
        .context(
            "Failed to execute 'typst' command.\n\
             \n\
             Is Typst installed?\n\
             - Install from: https://typst.app/\n\
             - Or run: cargo install typst-cli",
        )?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!(
            "Typst compilation failed:\n{}\n\nTemp file preserved at: {}",
            stderr,
            temp_path
        ));
    }

    // Clean up temp file
    let _ = fs::remove_file(temp_path);

    Ok(())
}

/// Generate Typst markup for a cover letter
fn generate_cover_letter_markup(
    letter: &CoverLetter,
    personal_info: &PersonalInfo,
    typst_config: &TypstConfig,
) -> String {
    String::new()
        .pipe(|s| generate_imports(s, typst_config))
        .pipe(|s| generate_meta(s, personal_info, typst_config))
        .pipe(|s| generate_document_setup(s, personal_info))
        .pipe(|s| generate_body(s, letter))
}

/// Generate imports (same pattern as markup.rs)
fn generate_imports(markup: String, typst_config: &TypstConfig) -> String {
    markup
        .pipe(|s| {
            append_line(
                s,
                &format!(
                    "#import \"{}:{}\": cover-letter, recipient-entry",
                    typst_config.theme.source, typst_config.theme.version
                ),
            )
        })
        .pipe(|s| append_lines(s, "#import \"@preview/fontawesome:0.5.0\": *"))
}

/// Generate meta variable (matching grotesk-cv expected structure)
fn generate_meta(markup: String, personal_info: &PersonalInfo, typst_config: &TypstConfig) -> String {
    let (first_name, last_name) = split_name(&personal_info.name);
    let address = personal_info.location.as_deref().unwrap_or("");
    let phone = personal_info.phone.as_deref().unwrap_or("");

    markup
        .pipe(|s| append_line(s, "#let meta = ("))
        // Layout section
        .pipe(|s| append_line(s, "  layout: ("))
        .pipe(|s| append_line(s, &format!("    fill_color: \"{}\",", typst_config.customization.colors.fill)))
        .pipe(|s| append_line(s, &format!("    paper_size: \"{}\",", typst_config.customization.layout.paper_size)))
        .pipe(|s| append_line(s, &format!("    accent_color: \"{}\",", typst_config.customization.colors.accent)))
        .pipe(|s| append_line(s, "    text: ("))
        .pipe(|s| append_line(s, &format!("      font: \"{}\",", typst_config.customization.layout.font)))
        .pipe(|s| append_line(s, "      size: \"11pt\","))
        .pipe(|s| append_line(s, "      cover_letter_size: \"11pt\","))
        .pipe(|s| append_line(s, "      color: ("))
        .pipe(|s| append_line(s, &format!("        light: \"{}\",", typst_config.customization.colors.text_light)))
        .pipe(|s| append_line(s, &format!("        medium: \"{}\",", typst_config.customization.colors.text_medium)))
        .pipe(|s| append_line(s, &format!("        dark: \"{}\"", typst_config.customization.colors.text_dark)))
        .pipe(|s| append_line(s, "      )"))
        .pipe(|s| append_line(s, "    )"))
        .pipe(|s| append_line(s, "  ),"))
        // Language section (required by cover-letter)
        .pipe(|s| append_line(s, "  language: ("))
        .pipe(|s| append_line(s, "    en: ("))
        .pipe(|s| append_line(s, &format!("      subtitle: \"{}\",", personal_info.title)))
        .pipe(|s| append_line(s, "      cover_letter_document_name: \"Cover Letter\""))
        .pipe(|s| append_line(s, "    )"))
        .pipe(|s| append_line(s, "  ),"))
        // Personal section
        .pipe(|s| append_line(s, "  personal: ("))
        .pipe(|s| append_line(s, &format!("    first_name: \"{}\",", first_name)))
        .pipe(|s| append_line(s, &format!("    last_name: \"{}\",", last_name)))
        .pipe(|s| append_line(s, "    language: \"en\","))
        .pipe(|s| append_line(s, "    include_icons: true,"))
        .pipe(|s| append_line(s, "    info: ("))
        .pipe(|s| append_line(s, &format!("      address: \"{}\",", address)))
        .pipe(|s| append_line(s, &format!("      telephone: \"{}\",", phone)))
        .pipe(|s| append_line(s, "      email: ("))
        .pipe(|s| append_line(s, &format!("        link: \"mailto:{}\",", personal_info.email)))
        .pipe(|s| append_line(s, &format!("        label: \"{}\"", personal_info.email)))
        .pipe(|s| append_line(s, "      ),"))
        .pipe(|s| {
            // Add homepage if website is present
            if let Some(website) = &personal_info.website {
                let label = website
                    .trim_start_matches("https://")
                    .trim_start_matches("http://");
                append_line(s, "      homepage: (")
                    .pipe(|s| append_line(s, &format!("        link: \"{}\",", website)))
                    .pipe(|s| append_line(s, &format!("        label: \"{}\"", label)))
                    .pipe(|s| append_line(s, "      )"))
            } else {
                s
            }
        })
        .pipe(|s| append_line(s, "    ),"))
        .pipe(|s| append_line(s, "    icon: ("))
        .pipe(|s| append_line(s, "      address: \"house\","))
        .pipe(|s| append_line(s, "      telephone: \"phone\","))
        .pipe(|s| {
            // Add email icon, with comma only if homepage follows
            if personal_info.website.is_some() {
                append_line(s, "      email: \"envelope\",")
                    .pipe(|s| append_line(s, "      homepage: \"globe\""))
            } else {
                append_line(s, "      email: \"envelope\"")
            }
        })
        .pipe(|s| append_line(s, "    )"))   // close icon
        .pipe(|s| append_line(s, "  )"))     // close personal
        .pipe(|s| append_lines(s, ")"))      // close meta
}

/// Generate document setup and apply cover-letter template
fn generate_document_setup(markup: String, personal_info: &PersonalInfo) -> String {
    let (first_name, last_name) = split_name(&personal_info.name);

    markup
        .pipe(|s| {
            append_line(
                s,
                &format!(
                    "#set document(title: \"Cover Letter - {} {}\", author: \"{} {}\")",
                    first_name, last_name, first_name, last_name
                ),
            )
        })
        .pipe(|s| append_lines(s, "#show: cover-letter.with(meta)"))
}

/// Generate the letter body with optional recipient entry and proper paragraphs
fn generate_body(markup: String, letter: &CoverLetter) -> String {
    let fm = &letter.frontmatter;

    // Add recipient entry if we have any recipient info
    let has_recipient_info = fm.recipient_name.is_some()
        || fm.recipient_title.is_some()
        || fm.company.is_some()
        || fm.address.is_some();

    let markup = if has_recipient_info {
        let recipient_name = fm.recipient_name.as_deref().unwrap_or("");
        let recipient_title = fm.recipient_title.as_deref().unwrap_or("");
        let company = fm.company.as_deref().unwrap_or("");
        let address = fm.address.as_deref().unwrap_or("");

        markup
            .pipe(|s| append_line(s, "#recipient-entry("))
            .pipe(|s| append_line(s, &format!("  name: [{}],", recipient_name)))
            .pipe(|s| append_line(s, &format!("  title: [{}],", recipient_title)))
            .pipe(|s| append_line(s, &format!("  company: [{}],", company)))
            .pipe(|s| append_line(s, &format!("  address: [{}],", address)))
            .pipe(|s| append_lines(s, ")"))
    } else {
        markup
    };

    // Add date
    let markup = markup.pipe(|s| {
        append_lines(
            s,
            "#datetime.today().display(\"[day]/[month]/[year]\")",
        )
    });

    // Add spacing before body
    let markup = markup.pipe(|s| append_lines(s, "#v(1em)"));

    // Split body into paragraphs and format each one
    // Double newlines indicate paragraph breaks
    let paragraphs: Vec<&str> = letter
        .body
        .split("\n\n")
        .map(|p| p.trim())
        .filter(|p| !p.is_empty())
        .collect();

    let mut result = markup;
    let mut in_list = false;

    for para in paragraphs {
        // Detect closing signature and add spacing before it
        let is_closing = is_closing_signature(para);
        if is_closing {
            if in_list {
                result = result.pipe(|s| append_lines(s, ")"));
                in_list = false;
            }
            result = result.pipe(|s| append_lines(s, "#v(1.5em)"));
        }

        // Check if this is a bullet point (starts with "- ")
        if para.starts_with("- ") {
            let item_text = &para[2..]; // Remove "- " prefix
            let escaped = escape_typst(item_text);
            let normalized = escaped.replace('\n', " ");

            if !in_list {
                result = result.pipe(|s| append_line(s, "#list(tight: false, indent: 0.5em,"));
                in_list = true;
            }
            result = result.pipe(|s| append_line(s, &format!("  [{}],", normalized)));
        } else {
            // Close any open list
            if in_list {
                result = result.pipe(|s| append_lines(s, ")"));
                in_list = false;
            }

            // Escape special Typst characters
            let escaped = escape_typst(para);
            // Replace single newlines with spaces (within paragraph)
            let normalized = escaped.replace('\n', " ");
            result = result
                .pipe(|s| append_line(s, "#par(justify: true)["))
                .pipe(|s| append_line(s, &normalized))
                .pipe(|s| append_lines(s, "]"));
        }
    }

    // Close any remaining open list
    if in_list {
        result = result.pipe(|s| append_lines(s, ")"));
    }

    result
}

/// Escape special Typst characters in text
fn escape_typst(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace('#', "\\#")
        .replace('$', "\\$")
        .replace('[', "\\[")
        .replace(']', "\\]")
        .replace('<', "\\<")
        .replace('>', "\\>")
        .replace('@', "\\@")
}

/// Check if a paragraph is a closing signature (e.g., "Best regards, Name")
fn is_closing_signature(text: &str) -> bool {
    let lower = text.to_lowercase();
    let closings = [
        "best regards",
        "kind regards",
        "sincerely",
        "yours sincerely",
        "yours truly",
        "regards,",
        "best,",
        "thanks,",
        "thank you,",
        "cheers,",
    ];
    closings.iter().any(|c| lower.starts_with(c))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_personal_info() -> PersonalInfo {
        PersonalInfo {
            name: "John Doe".to_string(),
            title: "Software Engineer".to_string(),
            email: "john@example.com".to_string(),
            phone: Some("+1234567890".to_string()),
            website: None,
            location: Some("San Francisco, CA".to_string()),
            summary: "Test summary".to_string(),
            social_links: im::HashMap::new(),
            profile_image: None,
            github_avatar_url: None,
        }
    }

    fn create_test_typst_config() -> TypstConfig {
        use crate::site_config::{TypstColors, TypstCustomization, TypstLayout, TypstTheme};
        TypstConfig {
            theme: TypstTheme {
                name: "grotesk-cv".to_string(),
                version: "1.0.5".to_string(),
                source: "@preview/grotesk-cv".to_string(),
            },
            customization: TypstCustomization {
                colors: TypstColors {
                    fill: "#f4f1eb".to_string(),
                    accent: "#d4d2cc".to_string(),
                    text_light: "#ededef".to_string(),
                    text_medium: "#78787e".to_string(),
                    text_dark: "#3c3c42".to_string(),
                },
                layout: TypstLayout {
                    paper_size: "a4".to_string(),
                    left_pane_width: "71%".to_string(),
                    font: "Hanken Grotesk".to_string(),
                    font_size: "9pt".to_string(),
                },
            },
        }
    }

    #[test]
    fn test_generate_markup_structure() {
        use crate::cover_letter::CoverLetterFrontMatter;
        let letter = CoverLetter {
            frontmatter: CoverLetterFrontMatter::default(),
            body: "Dear Hiring Manager,\n\nTest body.".to_string(),
        };
        let personal_info = create_test_personal_info();
        let typst_config = create_test_typst_config();

        let markup = generate_cover_letter_markup(&letter, &personal_info, &typst_config);

        assert!(markup.contains("#import \"@preview/grotesk-cv:1.0.5\": cover-letter, recipient-entry"));
        assert!(markup.contains("#show: cover-letter.with(meta)"));
        assert!(markup.contains("first_name: \"John\""));
        assert!(markup.contains("last_name: \"Doe\""));
        assert!(markup.contains("#par(justify: true)"));
    }

    #[test]
    fn test_escape_typst() {
        let input = "Hello #world @test";
        let escaped = escape_typst(input);
        assert_eq!(escaped, "Hello \\#world \\@test");
    }
}
