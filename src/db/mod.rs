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

pub mod error;
pub mod migrations;
pub mod optimized_queries;
pub mod pool_metrics;
pub mod repository;

#[allow(unused_imports)]
pub use migrations::run_migrations;
pub use repository::BlogRepository;

/// Run migrations on a database pool
#[allow(dead_code)]
pub async fn run_migrations_async(pool: &Pool<SqliteConnectionManager>) -> Result<()> {
    let conn = pool.get()?;
    migrations::run_migrations(&conn)
}

/// Create a connection pool for the database
#[allow(dead_code)]
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
        let metrics = Arc::new(pool_metrics::PoolMetrics::new(&format!(
            "db-{}",
            config.path
        )));

        Ok(Self {
            pool: Arc::new(pool),
            metrics,
        })
    }

    /// Creates the database schema for CV data
    pub fn create_schema(&self) -> Result<()> {
        // Create a runtime for executing async code in a synchronous context
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(self.create_schema_async())
    }

    /// Creates the database schema for CV data (async version)
    pub async fn create_schema_async(&self) -> Result<()> {
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
                    skill TEXT NOT NULL,
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
                    owner_avatar TEXT
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

            Ok(())
        })
        .await
    }

    /// Inserts CV data into the database
    pub fn insert_cv(&self, cv: &crate::cv_data::Cv) -> Result<()> {
        // Create a runtime for executing async code in a synchronous context
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(self.insert_cv_async(cv))
    }

    /// Inserts CV data into the database (async version)
    pub async fn insert_cv_async(&self, cv: &crate::cv_data::Cv) -> Result<()> {
        // Clone the CV data to avoid lifetime issues
        let cv_clone = cv.clone();

        self.with_connection_mut(move |conn| {
            // Start a transaction
            let tx = conn.transaction()?;

            // Insert personal_info
            tx.execute(
                "INSERT INTO personal_info (name, title, email, phone, website, location, summary)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    cv_clone.personal_info.name,
                    cv_clone.personal_info.title,
                    cv_clone.personal_info.email,
                    cv_clone.personal_info.phone,
                    cv_clone.personal_info.website,
                    cv_clone.personal_info.location,
                    cv_clone.personal_info.summary,
                ],
            )?;

            let personal_info_id = tx.last_insert_rowid();

            // Insert social_links
            for (platform, url) in &cv_clone.personal_info.social_links {
                tx.execute(
                    "INSERT INTO social_links (personal_info_id, platform, url)
                    VALUES (?1, ?2, ?3)",
                    params![personal_info_id, platform, url],
                )?;
            }

            // Insert experiences
            for exp in &cv_clone.experiences {
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
            for edu in &cv_clone.education {
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
            for category in &cv_clone.skill_categories {
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
            for project in &cv_clone.projects {
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
            for (language, proficiency) in &cv_clone.languages {
                tx.execute(
                    "INSERT INTO languages (language, proficiency)
                    VALUES (?1, ?2)",
                    params![language, proficiency],
                )?;
            }

            // Insert certifications
            for certification in &cv_clone.certifications {
                tx.execute(
                    "INSERT INTO certifications (certification)
                    VALUES (?1)",
                    params![certification],
                )?;
            }

            // Insert github_sources
            for source in &cv_clone.github_sources {
                tx.execute(
                    "INSERT INTO github_sources (username, organization)
                    VALUES (?1, ?2)",
                    params![source.username, source.organization],
                )?;
            }

            // Commit the transaction
            tx.commit()?;

            Ok(())
        }).await
    }

    /// Loads CV data from the database
    pub fn load_cv(&self) -> Result<crate::cv_data::Cv> {
        // Create a runtime for executing async code in a synchronous context
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(self.load_cv_async())
    }

    /// Loads CV data from the database (async version)
    pub async fn load_cv_async(&self) -> Result<crate::cv_data::Cv> {
        self.with_connection(|conn| {
            use im::Vector;

            // Load personal_info
            let mut stmt = conn.prepare("SELECT id, name, title, email, phone, website, location, summary FROM personal_info LIMIT 1")?;
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
                let mut social_stmt = conn
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
            let mut experiences = Vector::new();
            let mut exp_stmt = conn.prepare("SELECT id, company, position, start_date, end_date, location, description FROM experiences")?;
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
                let mut ach_stmt = conn.prepare(
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
                let mut tech_stmt = conn.prepare(
                    "SELECT technology FROM experience_technologies WHERE experience_id = ?1",
                )?;
                let tech_rows = tech_stmt.query_map(params![id], |row| {
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
            let mut education = Vector::new();
            let mut edu_stmt = conn.prepare("SELECT id, institution, degree, field, start_date, end_date, location, gpa FROM education")?;
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
                let mut ach_stmt = conn.prepare(
                    "SELECT achievement FROM education_achievements WHERE education_id = ?1",
                )?;
                let ach_rows = ach_stmt.query_map(params![id], |row| {
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

            // Load skill_categories
            let mut skill_categories = Vector::new();
            let mut cat_stmt = conn.prepare("SELECT id, name FROM skill_categories")?;
            let cat_rows = cat_stmt.query_map([], |row| {
                let id: i64 = row.get(0)?;
                let name: String = row.get(1)?;

                // Load skills
                let mut skills = Vector::new();
                let mut skill_stmt = conn
                    .prepare("SELECT skill FROM skills WHERE category_id = ?1")?;
                let skill_rows = skill_stmt.query_map(params![id], |row| {
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
            let mut projects = Vector::new();
            let mut proj_stmt = conn.prepare("SELECT id, name, description, url, repository, stars, owner_username, owner_avatar FROM projects")?;
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
                let mut tech_stmt = conn
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
                let mut high_stmt = conn
                    .prepare("SELECT highlight FROM project_highlights WHERE project_id = ?1")?;
                let high_rows = high_stmt.query_map(params![id], |row| {
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
            let mut certifications = Vector::new();
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
            let mut github_sources = Vector::new();
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
        }).await
    }

    /// Get a repository for blog operations
    #[allow(dead_code)]
    pub fn blog_repository(&self) -> BlogRepository {
        BlogRepository::new(Arc::clone(&self.pool))
    }

    /// Get the metrics object for the connection pool
    #[allow(dead_code)]
    pub fn metrics(&self) -> Arc<pool_metrics::PoolMetrics> {
        Arc::clone(&self.metrics)
    }

    /// Log a summary of the connection pool metrics
    #[allow(dead_code)]
    pub fn log_metrics_summary(&self) {
        self.metrics.log_summary();
    }

    /// Get a snapshot of the connection pool metrics
    #[allow(dead_code)]
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
            let _usage_tracker = metrics.connection_acquired(wait_time);

            // Execute the function with the connection

            // The usage_tracker will be dropped when this function returns,
            // which will record the connection usage time

            f(&conn)
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
            let _usage_tracker = metrics.connection_acquired(wait_time);

            // Execute the function with the mutable connection

            // The usage_tracker will be dropped when this function returns,
            // which will record the connection usage time

            f(&mut conn)
        })
        .await?
    }
}

/// Test that the database is working properly
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_database_creation() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test.db");

        let db = Database::new(db_path)?;

        // Test that we can execute a simple query
        db.with_connection(|conn| {
            conn.execute(
                "CREATE TABLE IF NOT EXISTS test (id INTEGER PRIMARY KEY)",
                [],
            )?;
            conn.execute("INSERT INTO test (id) VALUES (?1)", [1])?;
            let count: i64 = conn.query_row("SELECT COUNT(*) FROM test", [], |row| row.get(0))?;
            assert_eq!(count, 1);
            Ok(())
        })
        .await
    }
}
