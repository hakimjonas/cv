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
    (
        3,
        "Add user authentication",
        "
        -- Create users table
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY,
            username TEXT NOT NULL UNIQUE,
            display_name TEXT NOT NULL,
            email TEXT NOT NULL UNIQUE,
            password_hash TEXT NOT NULL,
            role TEXT NOT NULL DEFAULT 'Author',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        -- Add user_id column to posts table
        ALTER TABLE posts ADD COLUMN user_id INTEGER;

        -- Create index on user_id
        CREATE INDEX IF NOT EXISTS idx_posts_user_id ON posts(user_id);

        -- Create a default admin user with password 'admin'
        -- In production, this password should be changed immediately
        INSERT INTO users (
            username, 
            display_name, 
            email, 
            password_hash, 
            role, 
            created_at, 
            updated_at
        ) 
        VALUES (
            'admin', 
            'Administrator', 
            'admin@example.com', 
            '$argon2id$v=19$m=16,t=2,p=1$MTIzNDU2Nzg$2dIiQwpZKLvQhbKZ8j+6Yw', 
            'Admin', 
            datetime('now'), 
            datetime('now')
        );

        -- Get the ID of the admin user
        UPDATE posts SET user_id = (SELECT id FROM users WHERE username = 'admin');

        -- Add foreign key constraint
        -- SQLite doesn't support adding foreign key constraints to existing tables directly,
        -- so we need to create a new table with the constraint and copy the data
        CREATE TABLE posts_new (
            id INTEGER PRIMARY KEY,
            title TEXT NOT NULL,
            slug TEXT NOT NULL UNIQUE,
            date TEXT NOT NULL,
            user_id INTEGER,
            author TEXT NOT NULL,
            excerpt TEXT NOT NULL,
            content TEXT NOT NULL,
            published BOOLEAN NOT NULL DEFAULT 0,
            featured BOOLEAN NOT NULL DEFAULT 0,
            image TEXT,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE SET NULL
        );

        -- Copy data from posts to posts_new
        INSERT INTO posts_new (id, title, slug, date, user_id, author, excerpt, content, published, featured, image)
        SELECT id, title, slug, date, user_id, author, excerpt, content, published, featured, image FROM posts;

        -- Drop the old table
        DROP TABLE posts;

        -- Rename the new table to posts
        ALTER TABLE posts_new RENAME TO posts;

        -- Recreate indexes
        CREATE INDEX IF NOT EXISTS idx_posts_published ON posts(published);
        CREATE INDEX IF NOT EXISTS idx_posts_featured ON posts(featured);
        CREATE INDEX IF NOT EXISTS idx_posts_date ON posts(date);
        CREATE INDEX IF NOT EXISTS idx_posts_user_id ON posts(user_id);

        -- Update FTS triggers to include the new posts table structure
        DROP TRIGGER IF EXISTS posts_ai;
        DROP TRIGGER IF EXISTS posts_ad;
        DROP TRIGGER IF EXISTS posts_au;

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
        ",
    ),
];
