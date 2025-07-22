/*!
 * Database module providing async operations and connection pooling
 * This module implements a functional approach to database operations
 * with proper connection management and error handling
 */

use anyhow::Result;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;
use std::path::Path;
use std::sync::Arc;
use tokio::task;
use tracing::{debug, info, warn};

use super::error;
use super::migrations;
use super::optimized_queries;
use super::pool_metrics;
use super::repository::BlogRepository;

/// Run migrations on a database pool
pub async fn run_migrations_async(pool: &Pool<SqliteConnectionManager>) -> Result<()> {
    let conn = pool.get()?;
    migrations::run_migrations(&conn)
}

/// Create a connection pool for the database
pub fn create_connection_pool<P: AsRef<Path>>(path: P) -> Result<Pool<SqliteConnectionManager>> {
    let manager = SqliteConnectionManager::file(path);
    let pool = Pool::builder()
        .max_size(10)
        .connection_timeout(std::time::Duration::from_secs(30))
        .build(manager)?;

    // Configure the connections
    let conn = pool.get()?;
    conn.execute_batch(
        "
        PRAGMA journal_mode = WAL;
        PRAGMA synchronous = NORMAL;
        PRAGMA busy_timeout = 5000;
        PRAGMA foreign_keys = ON;
        ",
    )?;

    // Create the migrations table if it doesn't exist
    migrations::initialize_migrations_table(&conn)?;

    Ok(pool)
}

/// Configuration for the database connection
pub struct DatabaseConfig {
    /// Path to the SQLite database file
    pub path: String,
    /// Maximum number of connections in the pool
    pub max_connections: u32,
    /// Connection timeout in seconds
    pub connection_timeout: u64,
    /// Whether to use WAL mode
    pub use_wal: bool,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            path: ":memory:".to_string(),
            max_connections: 10,
            connection_timeout: 30,
            use_wal: true,
        }
    }
}

/// A database connection pool with async operations
pub struct Database {
    /// The connection pool
    pool: Arc<Pool<SqliteConnectionManager>>,
    /// Metrics for the connection pool
    metrics: Arc<pool_metrics::PoolMetrics>,
}

