use super::utils::{append_line, append_lines, join_strings};
/// Functions for generating Typst markup for different CV sections
use crate::cv_data::{Cv, Education, Experience, Project, SkillCategory};

/// Generates Typst markup for the summary section
///
/// # Arguments
///
/// * `cv` - The CV data
///
/// # Returns
///
/// The Typst markup for the summary section
pub fn generate_summary_section(cv: &Cv) -> String {
    let mut markup = String::new();

    markup = append_line(markup, "= #fa-icon(section_icons.profile) #h(5pt) Summary");
    markup = append_line(markup, "#v(5pt)");
    markup = append_lines(markup, &cv.personal_info.summary);

    markup
}

/// Generates Typst markup for the experience section
///
/// # Arguments
///
/// * `cv` - The CV data
///
/// # Returns
///
/// The Typst markup for the experience section
pub fn generate_experience_section(cv: &Cv) -> String {
    let mut markup = String::new();

    markup = append_line(
        markup,
        "= #fa-icon(section_icons.experience) #h(5pt) Experience",
    );
    markup = append_lines(markup, "== Professional Experience");

    if !cv.experiences.is_empty() {
        for exp in &cv.experiences {
            markup = append_experience_entry(markup, exp);
        }
    }

    markup
}

/// Generates Typst markup for a single experience entry
///
/// # Arguments
///
/// * `markup` - The current Typst markup
/// * `exp` - The experience entry
///
/// # Returns
///
/// The updated Typst markup
fn append_experience_entry(markup: String, exp: &Experience) -> String {
    let mut result = markup;

    // Position
    result = append_line(result, &format!("=== {}", exp.position));

    // Company and date range
    let mut company_line = format!("*{}* | {}", exp.company, exp.start_date);
    company_line.push_str(" – ");
    if let Some(end_date) = &exp.end_date {
        company_line.push_str(end_date);
    } else {
        company_line.push_str("Present");
    }
    result = append_lines(result, &company_line);

    // Description
    if !exp.description.is_empty() {
        result = append_lines(result, &exp.description);
    }

    // Achievements
    if !exp.achievements.is_empty() {
        for achievement in &exp.achievements {
            result = append_line(result, &format!("- {}", achievement));
        }
        result = append_line(result, "");
    }

    // Technologies
    if !exp.technologies.is_empty() {
        result = append_lines(
            result,
            &format!("*Technologies:* {}", join_strings(&exp.technologies, ", ")),
        );
    }

    result
}

/// Generates Typst markup for the projects section
///
/// # Arguments
///
/// * `cv` - The CV data
///
/// # Returns
///
/// The Typst markup for the projects section
pub fn generate_projects_section(cv: &Cv) -> String {
    let mut markup = String::new();

    if !cv.projects.is_empty() {
        markup = append_lines(markup, "= Projects");

        for project in &cv.projects {
            markup = append_project_entry(markup, project);
        }
    }

    markup
}

/// Generates Typst markup for a single project entry
///
/// # Arguments
///
/// * `markup` - The current Typst markup
/// * `project` - The project entry
///
/// # Returns
///
/// The updated Typst markup
fn append_project_entry(markup: String, project: &Project) -> String {
    let mut result = markup;

    // Project name and links
    let mut name_line = format!("== {}", project.name);

    if let Some(url) = &project.url {
        name_line.push_str(&format!(" #link(\"{}\")[Link]", url));
    }

    if let Some(repo) = &project.repository {
        name_line.push_str(&format!(" #link(\"{}\")[Repository]", repo));
    }

    result = append_lines(result, &name_line);

    // Description
    result = append_lines(result, &project.description);

    // Highlights
    if !project.highlights.is_empty() {
        for highlight in &project.highlights {
            result = append_line(result, &format!("- {}", highlight));
        }
        result = append_line(result, "");
    }

    // Technologies
    if !project.technologies.is_empty() {
        result = append_lines(
            result,
            &format!(
                "*Technologies:* {}",
                join_strings(&project.technologies, ", ")
            ),
        );
    }

    result
}

/// Generates Typst markup for the skills section
///
/// # Arguments
///
/// * `cv` - The CV data
///
/// # Returns
///
/// The Typst markup for the skills section
pub fn generate_skills_section(cv: &Cv) -> String {
    let mut markup = String::new();

    if !cv.skill_categories.is_empty() {
        markup = append_line(markup, "= #fa-icon(section_icons.skills) #h(5pt) Skills");
        markup = append_lines(markup, "== Key Skills");

        for category in &cv.skill_categories {
            markup = append_skill_category(markup, category);
        }
    }

    markup
}

/// Generates Typst markup for a single skill category
///
/// # Arguments
///
/// * `markup` - The current Typst markup
/// * `category` - The skill category
///
/// # Returns
///
/// The updated Typst markup
fn append_skill_category(markup: String, category: &SkillCategory) -> String {
    let mut result = markup;

    result = append_lines(result, &format!("=== {}", category.name));

    if !category.skills.is_empty() {
        for skill in &category.skills {
            result = append_line(result, &format!("- {}", skill));
        }
        result = append_line(result, "");
    }

    result
}

/// Generates Typst markup for the languages section
///
/// # Arguments
///
/// * `cv` - The CV data
///
/// # Returns
///
/// The Typst markup for the languages section
pub fn generate_languages_section(cv: &Cv) -> String {
    let mut markup = String::new();

    if !cv.languages.is_empty() {
        markup = append_line(
            markup,
            "= #fa-icon(section_icons.languages) #h(5pt) Languages",
        );
        markup = append_lines(markup, "#v(5pt)");

        for (language, proficiency) in &cv.languages {
            markup = append_lines(markup, &format!("*{}:* {}", language, proficiency));
        }
    }

    markup
}

/// Generates Typst markup for the education section
///
/// # Arguments
///
/// * `cv` - The CV data
///
/// # Returns
///
/// The Typst markup for the education section
pub fn generate_education_section(cv: &Cv) -> String {
    let mut markup = String::new();

    if !cv.education.is_empty() {
        markup = append_line(
            markup,
            "= #fa-icon(section_icons.education) #h(5pt) Education",
        );
        markup = append_lines(markup, "== Education");

        for edu in &cv.education {
            markup = append_education_entry(markup, edu);
        }
    }

    // Certifications Section
    if !cv.certifications.is_empty() {
        markup = append_lines(markup, "== Certificates");

        for certification in &cv.certifications {
            markup = append_line(markup, &format!("- *{}*", certification));
        }
        markup = append_line(markup, "");
    }

    markup
}

/// Generates Typst markup for a single education entry
///
/// # Arguments
///
/// * `markup` - The current Typst markup
/// * `edu` - The education entry
///
/// # Returns
///
/// The updated Typst markup
fn append_education_entry(markup: String, edu: &Education) -> String {
    let mut result = markup;

    // Degree and field
    result = append_line(result, &format!("=== {} in {}", edu.degree, edu.field));

    // Institution and date range
    let mut institution_line = edu.institution.clone();
    institution_line.push(' ');
    institution_line.push_str(&edu.start_date);

    if let Some(end_date) = &edu.end_date {
        institution_line.push('-');
        institution_line.push_str(end_date);
    }

    result = append_lines(result, &institution_line);

    // GPA
    if let Some(gpa) = &edu.gpa {
        result = append_lines(result, &format!("GPA: {}", gpa));
    }

    // Achievements
    if !edu.achievements.is_empty() {
        for achievement in &edu.achievements {
            result = append_line(result, &format!("- {}", achievement));
        }
        result = append_line(result, "");
    }

    result
}
