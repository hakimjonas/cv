use anyhow::{Context, Result};
use rusqlite::{Connection, params};
use std::path::Path;

use crate::cv_data::{
    Cv, Education, Experience, GitHubSource, PersonalInfo, Project, SkillCategory,
};

/// Database module for the CV generator
///
/// This module provides functions for creating, reading, updating, and deleting
/// CV data in a SQLite database.
pub struct Database {
    conn: Connection,
}

impl Database {
    /// Creates a new database connection
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the database file
    ///
    /// # Returns
    ///
    /// A Result containing the Database or an error
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path).context("Failed to open database")?;
        Ok(Self { conn })
    }

    /// Creates the database schema
    ///
    /// # Returns
    ///
    /// A Result indicating success or failure
    pub fn create_schema(&self) -> Result<()> {
        // Create personal_info table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS personal_info (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                title TEXT NOT NULL,
                email TEXT NOT NULL,
                phone TEXT,
                website TEXT,
                location TEXT,
                summary TEXT NOT NULL
            )",
            [],
        )?;

        // Create social_links table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS social_links (
                id INTEGER PRIMARY KEY,
                personal_info_id INTEGER NOT NULL,
                platform TEXT NOT NULL,
                url TEXT NOT NULL,
                FOREIGN KEY (personal_info_id) REFERENCES personal_info (id)
            )",
            [],
        )?;

        // Create experiences table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS experiences (
                id INTEGER PRIMARY KEY,
                company TEXT NOT NULL,
                position TEXT NOT NULL,
                start_date TEXT NOT NULL,
                end_date TEXT,
                location TEXT,
                description TEXT NOT NULL
            )",
            [],
        )?;

        // Create experience_achievements table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS experience_achievements (
                id INTEGER PRIMARY KEY,
                experience_id INTEGER NOT NULL,
                achievement TEXT NOT NULL,
                FOREIGN KEY (experience_id) REFERENCES experiences (id)
            )",
            [],
        )?;

        // Create experience_technologies table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS experience_technologies (
                id INTEGER PRIMARY KEY,
                experience_id INTEGER NOT NULL,
                technology TEXT NOT NULL,
                FOREIGN KEY (experience_id) REFERENCES experiences (id)
            )",
            [],
        )?;

        // Create education table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS education (
                id INTEGER PRIMARY KEY,
                institution TEXT NOT NULL,
                degree TEXT NOT NULL,
                field TEXT NOT NULL,
                start_date TEXT NOT NULL,
                end_date TEXT,
                location TEXT,
                gpa TEXT
            )",
            [],
        )?;

        // Create education_achievements table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS education_achievements (
                id INTEGER PRIMARY KEY,
                education_id INTEGER NOT NULL,
                achievement TEXT NOT NULL,
                FOREIGN KEY (education_id) REFERENCES education (id)
            )",
            [],
        )?;

        // Create skill_categories table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS skill_categories (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL
            )",
            [],
        )?;

        // Create skills table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS skills (
                id INTEGER PRIMARY KEY,
                category_id INTEGER NOT NULL,
                skill TEXT NOT NULL,
                FOREIGN KEY (category_id) REFERENCES skill_categories (id)
            )",
            [],
        )?;

        // Create projects table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS projects (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT NOT NULL,
                url TEXT,
                repository TEXT,
                stars INTEGER,
                owner_username TEXT,
                owner_avatar TEXT
            )",
            [],
        )?;

        // Create project_technologies table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS project_technologies (
                id INTEGER PRIMARY KEY,
                project_id INTEGER NOT NULL,
                technology TEXT NOT NULL,
                FOREIGN KEY (project_id) REFERENCES projects (id)
            )",
            [],
        )?;

        // Create project_highlights table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS project_highlights (
                id INTEGER PRIMARY KEY,
                project_id INTEGER NOT NULL,
                highlight TEXT NOT NULL,
                FOREIGN KEY (project_id) REFERENCES projects (id)
            )",
            [],
        )?;

        // Create languages table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS languages (
                id INTEGER PRIMARY KEY,
                language TEXT NOT NULL,
                proficiency TEXT NOT NULL
            )",
            [],
        )?;

        // Create certifications table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS certifications (
                id INTEGER PRIMARY KEY,
                certification TEXT NOT NULL
            )",
            [],
        )?;

        // Create github_sources table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS github_sources (
                id INTEGER PRIMARY KEY,
                username TEXT,
                organization TEXT
            )",
            [],
        )?;

        Ok(())
    }

    /// Inserts CV data into the database
    ///
    /// # Arguments
    ///
    /// * `cv` - The CV data to insert
    ///
    /// # Returns
    ///
    /// A Result indicating success or failure
    pub fn insert_cv(&mut self, cv: &Cv) -> Result<()> {
        // Start a transaction
        let tx = self.conn.transaction()?;

        // Insert personal_info
        tx.execute(
            "INSERT INTO personal_info (name, title, email, phone, website, location, summary)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                cv.personal_info.name,
                cv.personal_info.title,
                cv.personal_info.email,
                cv.personal_info.phone,
                cv.personal_info.website,
                cv.personal_info.location,
                cv.personal_info.summary,
            ],
        )?;

        let personal_info_id = tx.last_insert_rowid();

        // Insert social_links
        for (platform, url) in &cv.personal_info.social_links {
            tx.execute(
                "INSERT INTO social_links (personal_info_id, platform, url)
                 VALUES (?1, ?2, ?3)",
                params![personal_info_id, platform, url],
            )?;
        }

        // Insert experiences
        for exp in &cv.experiences {
            tx.execute(
                "INSERT INTO experiences (company, position, start_date, end_date, location, description)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    exp.company,
                    exp.position,
                    exp.start_date,
                    exp.end_date,
                    exp.location,
                    exp.description,
                ],
            )?;

            let experience_id = tx.last_insert_rowid();

            // Insert experience_achievements
            for achievement in &exp.achievements {
                tx.execute(
                    "INSERT INTO experience_achievements (experience_id, achievement)
                     VALUES (?1, ?2)",
                    params![experience_id, achievement],
                )?;
            }

            // Insert experience_technologies
            for technology in &exp.technologies {
                tx.execute(
                    "INSERT INTO experience_technologies (experience_id, technology)
                     VALUES (?1, ?2)",
                    params![experience_id, technology],
                )?;
            }
        }

        // Insert education
        for edu in &cv.education {
            tx.execute(
                "INSERT INTO education (institution, degree, field, start_date, end_date, location, gpa)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    edu.institution,
                    edu.degree,
                    edu.field,
                    edu.start_date,
                    edu.end_date,
                    edu.location,
                    edu.gpa,
                ],
            )?;

            let education_id = tx.last_insert_rowid();

            // Insert education_achievements
            for achievement in &edu.achievements {
                tx.execute(
                    "INSERT INTO education_achievements (education_id, achievement)
                     VALUES (?1, ?2)",
                    params![education_id, achievement],
                )?;
            }
        }

        // Insert skill_categories
        for category in &cv.skill_categories {
            tx.execute(
                "INSERT INTO skill_categories (name)
                 VALUES (?1)",
                params![category.name],
            )?;

            let category_id = tx.last_insert_rowid();

            // Insert skills
            for skill in &category.skills {
                tx.execute(
                    "INSERT INTO skills (category_id, skill)
                     VALUES (?1, ?2)",
                    params![category_id, skill],
                )?;
            }
        }

        // Insert projects
        for project in &cv.projects {
            tx.execute(
                "INSERT INTO projects (name, description, url, repository, stars, owner_username, owner_avatar)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    project.name,
                    project.description,
                    project.url,
                    project.repository,
                    project.stars,
                    project.owner_username,
                    project.owner_avatar,
                ],
            )?;

            let project_id = tx.last_insert_rowid();

            // Insert project_technologies
            for technology in &project.technologies {
                tx.execute(
                    "INSERT INTO project_technologies (project_id, technology)
                     VALUES (?1, ?2)",
                    params![project_id, technology],
                )?;
            }

            // Insert project_highlights
            for highlight in &project.highlights {
                tx.execute(
                    "INSERT INTO project_highlights (project_id, highlight)
                     VALUES (?1, ?2)",
                    params![project_id, highlight],
                )?;
            }
        }

        // Insert languages
        for (language, proficiency) in &cv.languages {
            tx.execute(
                "INSERT INTO languages (language, proficiency)
                 VALUES (?1, ?2)",
                params![language, proficiency],
            )?;
        }

        // Insert certifications
        for certification in &cv.certifications {
            tx.execute(
                "INSERT INTO certifications (certification)
                 VALUES (?1)",
                params![certification],
            )?;
        }

        // Insert github_sources
        for source in &cv.github_sources {
            tx.execute(
                "INSERT INTO github_sources (username, organization)
                 VALUES (?1, ?2)",
                params![source.username, source.organization],
            )?;
        }

        // Commit the transaction
        tx.commit()?;

        Ok(())
    }

    /// Loads CV data from the database
    ///
    /// # Returns
    ///
    /// A Result containing the CV data or an error
    pub fn load_cv(&self) -> Result<Cv> {
        use im::Vector;

        // Load personal_info
        let mut stmt = self.conn.prepare("SELECT id, name, title, email, phone, website, location, summary FROM personal_info LIMIT 1")?;
        let personal_info_row = stmt.query_row([], |row| {
            let id: i64 = row.get(0)?;
            let name: String = row.get(1)?;
            let title: String = row.get(2)?;
            let email: String = row.get(3)?;
            let phone: Option<String> = row.get(4)?;
            let website: Option<String> = row.get(5)?;
            let location: Option<String> = row.get(6)?;
            let summary: String = row.get(7)?;

            // Load social_links
            let mut social_links = im::HashMap::new();
            let mut social_stmt = self
                .conn
                .prepare("SELECT platform, url FROM social_links WHERE personal_info_id = ?1")?;
            let social_rows = social_stmt.query_map(params![id], |row| {
                let platform: String = row.get(0)?;
                let url: String = row.get(1)?;
                Ok((platform, url))
            })?;

            for social_row in social_rows {
                let (platform, url) = social_row?;
                social_links.insert(platform, url);
            }

            Ok(PersonalInfo {
                name,
                title,
                email,
                phone,
                website,
                location,
                summary,
                social_links,
            })
        })?;

        // Load experiences
        let mut experiences = Vector::new();
        let mut exp_stmt = self.conn.prepare("SELECT id, company, position, start_date, end_date, location, description FROM experiences")?;
        let exp_rows = exp_stmt.query_map([], |row| {
            let id: i64 = row.get(0)?;
            let company: String = row.get(1)?;
            let position: String = row.get(2)?;
            let start_date: String = row.get(3)?;
            let end_date: Option<String> = row.get(4)?;
            let location: Option<String> = row.get(5)?;
            let description: String = row.get(6)?;

            // Load achievements
            let mut achievements = Vector::new();
            let mut ach_stmt = self.conn.prepare(
                "SELECT achievement FROM experience_achievements WHERE experience_id = ?1",
            )?;
            let ach_rows = ach_stmt.query_map(params![id], |row| {
                let achievement: String = row.get(0)?;
                Ok(achievement)
            })?;

            for ach_row in ach_rows {
                achievements.push_back(ach_row?);
            }

            // Load technologies
            let mut technologies = Vector::new();
            let mut tech_stmt = self.conn.prepare(
                "SELECT technology FROM experience_technologies WHERE experience_id = ?1",
            )?;
            let tech_rows = tech_stmt.query_map(params![id], |row| {
                let technology: String = row.get(0)?;
                Ok(technology)
            })?;

            for tech_row in tech_rows {
                technologies.push_back(tech_row?);
            }

            Ok(Experience {
                company,
                position,
                start_date,
                end_date,
                location,
                description,
                achievements,
                technologies,
            })
        })?;

        for exp_row in exp_rows {
            experiences.push_back(exp_row?);
        }

        // Load education
        let mut education = Vector::new();
        let mut edu_stmt = self.conn.prepare("SELECT id, institution, degree, field, start_date, end_date, location, gpa FROM education")?;
        let edu_rows = edu_stmt.query_map([], |row| {
            let id: i64 = row.get(0)?;
            let institution: String = row.get(1)?;
            let degree: String = row.get(2)?;
            let field: String = row.get(3)?;
            let start_date: String = row.get(4)?;
            let end_date: Option<String> = row.get(5)?;
            let location: Option<String> = row.get(6)?;
            let gpa: Option<String> = row.get(7)?;

            // Load achievements
            let mut achievements = Vector::new();
            let mut ach_stmt = self.conn.prepare(
                "SELECT achievement FROM education_achievements WHERE education_id = ?1",
            )?;
            let ach_rows = ach_stmt.query_map(params![id], |row| {
                let achievement: String = row.get(0)?;
                Ok(achievement)
            })?;

            for ach_row in ach_rows {
                achievements.push_back(ach_row?);
            }

            Ok(Education {
                institution,
                degree,
                field,
                start_date,
                end_date,
                location,
                gpa,
                achievements,
            })
        })?;

        for edu_row in edu_rows {
            education.push_back(edu_row?);
        }

        // Load skill_categories
        let mut skill_categories = Vector::new();
        let mut cat_stmt = self.conn.prepare("SELECT id, name FROM skill_categories")?;
        let cat_rows = cat_stmt.query_map([], |row| {
            let id: i64 = row.get(0)?;
            let name: String = row.get(1)?;

            // Load skills
            let mut skills = Vector::new();
            let mut skill_stmt = self
                .conn
                .prepare("SELECT skill FROM skills WHERE category_id = ?1")?;
            let skill_rows = skill_stmt.query_map(params![id], |row| {
                let skill: String = row.get(0)?;
                Ok(skill)
            })?;

            for skill_row in skill_rows {
                skills.push_back(skill_row?);
            }

            Ok(SkillCategory { name, skills })
        })?;

        for cat_row in cat_rows {
            skill_categories.push_back(cat_row?);
        }

        // Load projects
        let mut projects = Vector::new();
        let mut proj_stmt = self.conn.prepare("SELECT id, name, description, url, repository, stars, owner_username, owner_avatar FROM projects")?;
        let proj_rows = proj_stmt.query_map([], |row| {
            let id: i64 = row.get(0)?;
            let name: String = row.get(1)?;
            let description: String = row.get(2)?;
            let url: Option<String> = row.get(3)?;
            let repository: Option<String> = row.get(4)?;
            let stars: Option<u32> = row.get(5)?;
            let owner_username: Option<String> = row.get(6)?;
            let owner_avatar: Option<String> = row.get(7)?;

            // Load technologies
            let mut technologies = Vector::new();
            let mut tech_stmt = self
                .conn
                .prepare("SELECT technology FROM project_technologies WHERE project_id = ?1")?;
            let tech_rows = tech_stmt.query_map(params![id], |row| {
                let technology: String = row.get(0)?;
                Ok(technology)
            })?;

            for tech_row in tech_rows {
                technologies.push_back(tech_row?);
            }

            // Load highlights
            let mut highlights = Vector::new();
            let mut high_stmt = self
                .conn
                .prepare("SELECT highlight FROM project_highlights WHERE project_id = ?1")?;
            let high_rows = high_stmt.query_map(params![id], |row| {
                let highlight: String = row.get(0)?;
                Ok(highlight)
            })?;

            for high_row in high_rows {
                highlights.push_back(high_row?);
            }

            Ok(Project {
                name,
                description,
                url,
                repository,
                technologies,
                highlights,
                stars,
                owner_username,
                owner_avatar,
                language: None,
                language_icon: None,
                display_name: None,
            })
        })?;

        for proj_row in proj_rows {
            projects.push_back(proj_row?);
        }

        // Load languages
        let mut languages = im::HashMap::new();
        let mut lang_stmt = self
            .conn
            .prepare("SELECT language, proficiency FROM languages")?;
        let lang_rows = lang_stmt.query_map([], |row| {
            let language: String = row.get(0)?;
            let proficiency: String = row.get(1)?;
            Ok((language, proficiency))
        })?;

        for lang_row in lang_rows {
            let (language, proficiency) = lang_row?;
            languages.insert(language, proficiency);
        }

        // Load certifications
        let mut certifications = Vector::new();
        let mut cert_stmt = self
            .conn
            .prepare("SELECT certification FROM certifications")?;
        let cert_rows = cert_stmt.query_map([], |row| {
            let certification: String = row.get(0)?;
            Ok(certification)
        })?;

        for cert_row in cert_rows {
            certifications.push_back(cert_row?);
        }

        // Load github_sources
        let mut github_sources = Vector::new();
        let mut src_stmt = self
            .conn
            .prepare("SELECT username, organization FROM github_sources")?;
        let src_rows = src_stmt.query_map([], |row| {
            let username: Option<String> = row.get(0)?;
            let organization: Option<String> = row.get(1)?;
            Ok(GitHubSource {
                username,
                organization,
            })
        })?;

        for src_row in src_rows {
            github_sources.push_back(src_row?);
        }

        // Create the CV
        Ok(Cv {
            personal_info: personal_info_row,
            experiences,
            education,
            skill_categories,
            projects,
            languages,
            certifications,
            github_sources,
        })
    }
}
