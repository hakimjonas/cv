use super::sections::{
    generate_education_section, generate_experience_section, generate_languages_section,
    generate_projects_section, generate_skills_section, generate_summary_section,
};
use super::utils::{append_line, append_lines, append_markup, format_email_for_typst, split_name};
/// Functions for generating complete Typst markup from CV data
use crate::cv_data::Cv;

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
    let mut markup = String::new();

    // Add imports and setup
    markup = append_markup(markup, &generate_imports());

    // Add layout settings
    markup = append_markup(markup, &generate_layout_settings());

    // Add text settings
    markup = append_markup(markup, &generate_text_settings());

    // Add section icons
    markup = append_markup(markup, &generate_section_icons());

    // Add personal info icons
    markup = append_markup(markup, &generate_personal_info_icons());

    // Add personal info
    markup = append_markup(markup, &generate_personal_info(cv));

    // Add left pane content
    markup = append_markup(markup, &generate_left_pane(cv));

    // Add right pane content
    markup = append_markup(markup, &generate_right_pane(cv));

    // Add final CV with template
    markup = append_markup(markup, &generate_final_template(cv));

    markup
}

/// Generates Typst imports and setup
///
/// # Returns
///
/// The Typst markup for imports and setup
fn generate_imports() -> String {
    let mut markup = String::new();

    markup = append_line(markup, "#let meta = ()");
    markup = append_line(markup, "#import \"@preview/grotesk-cv:1.0.2\": cv");
    markup = append_lines(markup, "#import \"@preview/fontawesome:0.5.0\": *");

    markup
}

/// Generates Typst layout settings
///
/// # Returns
///
/// The Typst markup for layout settings
fn generate_layout_settings() -> String {
    let mut markup = String::new();

    markup = append_line(markup, "#let fill_color = \"#f4f1eb\"");
    markup = append_line(markup, "#let paper_size = \"a4\"");
    markup = append_line(markup, "#let accent_color = \"#d4d2cc\"");
    markup = append_lines(markup, "#let left_pane_width = \"71%\"");

    markup
}

/// Generates Typst text settings
///
/// # Returns
///
/// The Typst markup for text settings
fn generate_text_settings() -> String {
    let mut markup = String::new();

    markup = append_line(markup, "#let font = \"HK Grotesk\"");
    markup = append_line(markup, "#let size = \"9pt\"");
    markup = append_line(markup, "#let text_color_light = \"#ededef\"");
    markup = append_line(markup, "#let text_color_medium = \"#78787e\"");
    markup = append_lines(markup, "#let text_color_dark = \"#3c3c42\"");

    markup
}

/// Generates Typst section icons
///
/// # Returns
///
/// The Typst markup for section icons
fn generate_section_icons() -> String {
    let mut markup = String::new();

    markup = append_line(markup, "#let section_icons = (");
    markup = append_markup(markup, "education: \"graduation-cap\", ");
    markup = append_markup(markup, "experience: \"briefcase\", ");
    markup = append_markup(markup, "languages: \"language\", ");
    markup = append_markup(markup, "profile: \"id-card\", ");
    markup = append_markup(markup, "skills: \"cogs\"");
    markup = append_lines(markup, ")");

    markup
}

