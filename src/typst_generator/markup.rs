use super::sections::{
    generate_education_section, generate_experience_section, generate_languages_section,
    generate_projects_section, generate_skills_section, generate_summary_section,
};
use super::utils::{append_line, append_lines, append_markup, format_email_for_typst, split_name};
/// Functions for generating complete Typst markup from CV data
use crate::cv_data::Cv;

// Extension trait to enable method chaining with pipe
trait Pipe: Sized {
    fn pipe<F, R>(self, f: F) -> R
    where
        F: FnOnce(Self) -> R,
    {
        f(self)
    }
}

// Implement Pipe for String to enable method chaining
impl Pipe for String {}

/// Generates complete Typst markup from CV data
///
/// # Arguments
///
/// * `cv` - The CV data
///
/// # Returns
///
/// The complete Typst markup
pub fn generate_typst_markup(cv: &Cv) -> String {
    String::new()
        .pipe(|s| append_markup(s, &generate_imports()))
        .pipe(|s| append_markup(s, &generate_layout_settings()))
        .pipe(|s| append_markup(s, &generate_text_settings()))
        .pipe(|s| append_markup(s, &generate_section_icons()))
        .pipe(|s| append_markup(s, &generate_personal_info_icons()))
        .pipe(|s| append_markup(s, &generate_personal_info(cv)))
        .pipe(|s| append_markup(s, &generate_left_pane(cv)))
        .pipe(|s| append_markup(s, &generate_right_pane(cv)))
        .pipe(|s| append_markup(s, &generate_final_template(cv)))
}

/// Generates Typst imports and setup
///
/// # Returns
///
/// The Typst markup for imports and setup
fn generate_imports() -> String {
    String::new()
        .pipe(|s| append_line(s, "#let meta = ()"))
        .pipe(|s| append_line(s, "#import \"@preview/grotesk-cv:1.0.2\": cv"))
        .pipe(|s| append_lines(s, "#import \"@preview/fontawesome:0.5.0\": *"))
}

/// Generates Typst layout settings
///
/// # Returns
///
/// The Typst markup for layout settings
fn generate_layout_settings() -> String {
    String::new()
        .pipe(|s| append_line(s, "#let fill_color = \"#f4f1eb\""))
        .pipe(|s| append_line(s, "#let paper_size = \"a4\""))
        .pipe(|s| append_line(s, "#let accent_color = \"#d4d2cc\""))
        .pipe(|s| append_lines(s, "#let left_pane_width = \"71%\""))
}

/// Generates Typst text settings
///
/// # Returns
///
/// The Typst markup for text settings
fn generate_text_settings() -> String {
    String::new()
        .pipe(|s| append_line(s, "#let font = \"HK Grotesk\""))
        .pipe(|s| append_line(s, "#let size = \"9pt\""))
        .pipe(|s| append_line(s, "#let text_color_light = \"#ededef\""))
        .pipe(|s| append_line(s, "#let text_color_medium = \"#78787e\""))
        .pipe(|s| append_lines(s, "#let text_color_dark = \"#3c3c42\""))
}

/// Generates Typst section icons
///
/// # Returns
///
/// The Typst markup for section icons
fn generate_section_icons() -> String {
    String::new()
        .pipe(|s| append_line(s, "#let section_icons = ("))
        .pipe(|s| append_markup(s, "education: \"graduation-cap\", "))
        .pipe(|s| append_markup(s, "experience: \"briefcase\", "))
        .pipe(|s| append_markup(s, "languages: \"language\", "))
        .pipe(|s| append_markup(s, "profile: \"id-card\", "))
        .pipe(|s| append_markup(s, "skills: \"cogs\""))
        .pipe(|s| append_lines(s, ")"))
}

/// Generates Typst personal info icons
///
/// # Returns
///
/// The Typst markup for personal info icons
fn generate_personal_info_icons() -> String {
    String::new()
        .pipe(|s| append_line(s, "#let personal_icons = ("))
        .pipe(|s| append_markup(s, "address: \"house\", "))
        .pipe(|s| append_markup(s, "telephone: \"phone\", "))
        .pipe(|s| append_markup(s, "email: \"envelope\", "))
        .pipe(|s| append_markup(s, "linkedin: \"linkedin\", "))
        .pipe(|s| append_markup(s, "github: \"github\", "))
        .pipe(|s| append_markup(s, "homepage: \"globe\""))
        .pipe(|s| append_lines(s, ")"))
}

