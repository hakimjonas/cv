use super::utils::{append_line, append_lines, join_strings};
/// Functions for generating Typst markup for different CV sections
use crate::cv_data::{Cv, Education, Experience, Project, SkillCategory};
use im::Vector;

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
    String::new()
        .pipe(|s| append_line(s, "= #fa-icon(section_icons.profile) #h(5pt) Summary"))
        .pipe(|s| append_line(s, "#v(5pt)"))
        .pipe(|s| append_lines(s, &cv.personal_info.summary))
}

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

/// Appends a list of items as bullet points to the markup
///
/// # Arguments
///
/// * `markup` - The current Typst markup
/// * `items` - The list of items to append
/// * `format_fn` - A function to format each item
///
/// # Returns
///
/// The updated Typst markup with items as bullet points and an empty line at the end
fn append_bullet_list<T, F>(markup: String, items: &Vector<T>, format_fn: F) -> String
where
    T: Clone,
    F: Fn(&T) -> String,
{
    if items.is_empty() {
        return markup;
    }

    // Use fold to accumulate items with bullet points
    let with_items = items.iter().fold(markup, |acc, item| {
        append_line(acc, &format!("- {}", format_fn(item)))
    });

    // Add an empty line after the list
    append_line(with_items, "")
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
    let base = String::new()
        .pipe(|s| append_line(s, "= #fa-icon(section_icons.experience) #h(5pt) Experience"))
        .pipe(|s| append_lines(s, "== Professional Experience"));

    // Use fold to accumulate experience entries
    cv.experiences.iter().fold(base, append_experience_entry)
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
    // Position
    let with_position = markup.pipe(|s| append_line(s, &format!("=== {}", exp.position)));

    // Company and date range
    let company_line = format!(
        "*{}* | {} – {}",
        exp.company,
        exp.start_date,
        exp.end_date.as_deref().unwrap_or("Present")
    );
    let with_company = with_position.pipe(|s| append_lines(s, &company_line));

    // Description
    let with_description = if !exp.description.is_empty() {
        with_company.pipe(|s| append_lines(s, &exp.description))
    } else {
        with_company
    };

    // Achievements
    let with_achievements =
        append_bullet_list(with_description, &exp.achievements, |achievement| {
            achievement.clone()
        });

    // Technologies
    if !exp.technologies.is_empty() {
        with_achievements.pipe(|s| {
            append_lines(
                s,
                &format!("*Technologies:* {}", join_strings(&exp.technologies, ", ")),
            )
        })
    } else {
        with_achievements
    }
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
    if !cv.projects.is_empty() {
        // Start with the section header
        let base = String::new().pipe(|s| append_lines(s, "= Projects"));

        // Use fold to accumulate project entries
        cv.projects.iter().fold(base, append_project_entry)
    } else {
        String::new()
    }
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
    // Project name and links
    let name_line = {
        // Start with base name
        let base_name = format!("== {}", project.name);

        // Add URL link if available
        let with_url = project
            .url
            .as_ref()
            .map(|url| format!("{} #link(\"{}\")[Link]", base_name, url))
            .unwrap_or(base_name);

        // Add repository link if available
        project
            .repository
            .as_ref()
            .map(|repo| format!("{} #link(\"{}\")[Repository]", with_url, repo))
            .unwrap_or(with_url)
    };

    // Add name line and description
    let with_description = markup
        .pipe(|s| append_lines(s, &name_line))
        .pipe(|s| append_lines(s, &project.description));

    // Add highlights
    let with_highlights = append_bullet_list(with_description, &project.highlights, |highlight| {
        highlight.clone()
    });

    // Add technologies
    if !project.technologies.is_empty() {
        with_highlights.pipe(|s| {
            append_lines(
                s,
                &format!(
                    "*Technologies:* {}",
                    join_strings(&project.technologies, ", ")
                ),
            )
        })
    } else {
        with_highlights
    }
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
    if !cv.skill_categories.is_empty() {
        // Start with the section header
        let base = String::new()
            .pipe(|s| append_line(s, "= #fa-icon(section_icons.skills) #h(5pt) Skills"))
            .pipe(|s| append_lines(s, "== Key Skills"));

        // Use fold to accumulate skill categories
        cv.skill_categories.iter().fold(base, append_skill_category)
    } else {
        String::new()
    }
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
    // Add category name
    let with_name = markup.pipe(|s| append_lines(s, &format!("=== {}", category.name)));

    // Add skills
    append_bullet_list(with_name, &category.skills, |skill| skill.clone())
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
    if !cv.languages.is_empty() {
        // Start with the section header
        let base = String::new()
            .pipe(|s| append_line(s, "= #fa-icon(section_icons.languages) #h(5pt) Languages"))
            .pipe(|s| append_lines(s, "#v(5pt)"));

        // Use fold to accumulate language entries
        cv.languages
            .iter()
            .fold(base, |acc, (language, proficiency)| {
                append_lines(acc, &format!("*{}:* {}", language, proficiency))
            })
    } else {
        String::new()
    }
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
    // Start with an empty string
    let base = String::new();

    // Add education section if not empty
    let with_education = if !cv.education.is_empty() {
        // Add section header
        let header = base
            .pipe(|s| append_line(s, "= #fa-icon(section_icons.education) #h(5pt) Education"))
            .pipe(|s| append_lines(s, "== Education"));

        // Use fold to accumulate education entries
        cv.education.iter().fold(header, append_education_entry)
    } else {
        base
    };

    // Add certifications section if not empty
    if !cv.certifications.is_empty() {
        // Add section header
        let with_cert_header = with_education.pipe(|s| append_lines(s, "== Certificates"));

        // Use fold to accumulate certifications
        let with_certs = cv
            .certifications
            .iter()
            .fold(with_cert_header, |acc, certification| {
                append_line(acc, &format!("- *{}*", certification))
            });

        // Add an empty line after certifications
        append_line(with_certs, "")
    } else {
        with_education
    }
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
    // Degree and field
    let with_degree =
        markup.pipe(|s| append_line(s, &format!("=== {} in {}", edu.degree, edu.field)));

    // Institution and date range
    let institution_line = {
        let base = format!("{} {}", edu.institution, edu.start_date);
        if let Some(end_date) = &edu.end_date {
            format!("{}-{}", base, end_date)
        } else {
            base
        }
    };

    let with_institution = with_degree.pipe(|s| append_lines(s, &institution_line));

    // GPA
    let with_gpa = if let Some(gpa) = &edu.gpa {
        with_institution.pipe(|s| append_lines(s, &format!("GPA: {}", gpa)))
    } else {
        with_institution
    };

    // Achievements
    append_bullet_list(with_gpa, &edu.achievements, |achievement| {
        achievement.clone()
    })
}