impl Database {
    /// Create a new database connection pool
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        let config = DatabaseConfig {
            path: path_str,
            ..Default::default()
        };
        Self::with_config(config)
    }

    /// Create a new database connection pool with custom configuration
    pub fn with_config(config: DatabaseConfig) -> Result<Self> {
        // Create the connection manager
        let manager = SqliteConnectionManager::file(&config.path);

        // Create the connection pool
        let pool = Pool::builder()
            .max_size(config.max_connections)
            .connection_timeout(std::time::Duration::from_secs(config.connection_timeout))
            .build(manager)?;

        // Configure the connections
        let conn = pool.get()?;
        if config.use_wal {
            conn.execute_batch(
                "
                PRAGMA journal_mode = WAL;
                PRAGMA synchronous = NORMAL;
                PRAGMA busy_timeout = 5000;
                PRAGMA foreign_keys = ON;
            ",
            )?;
        } else {
            conn.execute_batch(
                "
                PRAGMA journal_mode = DELETE;
                PRAGMA synchronous = FULL;
                PRAGMA busy_timeout = 5000;
                PRAGMA foreign_keys = ON;
            ",
            )?;
        }

        // Create the migrations table if it doesn't exist
        migrations::initialize_migrations_table(&conn)?;

        // Run migrations
        migrations::run_migrations(&conn)?;

        // Create metrics for the connection pool
        let metrics = Arc::new(pool_metrics::PoolMetrics::new(&format!("db-{}", config.path)));

        Ok(Self {
            pool: Arc::new(pool),
            metrics,
        })
    }

    /// Creates the database schema for CV data (async version)
    pub async fn create_schema(&self) -> Result<()> {
        self.with_connection(|conn| {
            // Create personal_info table
            conn.execute(
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
            conn.execute(
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
            conn.execute(
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
            conn.execute(
                "CREATE TABLE IF NOT EXISTS experience_achievements (
                    id INTEGER PRIMARY KEY,
                    experience_id INTEGER NOT NULL,
                    achievement TEXT NOT NULL,
                    FOREIGN KEY (experience_id) REFERENCES experiences (id)
                )",
                [],
            )?;

            // Create experience_technologies table
            conn.execute(
                "CREATE TABLE IF NOT EXISTS experience_technologies (
                    id INTEGER PRIMARY KEY,
                    experience_id INTEGER NOT NULL,
                    technology TEXT NOT NULL,
                    FOREIGN KEY (experience_id) REFERENCES experiences (id)
                )",
                [],
            )?;

            // Create education table
            conn.execute(
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
            conn.execute(
                "CREATE TABLE IF NOT EXISTS education_achievements (
                    id INTEGER PRIMARY KEY,
                    education_id INTEGER NOT NULL,
                    achievement TEXT NOT NULL,
                    FOREIGN KEY (education_id) REFERENCES education (id)
                )",
                [],
            )?;

            // Create skill_categories table
            conn.execute(
                "CREATE TABLE IF NOT EXISTS skill_categories (
                    id INTEGER PRIMARY KEY,
                    name TEXT NOT NULL
                )",
                [],
            )?;

            // Create skills table
            conn.execute(
                "CREATE TABLE IF NOT EXISTS skills (
                    id INTEGER PRIMARY KEY,
                    category_id INTEGER NOT NULL,
                    name TEXT NOT NULL,
                    FOREIGN KEY (category_id) REFERENCES skill_categories (id)
                )",
                [],
            )?;

            // Create projects table
            conn.execute(
                "CREATE TABLE IF NOT EXISTS projects (
                    id INTEGER PRIMARY KEY,
                    name TEXT NOT NULL,
                    description TEXT NOT NULL,
                    url TEXT,
                    repository TEXT,
                    stars INTEGER,
                    owner_username TEXT,
                    owner_avatar TEXT,
                    language TEXT,
                    language_icon TEXT,
                    display_name TEXT
                )",
                [],
            )?;

            // Create project_technologies table
            conn.execute(
                "CREATE TABLE IF NOT EXISTS project_technologies (
                    id INTEGER PRIMARY KEY,
                    project_id INTEGER NOT NULL,
                    technology TEXT NOT NULL,
                    FOREIGN KEY (project_id) REFERENCES projects (id)
                )",
                [],
            )?;

            // Create project_highlights table
            conn.execute(
                "CREATE TABLE IF NOT EXISTS project_highlights (
                    id INTEGER PRIMARY KEY,
                    project_id INTEGER NOT NULL,
                    highlight TEXT NOT NULL,
                    FOREIGN KEY (project_id) REFERENCES projects (id)
                )",
                [],
            )?;

            // Create languages table
            conn.execute(
                "CREATE TABLE IF NOT EXISTS languages (
                    id INTEGER PRIMARY KEY,
                    language TEXT NOT NULL,
                    proficiency TEXT NOT NULL
                )",
                [],
            )?;

            // Create certifications table
            conn.execute(
                "CREATE TABLE IF NOT EXISTS certifications (
                    id INTEGER PRIMARY KEY,
                    certification TEXT NOT NULL
                )",
                [],
            )?;

            // Create github_sources table
            conn.execute(
                "CREATE TABLE IF NOT EXISTS github_sources (
                    id INTEGER PRIMARY KEY,
                    username TEXT,
                    organization TEXT
                )",
                [],
            )?;

            // Create blog_posts table
            conn.execute(
                "CREATE TABLE IF NOT EXISTS posts (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    title TEXT NOT NULL,
                    slug TEXT NOT NULL UNIQUE,
                    date TEXT NOT NULL,
                    author TEXT NOT NULL,
                    excerpt TEXT NOT NULL,
                    content TEXT NOT NULL,
                    published BOOLEAN NOT NULL,
                    featured BOOLEAN NOT NULL,
                    image TEXT
                )",
                [],
            )?;

            // Create tags table
            conn.execute(
                "CREATE TABLE IF NOT EXISTS tags (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    name TEXT NOT NULL UNIQUE,
                    slug TEXT NOT NULL UNIQUE
                )",
                [],
            )?;

            // Create post_tags table
            conn.execute(
                "CREATE TABLE IF NOT EXISTS post_tags (
                    post_id INTEGER NOT NULL,
                    tag_id INTEGER NOT NULL,
                    PRIMARY KEY (post_id, tag_id),
                    FOREIGN KEY (post_id) REFERENCES posts(id) ON DELETE CASCADE,
                    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
                )",
                [],
            )?;

            // Create post_metadata table
            conn.execute(
                "CREATE TABLE IF NOT EXISTS post_metadata (
                    post_id INTEGER NOT NULL,
                    key TEXT NOT NULL,
                    value TEXT NOT NULL,
                    PRIMARY KEY (post_id, key),
                    FOREIGN KEY (post_id) REFERENCES posts(id) ON DELETE CASCADE
                )",
                [],
            )?;

            Ok(())
        })
        .await
    }

    /// Insert a CV into the database (async version)
    pub async fn insert_cv(&self, cv: &crate::cv_data::Cv) -> Result<()> {
        self.with_connection_mut(|conn| {
            // Start a transaction
            let tx = conn.transaction()?;

            // Insert personal info
            tx.execute(
                "INSERT OR REPLACE INTO personal_info (id, name, title, email, phone, website, location, summary)
                 VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    cv.personal_info.name,
                    cv.personal_info.title,
                    cv.personal_info.email,
                    cv.personal_info.phone,
                    cv.personal_info.website,
                    cv.personal_info.location,
                    cv.personal_info.summary
                ],
            )?;

            // Clear existing social links
            tx.execute("DELETE FROM social_links WHERE personal_info_id = 1", [])?;

            // Insert social links
            for (platform, url) in cv.personal_info.social_links.iter() {
                tx.execute(
                    "INSERT INTO social_links (personal_info_id, platform, url) VALUES (1, ?1, ?2)",
                    params![platform, url],
                )?;
            }

            // Clear existing experiences
            tx.execute("DELETE FROM experiences", [])?;
            tx.execute("DELETE FROM experience_achievements", [])?;
            tx.execute("DELETE FROM experience_technologies", [])?;

            // Insert experiences
            for exp in cv.experiences.iter() {
                tx.execute(
                    "INSERT INTO experiences (company, position, start_date, end_date, location, description)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![
                        exp.company,
                        exp.position,
                        exp.start_date,
                        exp.end_date,
                        exp.location,
                        exp.description
                    ],
                )?;

                let exp_id = tx.last_insert_rowid();

                // Insert achievements
                for achievement in exp.achievements.iter() {
                    tx.execute(
                        "INSERT INTO experience_achievements (experience_id, achievement)
                         VALUES (?1, ?2)",
                        params![exp_id, achievement],
                    )?;
                }

                // Insert technologies
                for tech in exp.technologies.iter() {
                    tx.execute(
                        "INSERT INTO experience_technologies (experience_id, technology)
                         VALUES (?1, ?2)",
                        params![exp_id, tech],
                    )?;
                }
            }

            // Clear existing education
            tx.execute("DELETE FROM education", [])?;
            tx.execute("DELETE FROM education_achievements", [])?;

            // Insert education
            for edu in cv.education.iter() {
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
                        edu.gpa
                    ],
                )?;

                let edu_id = tx.last_insert_rowid();

                // Insert achievements
                for achievement in edu.achievements.iter() {
                    tx.execute(
                        "INSERT INTO education_achievements (education_id, achievement)
                         VALUES (?1, ?2)",
                        params![edu_id, achievement],
                    )?;
                }
            }

            // Clear existing skill categories and skills
            tx.execute("DELETE FROM skills", [])?;
            tx.execute("DELETE FROM skill_categories", [])?;

            // Insert skill categories and skills
            for cat in cv.skill_categories.iter() {
                tx.execute(
                    "INSERT INTO skill_categories (name) VALUES (?1)",
                    params![cat.name],
                )?;

                let cat_id = tx.last_insert_rowid();

                // Insert skills
                for skill in cat.skills.iter() {
                    tx.execute(
                        "INSERT INTO skills (category_id, name) VALUES (?1, ?2)",
                        params![cat_id, skill],
                    )?;
                }
            }

            // Clear existing projects
            tx.execute("DELETE FROM projects", [])?;
            tx.execute("DELETE FROM project_technologies", [])?;
            tx.execute("DELETE FROM project_highlights", [])?;

            // Insert projects
            for proj in cv.projects.iter() {
                tx.execute(
                    "INSERT INTO projects (name, description, url, repository, stars, owner_username, owner_avatar, language, language_icon, display_name)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                    params![
                        proj.name,
                        proj.description,
                        proj.url,
                        proj.repository,
                        proj.stars,
                        proj.owner_username,
                        proj.owner_avatar,
                        proj.language,
                        proj.language_icon,
                        proj.display_name
                    ],
                )?;

                let proj_id = tx.last_insert_rowid();

                // Insert technologies
                for tech in proj.technologies.iter() {
                    tx.execute(
                        "INSERT INTO project_technologies (project_id, technology)
                         VALUES (?1, ?2)",
                        params![proj_id, tech],
                    )?;
                }

                // Insert highlights
                for highlight in proj.highlights.iter() {
                    tx.execute(
                        "INSERT INTO project_highlights (project_id, highlight)
                         VALUES (?1, ?2)",
                        params![proj_id, highlight],
                    )?;
                }
            }

            // Clear existing languages
            tx.execute("DELETE FROM languages", [])?;

            // Insert languages
            for (lang, prof) in cv.languages.iter() {
                tx.execute(
                    "INSERT INTO languages (language, proficiency) VALUES (?1, ?2)",
                    params![lang, prof],
                )?;
            }

            // Clear existing certifications
            tx.execute("DELETE FROM certifications", [])?;

            // Insert certifications
            for cert in cv.certifications.iter() {
                tx.execute(
                    "INSERT INTO certifications (certification) VALUES (?1)",
                    params![cert],
                )?;
            }

            // Clear existing github sources
            tx.execute("DELETE FROM github_sources", [])?;

            // Insert github sources
            for src in cv.github_sources.iter() {
                tx.execute(
                    "INSERT INTO github_sources (username, organization) VALUES (?1, ?2)",
                    params![src.username, src.organization],
                )?;
            }

            // Commit the transaction
            tx.commit()?;

            Ok(())
        })
        .await
    }

    /// Load a CV from the database (async version)
    pub async fn load_cv(&self) -> Result<crate::cv_data::Cv> {
        self.with_connection(|conn| {
            // Load personal info
            let mut personal_info_stmt = conn.prepare(
                "SELECT name, title, email, phone, website, location, summary FROM personal_info WHERE id = 1",
            )?;
            let personal_info_row = personal_info_stmt.query_row([], |row| {
                let name: String = row.get(0)?;
                let title: String = row.get(1)?;
                let email: String = row.get(2)?;
                let phone: Option<String> = row.get(3)?;
                let website: Option<String> = row.get(4)?;
                let location: Option<String> = row.get(5)?;
                let summary: String = row.get(6)?;

                // Load social links
                let mut social_links = im::HashMap::new();
                let mut social_links_stmt = conn.prepare(
                    "SELECT platform, url FROM social_links WHERE personal_info_id = 1",
                )?;
                let social_links_rows = social_links_stmt.query_map([], |row| {
                    let platform: String = row.get(0)?;
                    let url: String = row.get(1)?;
                    Ok((platform, url))
                })?;

                for link_row in social_links_rows {
                    let (platform, url) = link_row?;
                    social_links = social_links.update(platform, url);
                }

                Ok(crate::cv_data::PersonalInfo {
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
            let mut experiences = im::Vector::new();
            let mut exp_stmt = conn.prepare(
                "SELECT id, company, position, start_date, end_date, location, description FROM experiences",
            )?;
            let exp_rows = exp_stmt.query_map([], |row| {
                let id: i64 = row.get(0)?;
                let company: String = row.get(1)?;
                let position: String = row.get(2)?;
                let start_date: String = row.get(3)?;
                let end_date: Option<String> = row.get(4)?;
                let location: Option<String> = row.get(5)?;
                let description: String = row.get(6)?;

                // Load achievements
                let mut achievements = im::Vector::new();
                let mut ach_stmt = conn.prepare(
                    "SELECT achievement FROM experience_achievements WHERE experience_id = ?1",
                )?;
                let ach_rows = ach_stmt.query_map([id], |row| {
                    let achievement: String = row.get(0)?;
                    Ok(achievement)
                })?;

                for ach_row in ach_rows {
                    achievements.push_back(ach_row?);
                }

                // Load technologies
                let mut technologies = im::Vector::new();
                let mut tech_stmt = conn.prepare(
                    "SELECT technology FROM experience_technologies WHERE experience_id = ?1",
                )?;
                let tech_rows = tech_stmt.query_map([id], |row| {
                    let technology: String = row.get(0)?;
                    Ok(technology)
                })?;

                for tech_row in tech_rows {
                    technologies.push_back(tech_row?);
                }

                Ok(crate::cv_data::Experience {
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
            let mut education = im::Vector::new();
            let mut edu_stmt = conn.prepare(
                "SELECT id, institution, degree, field, start_date, end_date, location, gpa FROM education",
            )?;
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
                let mut achievements = im::Vector::new();
                let mut ach_stmt = conn.prepare(
                    "SELECT achievement FROM education_achievements WHERE education_id = ?1",
                )?;
                let ach_rows = ach_stmt.query_map([id], |row| {
                    let achievement: String = row.get(0)?;
                    Ok(achievement)
                })?;

                for ach_row in ach_rows {
                    achievements.push_back(ach_row?);
                }

                Ok(crate::cv_data::Education {
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

            // Load skill categories
            let mut skill_categories = im::Vector::new();
            let mut cat_stmt = conn.prepare("SELECT id, name FROM skill_categories")?;
            let cat_rows = cat_stmt.query_map([], |row| {
                let id: i64 = row.get(0)?;
                let name: String = row.get(1)?;

                // Load skills
                let mut skills = im::Vector::new();
                let mut skill_stmt =
                    conn.prepare("SELECT name FROM skills WHERE category_id = ?1")?;
                let skill_rows = skill_stmt.query_map([id], |row| {
                    let skill: String = row.get(0)?;
                    Ok(skill)
                })?;

                for skill_row in skill_rows {
                    skills.push_back(skill_row?);
                }

                Ok(crate::cv_data::SkillCategory { name, skills })
            })?;

            for cat_row in cat_rows {
                skill_categories.push_back(cat_row?);
            }

            // Load projects
            let mut projects = im::Vector::new();
            let mut proj_stmt = conn.prepare(
                "SELECT id, name, description, url, repository, stars, owner_username, owner_avatar, language, language_icon, display_name FROM projects",
            )?;
            let proj_rows = proj_stmt.query_map([], |row| {
                let id: i64 = row.get(0)?;
                let name: String = row.get(1)?;
                let description: String = row.get(2)?;
                let url: Option<String> = row.get(3)?;
                let repository: Option<String> = row.get(4)?;
                let stars: Option<i64> = row.get(5)?;
                let owner_username: Option<String> = row.get(6)?;
                let owner_avatar: Option<String> = row.get(7)?;
                let language: Option<String> = row.get(8)?;
                let language_icon: Option<String> = row.get(9)?;
                let display_name: Option<String> = row.get(10)?;

                // Load technologies
                let mut technologies = im::Vector::new();
                let mut tech_stmt = conn.prepare(
                    "SELECT technology FROM project_technologies WHERE project_id = ?1",
                )?;
                let tech_rows = tech_stmt.query_map([id], |row| {
                    let technology: String = row.get(0)?;
                    Ok(technology)
                })?;

                for tech_row in tech_rows {
                    technologies.push_back(tech_row?);
                }

                // Load highlights
                let mut highlights = im::Vector::new();
                let mut high_stmt = conn.prepare(
                    "SELECT highlight FROM project_highlights WHERE project_id = ?1",
                )?;
                let high_rows = high_stmt.query_map([id], |row| {
                    let highlight: String = row.get(0)?;
                    Ok(highlight)
                })?;

                for high_row in high_rows {
                    highlights.push_back(high_row?);
                }

                Ok(crate::cv_data::Project {
                    name,
                    description,
                    url,
                    repository,
                    technologies,
                    highlights,
                    stars,
                    owner_username,
                    owner_avatar,
                    language,
                    language_icon,
                    display_name,
                })
            })?;

            for proj_row in proj_rows {
                projects.push_back(proj_row?);
            }

            // Load languages
            let mut languages = im::HashMap::new();
            let mut lang_stmt = conn
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
            let mut certifications = im::Vector::new();
            let mut cert_stmt = conn
                .prepare("SELECT certification FROM certifications")?;
            let cert_rows = cert_stmt.query_map([], |row| {
                let certification: String = row.get(0)?;
                Ok(certification)
            })?;

            for cert_row in cert_rows {
                certifications.push_back(cert_row?);
            }

            // Load github_sources
            let mut github_sources = im::Vector::new();
            let mut src_stmt = conn
                .prepare("SELECT username, organization FROM github_sources")?;
            let src_rows = src_stmt.query_map([], |row| {
                let username: Option<String> = row.get(0)?;
                let organization: Option<String> = row.get(1)?;
                Ok(crate::cv_data::GitHubSource {
                    username,
                    organization,
                })
            })?;

            for src_row in src_rows {
                github_sources.push_back(src_row?);
            }

            // Create the CV
            Ok(crate::cv_data::Cv {
                personal_info: personal_info_row,
                experiences,
                education,
                skill_categories,
                projects,
                languages,
                certifications,
                github_sources,
            })
        })
        .await
    }

    /// Get a repository for blog operations
    pub fn blog_repository(&self) -> BlogRepository {
        BlogRepository::new(Arc::clone(&self.pool))
    }
    
    /// Get the metrics object for the connection pool
    pub fn metrics(&self) -> Arc<pool_metrics::PoolMetrics> {
        Arc::clone(&self.metrics)
    }
    
    /// Log a summary of the connection pool metrics
    pub fn log_metrics_summary(&self) {
        self.metrics.log_summary();
    }
    
    /// Get a snapshot of the connection pool metrics
    pub fn get_metrics_snapshot(&self) -> pool_metrics::MetricsSnapshot {
        self.metrics.get_snapshot()
    }

    /// Execute a function with a connection from the pool
    pub async fn with_connection<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&rusqlite::Connection) -> Result<T> + Send + 'static,
        T: Send + 'static,
    {
        let pool = Arc::clone(&self.pool);
        let metrics = Arc::clone(&self.metrics);
        
        task::spawn_blocking(move || {
            // Record the start time for connection acquisition
            let start_time = std::time::Instant::now();
            
            // Get a connection from the pool
            let conn = match pool.get() {
                Ok(conn) => conn,
                Err(e) => {
                    // Record connection error
                    metrics.connection_error();
                    return Err(e.into());
                }
            };
            
            // Calculate wait time and record connection acquisition
            let wait_time = start_time.elapsed();
            let usage_tracker = metrics.connection_acquired(wait_time);
            
            // Execute the function with the connection
            let result = f(&conn);
            
            // The usage_tracker will be dropped when this function returns,
            // which will record the connection usage time
            
            result
        })
        .await?
    }

    /// Execute a function with a mutable connection from the pool
    pub async fn with_connection_mut<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&mut rusqlite::Connection) -> Result<T> + Send + 'static,
        T: Send + 'static,
    {
        let pool = Arc::clone(&self.pool);
        let metrics = Arc::clone(&self.metrics);
        
        task::spawn_blocking(move || {
            // Record the start time for connection acquisition
            let start_time = std::time::Instant::now();
            
            // Get a connection from the pool
            let mut conn = match pool.get() {
                Ok(conn) => conn,
                Err(e) => {
                    // Record connection error
                    metrics.connection_error();
                    return Err(e.into());
                }
            };
            
            // Calculate wait time and record connection acquisition
            let wait_time = start_time.elapsed();
            let usage_tracker = metrics.connection_acquired(wait_time);
            
            // Execute the function with the mutable connection
            let result = f(&mut conn);
            
            // The usage_tracker will be dropped when this function returns,
            // which will record the connection usage time
            
            result
        })
        .await?
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_database_creation() -> Result<()> {
        let temp_dir = tempdir()?;
        let db_path = temp_dir.path().join("test.db");
        
        let db = Database::new(&db_path)?;
        
        // Create schema
        db.create_schema().await?;
        
        // Check that the database file exists
        assert!(db_path.exists());
        
        Ok(())
    }
}