/// Generates Typst personal info
///
/// # Arguments
///
/// * `cv` - The CV data
///
/// # Returns
///
/// The Typst markup for personal info
fn generate_personal_info(cv: &Cv) -> String {
    // Name
    let (first_name, last_name) = split_name(&cv.personal_info.name);

    // Email
    let email_display = format_email_for_typst(&cv.personal_info.email);

    // Start building the markup
    let base = String::new()
        .pipe(|s| append_line(s, "#let personal_info = ("))
        .pipe(|s| append_markup(s, &format!("first_name: \"{first_name}\", ")))
        .pipe(|s| append_markup(s, &format!("last_name: \"{last_name}\", ")))
        .pipe(|s| append_markup(s, &format!("email: \"{email_display}\", ")));

    // Phone
    let with_phone = if let Some(phone) = &cv.personal_info.phone {
        base.pipe(|s| append_markup(s, &format!("telephone: \"{phone}\", ")))
    } else {
        base
    };

    // Location
    let with_location = if let Some(location) = &cv.personal_info.location {
        with_phone.pipe(|s| append_markup(s, &format!("address: \"{location}\", ")))
    } else {
        with_phone
    };

    // Title/subtitle
    let with_title = with_location
        .pipe(|s| append_markup(s, &format!("subtitle: \"{}", cv.personal_info.title)));

    // Add focus if summary is not empty
    let with_focus = if !cv.personal_info.summary.is_empty() {
        with_title.pipe(|s| {
            append_markup(
                s,
                " with a focus on functional programming and scalable solutions",
            )
        })
    } else {
        with_title
    };

    // Close the subtitle and the personal_info object
    with_focus
        .pipe(|s| append_markup(s, "\""))
        .pipe(|s| append_lines(s, ")"))
}

/// Generates Typst left pane content
///
/// # Arguments
///
/// * `cv` - The CV data
///
/// # Returns
///
/// The Typst markup for the left pane
fn generate_left_pane(cv: &Cv) -> String {
    String::new()
        .pipe(|s| append_line(s, "#let left_pane = ["))
        // Summary section
        .pipe(|s| append_markup(s, &generate_summary_section(cv)))
        // Experience section
        .pipe(|s| append_markup(s, &generate_experience_section(cv)))
        // Projects section
        .pipe(|s| append_markup(s, &generate_projects_section(cv)))
        // End of left pane
        .pipe(|s| append_lines(s, "]"))
}

/// Generates Typst right pane content
///
/// # Arguments
///
/// * `cv` - The CV data
///
/// # Returns
///
/// The Typst markup for the right pane
fn generate_right_pane(cv: &Cv) -> String {
    String::new()
        .pipe(|s| append_line(s, "#let right_pane = ["))
        // Skills section
        .pipe(|s| append_markup(s, &generate_skills_section(cv)))
        // Languages section
        .pipe(|s| append_markup(s, &generate_languages_section(cv)))
        // Education section
        .pipe(|s| append_markup(s, &generate_education_section(cv)))
        // End of right pane
        .pipe(|s| append_lines(s, "]"))
}

/// Generates the final Typst template
///
/// # Arguments
///
/// * `cv` - The CV data
///
/// # Returns
///
/// The Typst markup for the final template
fn generate_final_template(cv: &Cv) -> String {
    // Format document title and author
    let document_settings = format!(
        "#set document(title: \"{} - CV\", author: \"{}\")",
        cv.personal_info.name, cv.personal_info.name
    );

    String::new()
        // Photo and document settings
        .pipe(|s| append_line(s, "#let photo = image(\"static/img/profile_picture.jpg\")"))
        .pipe(|s| append_line(s, &document_settings))
        // Define meta variable with the appropriate structure
        .pipe(|s| append_markup(s, &generate_meta_variable(cv)))
        // Show the CV with the template
        .pipe(|s| append_line(s, "#show: cv.with("))
        .pipe(|s| append_line(s, "  meta,"))
        .pipe(|s| append_line(s, "  photo: photo,"))
        .pipe(|s| append_line(s, "  use-photo: true,"))
        .pipe(|s| append_line(s, "  left-pane: left_pane,"))
        .pipe(|s| append_line(s, "  right-pane: right_pane,"))
        .pipe(|s| append_line(s, "  left-pane-proportion: 71%,"))
        .pipe(|s| append_line(s, ")"))
}

