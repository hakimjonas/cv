/*!
 * Database migrations
 * This module handles creating and updating the database schema
 */

use anyhow::{Result, anyhow};
use rusqlite::{Connection, OptionalExtension, params};
use tracing::{error, info};

/// Initialize the migrations table
pub fn initialize_migrations_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "
        CREATE TABLE IF NOT EXISTS migrations (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            applied_at TEXT NOT NULL
        )
    ",
        [],
    )?;

    Ok(())
}

/// Run all pending migrations
pub fn run_migrations(conn: &Connection) -> Result<()> {
    // Get the latest applied migration
    let latest_migration: Option<i64> = conn
        .query_row(
            "
        SELECT id FROM migrations ORDER BY id DESC LIMIT 1
    ",
            [],
            |row| row.get(0),
        )
        .optional()?;

    let latest_migration = latest_migration.unwrap_or(0);
    info!("Latest migration applied: {}", latest_migration);

    // Apply all migrations after the latest one
    for (id, name, sql) in MIGRATIONS.iter() {
        if *id > latest_migration {
            info!("Applying migration {}: {}", id, name);

            // Execute the migration
            match conn.execute_batch(sql) {
                Ok(_) => {
                    // Record the migration
                    let now = chrono::Utc::now().to_rfc3339();
                    conn.execute(
                        "
                        INSERT INTO migrations (id, name, applied_at)
                        VALUES (?1, ?2, ?3)
                    ",
                        params![*id, *name, now],
                    )?;
                    info!("Migration {} applied successfully", id);
                }
                Err(e) => {
                    error!("Failed to apply migration {}: {}", name, e);
                    return Err(anyhow!("Failed to apply migration {}: {}", name, e));
                }
            }
        }
    }

    info!("All migrations applied successfully");
    Ok(())
}

/// List of all migrations
const MIGRATIONS: &[(i64, &str, &str)] = &[
    (
        1,
        "Initial schema",
        "
        -- Posts table
        CREATE TABLE IF NOT EXISTS posts (
            id INTEGER PRIMARY KEY,
            title TEXT NOT NULL,
            slug TEXT NOT NULL UNIQUE,
            date TEXT NOT NULL,
            author TEXT NOT NULL,
            excerpt TEXT NOT NULL,
            content TEXT NOT NULL,
            published BOOLEAN NOT NULL DEFAULT 0,
            featured BOOLEAN NOT NULL DEFAULT 0,
            image TEXT
        );

        -- Tags table
        CREATE TABLE IF NOT EXISTS tags (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            slug TEXT NOT NULL UNIQUE
        );

        -- Post-Tag relationship table
        CREATE TABLE IF NOT EXISTS post_tags (
            post_id INTEGER NOT NULL,
            tag_id INTEGER NOT NULL,
            PRIMARY KEY (post_id, tag_id),
            FOREIGN KEY (post_id) REFERENCES posts(id) ON DELETE CASCADE,
            FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
        );

        -- Post metadata table
        CREATE TABLE IF NOT EXISTS post_metadata (
            post_id INTEGER NOT NULL,
            key TEXT NOT NULL,
            value TEXT NOT NULL,
            PRIMARY KEY (post_id, key),
            FOREIGN KEY (post_id) REFERENCES posts(id) ON DELETE CASCADE
        );

        -- Indexes
        CREATE INDEX IF NOT EXISTS idx_posts_published ON posts(published);
        CREATE INDEX IF NOT EXISTS idx_posts_featured ON posts(featured);
        CREATE INDEX IF NOT EXISTS idx_posts_date ON posts(date);
        CREATE INDEX IF NOT EXISTS idx_tags_slug ON tags(slug);
        ",
    ),
    (
        2,
        "Add full-text search",
        "
        -- Create virtual table for full-text search
        CREATE VIRTUAL TABLE IF NOT EXISTS posts_fts USING fts5(
            title, content, excerpt,
            content='posts',
            content_rowid='id'
        );

        -- Create triggers to keep the FTS index in sync
        CREATE TRIGGER IF NOT EXISTS posts_ai AFTER INSERT ON posts BEGIN
            INSERT INTO posts_fts(rowid, title, content, excerpt) 
            VALUES (new.id, new.title, new.content, new.excerpt);
        END;

        CREATE TRIGGER IF NOT EXISTS posts_ad AFTER DELETE ON posts BEGIN
            INSERT INTO posts_fts(posts_fts, rowid, title, content, excerpt) 
            VALUES('delete', old.id, old.title, old.content, old.excerpt);
        END;

        CREATE TRIGGER IF NOT EXISTS posts_au AFTER UPDATE ON posts BEGIN
            INSERT INTO posts_fts(posts_fts, rowid, title, content, excerpt) 
            VALUES('delete', old.id, old.title, old.content, old.excerpt);
            INSERT INTO posts_fts(rowid, title, content, excerpt) 
            VALUES (new.id, new.title, new.content, new.excerpt);
        END;

        -- Populate FTS table with existing data
        INSERT INTO posts_fts(rowid, title, content, excerpt)
        SELECT id, title, content, excerpt FROM posts;
        ",
    ),
];