/// Generates Typst personal info icons
///
/// # Returns
///
/// The Typst markup for personal info icons
fn generate_personal_info_icons() -> String {
    let mut markup = String::new();

    markup = append_line(markup, "#let personal_icons = (");
    markup = append_markup(markup, "address: \"house\", ");
    markup = append_markup(markup, "telephone: \"phone\", ");
    markup = append_markup(markup, "email: \"envelope\", ");
    markup = append_markup(markup, "linkedin: \"linkedin\", ");
    markup = append_markup(markup, "github: \"github\", ");
    markup = append_markup(markup, "homepage: \"globe\"");
    markup = append_lines(markup, ")");

    markup
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
    let mut markup = String::new();

    markup = append_line(markup, "#let personal_info = (");

    // Name
    let (first_name, last_name) = split_name(&cv.personal_info.name);
    markup = append_markup(markup, &format!("first_name: \"{}\", ", first_name));
    markup = append_markup(markup, &format!("last_name: \"{}\", ", last_name));

    // Email
    let email_display = format_email_for_typst(&cv.personal_info.email);
    markup = append_markup(markup, &format!("email: \"{}\", ", email_display));

    // Phone
    if let Some(phone) = &cv.personal_info.phone {
        markup = append_markup(markup, &format!("telephone: \"{}\", ", phone));
    }

    // Location
    if let Some(location) = &cv.personal_info.location {
        markup = append_markup(markup, &format!("address: \"{}\", ", location));
    }

    // Title/subtitle
    markup = append_markup(markup, &format!("subtitle: \"{}", cv.personal_info.title));
    if !cv.personal_info.summary.is_empty() {
        markup = append_markup(
            markup,
            " with a focus on functional programming and scalable solutions",
        );
    }
    markup = append_markup(markup, "\"");

    markup = append_lines(markup, ")");

    markup
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
    let mut markup = String::new();

    markup = append_line(markup, "#let left_pane = [");

    // Summary section
    markup = append_markup(markup, &generate_summary_section(cv));

    // Experience section
    markup = append_markup(markup, &generate_experience_section(cv));

    // Projects section
    markup = append_markup(markup, &generate_projects_section(cv));

    markup = append_lines(markup, "]"); // End of left pane

    markup
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
    let mut markup = String::new();

    markup = append_line(markup, "#let right_pane = [");

    // Skills section
    markup = append_markup(markup, &generate_skills_section(cv));

    // Languages section
    markup = append_markup(markup, &generate_languages_section(cv));

    // Education section
    markup = append_markup(markup, &generate_education_section(cv));

    markup = append_lines(markup, "]"); // End of right pane

    markup
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
    let mut markup = String::new();

    // Photo and document settings
    markup = append_line(
        markup,
        "#let photo = image(\"static/img/profile_picture.jpg\")",
    );
    markup = append_line(
        markup,
        &format!(
            "#set document(title: \"{} - CV\", author: \"{}\")",
            cv.personal_info.name, cv.personal_info.name
        ),
    );

    // Define meta variable with the appropriate structure
    markup = append_markup(markup, &generate_meta_variable(cv));

    // Show the CV with the template
    markup = append_line(markup, "#show: cv.with(");
    markup = append_line(markup, "  meta,");
    markup = append_line(markup, "  photo: photo,");
    markup = append_line(markup, "  use-photo: true,");
    markup = append_line(markup, "  left-pane: left_pane,");
    markup = append_line(markup, "  right-pane: right_pane,");
    markup = append_line(markup, "  left-pane-proportion: 71%,");
    markup = append_line(markup, ")");

    markup
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
    let mut markup = String::new();

    markup = append_line(markup, "#let meta = (");
    markup = append_line(markup, "  layout: (");
    markup = append_line(markup, "    fill_color: \"#f4f1eb\",");
    markup = append_line(markup, "    paper_size: \"a4\",");
    markup = append_line(markup, "    accent_color: \"#d4d2cc\",");
    markup = append_line(markup, "    left_pane_width: \"71%\",");
    markup = append_line(markup, "    text: (");
    markup = append_line(markup, "      font: \"HK Grotesk\",");
    markup = append_line(markup, "      size: \"9pt\",");
    markup = append_line(markup, "      color: (");
    markup = append_line(markup, "        light: \"#ededef\",");
    markup = append_line(markup, "        medium: \"#78787e\",");
    markup = append_line(markup, "        dark: \"#3c3c42\"");
    markup = append_line(markup, "      )");
    markup = append_line(markup, "    )");
    markup = append_line(markup, "  ),");
    markup = append_line(markup, "  language: (");
    markup = append_line(markup, "    en: (");
    markup = append_line(
        markup,
        "      subtitle: \"Data Engineer with a focus on functional programming and scalable solutions\"",
    );
    markup = append_line(markup, "    ),");
    markup = append_line(markup, "    dk: (");
    markup = append_line(markup, "      subtitle: \"\"");
    markup = append_line(markup, "    )");
    markup = append_line(markup, "  ),");
    markup = append_line(markup, "  personal: (");

    // Name
    let (first_name, last_name) = split_name(&cv.personal_info.name);
    markup = append_line(markup, &format!("    first_name: \"{}\",", first_name));
    markup = append_line(markup, &format!("    last_name: \"{}\",", last_name));

    markup = append_line(markup, "    language: \"en\",");
    markup = append_line(markup, "    include_icons: true,");
    markup = append_line(
        markup,
        "    profile_image: \"static/img/profile_picture.jpg\",",
    );

    // Subtitle
    markup = append_markup(markup, "    subtitle: \"");
    markup = append_markup(markup, &cv.personal_info.title);
    if !cv.personal_info.summary.is_empty() {
        markup = append_markup(
            markup,
            " with a focus on functional programming and scalable solutions",
        );
    }
    markup = append_line(markup, "\",");

    // Info
    markup = append_line(markup, "    info: (");

    // Address
    markup = append_markup(markup, "      address: \"");
    if let Some(location) = &cv.personal_info.location {
        markup = append_markup(markup, location);
    }
    markup = append_line(markup, "\",");

    // Telephone
    markup = append_markup(markup, "      telephone: \"");
    if let Some(phone) = &cv.personal_info.phone {
        markup = append_markup(markup, phone);
    }
    markup = append_line(markup, "\",");

    // Email
    markup = append_line(markup, "      email: (");
    markup = append_line(
        markup,
        &format!("        link: \"mailto:{}\",", cv.personal_info.email),
    );
    markup = append_line(
        markup,
        &format!("        label: \"{}\"", cv.personal_info.email),
    );
    markup = append_line(markup, "      )");
    markup = append_line(markup, "    ),");

    // Icons
    markup = append_line(markup, "    icon: (");
    markup = append_line(markup, "      address: \"house\",");
    markup = append_line(markup, "      telephone: \"phone\",");
    markup = append_line(markup, "      email: \"envelope\",");
    markup = append_line(markup, "      linkedin: \"linkedin\",");
    markup = append_line(markup, "      github: \"github\",");
    markup = append_line(markup, "      homepage: \"globe\"");
    markup = append_line(markup, "    )");
    markup = append_line(markup, "  ),");

    // Section icons
    markup = append_line(markup, "  section: (");
    markup = append_line(markup, "    icon: (");
    markup = append_line(markup, "      education: \"graduation-cap\",");
    markup = append_line(markup, "      experience: \"briefcase\",");
    markup = append_line(markup, "      languages: \"language\",");
    markup = append_line(markup, "      profile: \"id-card\",");
    markup = append_line(markup, "      skills: \"cogs\"");
    markup = append_line(markup, "    )");
    markup = append_line(markup, "  ),");

    // Imports
    markup = append_line(markup, "  imports: (");
    markup = append_line(markup, "    path: \"@preview/grotesk-cv:1.0.2\",");
    markup = append_line(markup, "    fontawesome: \"@preview/fontawesome:0.5.0\"");
    markup = append_line(markup, "  )");
    markup = append_line(markup, ")");

    markup
}