/// Generates the meta variable for the Typst template
///
/// # Arguments
///
/// * `cv` - The CV data
///
/// # Returns
///
/// The Typst markup for the meta variable
fn generate_meta_variable(cv: &Cv) -> String {
    // Extract name components
    let (first_name, last_name) = split_name(&cv.personal_info.name);

    // Create subtitle with optional focus
    let subtitle = {
        let base = cv.personal_info.title.clone();
        if !cv.personal_info.summary.is_empty() {
            format!("{base} with a focus on functional programming and scalable solutions")
        } else {
            base
        }
    };

    // Start building the markup
    let base = String::new()
        .pipe(|s| append_line(s, "#let meta = ("))
        // Layout section
        .pipe(|s| append_line(s, "  layout: ("))
        .pipe(|s| append_line(s, "    fill_color: \"#f4f1eb\","))
        .pipe(|s| append_line(s, "    paper_size: \"a4\","))
        .pipe(|s| append_line(s, "    accent_color: \"#d4d2cc\","))
        .pipe(|s| append_line(s, "    left_pane_width: \"71%\","))
        .pipe(|s| append_line(s, "    text: ("))
        .pipe(|s| append_line(s, "      font: \"HK Grotesk\","))
        .pipe(|s| append_line(s, "      size: \"9pt\","))
        .pipe(|s| append_line(s, "      color: ("))
        .pipe(|s| append_line(s, "        light: \"#ededef\","))
        .pipe(|s| append_line(s, "        medium: \"#78787e\","))
        .pipe(|s| append_line(s, "        dark: \"#3c3c42\""))
        .pipe(|s| append_line(s, "      )"))
        .pipe(|s| append_line(s, "    )"))
        .pipe(|s| append_line(s, "  ),"))
        // Language section
        .pipe(|s| append_line(s, "  language: ("))
        .pipe(|s| append_line(s, "    en: ("))
        .pipe(|s| append_line(s, "      subtitle: \"Data Engineer with a focus on functional programming and scalable solutions\""))
        .pipe(|s| append_line(s, "    ),"))
        .pipe(|s| append_line(s, "    dk: ("))
        .pipe(|s| append_line(s, "      subtitle: \"\""))
        .pipe(|s| append_line(s, "    )"))
        .pipe(|s| append_line(s, "  ),"))
        // Personal section
        .pipe(|s| append_line(s, "  personal: ("))
        // Name
        .pipe(|s| append_line(s, &format!("    first_name: \"{first_name}\",")))
        .pipe(|s| append_line(s, &format!("    last_name: \"{last_name}\",")))
        .pipe(|s| append_line(s, "    language: \"en\","))
        .pipe(|s| append_line(s, "    include_icons: true,"))
        .pipe(|s| append_line(s, "    profile_image: \"static/img/profile_picture.jpg\","))
        // Subtitle
        .pipe(|s| append_line(s, &format!("    subtitle: \"{subtitle}\",")))
        // Info section
        .pipe(|s| append_line(s, "    info: ("));

    // Address
    let address_value = cv.personal_info.location.as_deref().unwrap_or("");
    let with_address =
        base.pipe(|s| append_line(s, &format!("      address: \"{address_value}\",")));

    // Telephone
    let phone_value = cv.personal_info.phone.as_deref().unwrap_or("");
    let with_phone =
        with_address.pipe(|s| append_line(s, &format!("      telephone: \"{phone_value}\",")));

    // Email
    let with_email = with_phone
        .pipe(|s| append_line(s, "      email: ("))
        .pipe(|s| {
            append_line(
                s,
                &format!("        link: \"mailto:{}\",", cv.personal_info.email),
            )
        })
        .pipe(|s| append_line(s, &format!("        label: \"{}\"", cv.personal_info.email)))
        .pipe(|s| append_line(s, "      )"))
        .pipe(|s| append_line(s, "    ),"));

    // Complete the rest of the meta variable
    with_email
        // Icons
        .pipe(|s| append_line(s, "    icon: ("))
        .pipe(|s| append_line(s, "      address: \"house\","))
        .pipe(|s| append_line(s, "      telephone: \"phone\","))
        .pipe(|s| append_line(s, "      email: \"envelope\","))
        .pipe(|s| append_line(s, "      linkedin: \"linkedin\","))
        .pipe(|s| append_line(s, "      github: \"github\","))
        .pipe(|s| append_line(s, "      homepage: \"globe\""))
        .pipe(|s| append_line(s, "    )"))
        .pipe(|s| append_line(s, "  ),"))
        // Section icons
        .pipe(|s| append_line(s, "  section: ("))
        .pipe(|s| append_line(s, "    icon: ("))
        .pipe(|s| append_line(s, "      education: \"graduation-cap\","))
        .pipe(|s| append_line(s, "      experience: \"briefcase\","))
        .pipe(|s| append_line(s, "      languages: \"language\","))
        .pipe(|s| append_line(s, "      profile: \"id-card\","))
        .pipe(|s| append_line(s, "      skills: \"cogs\""))
        .pipe(|s| append_line(s, "    )"))
        .pipe(|s| append_line(s, "  ),"))
        // Imports
        .pipe(|s| append_line(s, "  imports: ("))
        .pipe(|s| append_line(s, "    path: \"@preview/grotesk-cv:1.0.2\","))
        .pipe(|s| append_line(s, "    fontawesome: \"@preview/fontawesome:0.5.0\""))
        .pipe(|s| append_line(s, "  )"))
        .pipe(|s| append_line(s, ")"))
}
