# Database Documentation

This document provides comprehensive documentation for the SQLite database used in the CV project. It includes the database schema, relationships between tables, and explanations of design decisions.

## Database Schema

The database schema consists of the following tables:

### Posts Table

The `posts` table stores blog posts with the following structure:

| Column | Type | Description |
|--------|------|-------------|
| id | INTEGER | Primary key for the post |
| title | TEXT | Title of the post |
| slug | TEXT | URL-friendly version of the title (unique) |
| date | TEXT | Publication date in ISO format |
| author | TEXT | Author of the post |
| excerpt | TEXT | Short summary of the post |
| content | TEXT | Full content of the post |
| published | BOOLEAN | Whether the post is published (1) or draft (0) |
| featured | BOOLEAN | Whether the post is featured (1) or not (0) |
| image | TEXT | Optional URL or path to a featured image |

Indexes:
- `idx_posts_published`: Index on the `published` column for efficient filtering of published posts
- `idx_posts_featured`: Index on the `featured` column for efficient filtering of featured posts
- `idx_posts_date`: Index on the `date` column for efficient sorting by date

### Tags Table

The `tags` table stores tags that can be applied to blog posts:

| Column | Type | Description |
|--------|------|-------------|
| id | INTEGER | Primary key for the tag |
| name | TEXT | Display name of the tag |
| slug | TEXT | URL-friendly version of the name (unique) |

Indexes:
- `idx_tags_slug`: Index on the `slug` column for efficient lookup by slug

### Post-Tags Relationship Table

The `post_tags` table establishes a many-to-many relationship between posts and tags:

| Column | Type | Description |
|--------|------|-------------|
| post_id | INTEGER | Foreign key referencing posts.id |
| tag_id | INTEGER | Foreign key referencing tags.id |

Primary key: (post_id, tag_id)

Foreign key constraints:
- `post_id` references `posts(id)` with CASCADE on delete
- `tag_id` references `tags(id)` with CASCADE on delete

### Post Metadata Table

The `post_metadata` table stores arbitrary key-value pairs associated with posts:

| Column | Type | Description |
|--------|------|-------------|
| post_id | INTEGER | Foreign key referencing posts.id |
| key | TEXT | Metadata key |
| value | TEXT | Metadata value |

Primary key: (post_id, key)

Foreign key constraints:
- `post_id` references `posts(id)` with CASCADE on delete

### Full-Text Search Virtual Table

The `posts_fts` virtual table is a FTS5 (Full-Text Search) table that enables efficient text search across post content:

| Column | Type | Description |
|--------|------|-------------|
| title | TEXT | Title of the post (indexed for search) |
| content | TEXT | Content of the post (indexed for search) |
| excerpt | TEXT | Excerpt of the post (indexed for search) |

This is a virtual table that references the `posts` table, with `content='posts'` and `content_rowid='id'`.

## Entity-Relationship Diagram

```
+---------------+       +----------------+       +-------------+
|    posts      |       |   post_tags    |       |    tags     |
+---------------+       +----------------+       +-------------+
| id (PK)       |       | post_id (PK,FK)|------>| id (PK)     |
| title         |       | tag_id (PK,FK) |<------| name        |
| slug          |<------+                |       | slug        |
| date          |       +----------------+       +-------------+
| author        |
| excerpt       |       +----------------+
| content       |       | post_metadata  |
| published     |       +----------------+
| featured      |       | post_id (PK,FK)|
| image         |<------| key (PK)       |
+---------------+       | value          |
      ^                 +----------------+
      |
      |
+---------------+
|   posts_fts   |
+---------------+
| rowid         |
| title         |
| content       |
| excerpt       |
+---------------+
```

## Design Decisions

### Use of SQLite

SQLite was chosen for this project because:
1. It's serverless and requires no separate database process
2. It's lightweight and easy to deploy
3. It provides excellent performance for read-heavy workloads
4. It supports advanced features like full-text search

### Schema Design

1. **Normalization**: The schema follows database normalization principles by separating posts, tags, and metadata into different tables.

2. **Many-to-Many Relationships**: The `post_tags` junction table implements a many-to-many relationship between posts and tags, allowing each post to have multiple tags and each tag to be associated with multiple posts.

3. **Flexible Metadata**: The `post_metadata` table allows for storing arbitrary key-value pairs with posts, providing flexibility without requiring schema changes for new metadata fields.

4. **Full-Text Search**: The FTS5 virtual table (`posts_fts`) provides efficient full-text search capabilities across post content, title, and excerpt.

5. **Triggers for FTS Synchronization**: Triggers are used to keep the FTS index in sync with the posts table, ensuring that changes to posts are automatically reflected in the search index.

### Indexing Strategy

1. **Selective Indexing**: Indexes are created only on columns that are frequently used in WHERE clauses or for sorting, to balance query performance with write performance.

2. **Compound Primary Keys**: The junction tables use compound primary keys to enforce uniqueness and provide efficient lookups.

3. **Foreign Key Constraints**: Foreign key constraints with CASCADE delete ensure referential integrity while automatically handling cleanup when parent records are deleted.

## Migrations

The database schema is managed through migrations, which are applied in sequence to create or update the database schema. Each migration is assigned a unique ID and is recorded in the `migrations` table to track which migrations have been applied.

Current migrations:
1. Initial schema (creates tables and indexes)
2. Add full-text search (creates FTS virtual table and triggers)

## Performance Considerations

1. **Query Optimization**: The repository implementation includes optimized query functions that use JOINs to fetch posts with their tags and metadata in a single database roundtrip, reducing the N+1 query problem.

2. **Connection Pooling**: A connection pool is used to manage database connections efficiently, with metrics collection to monitor usage patterns.

3. **WAL Mode**: The database is configured to use Write-Ahead Logging (WAL) mode for better concurrency and performance.

4. **Prepared Statements**: SQL queries use prepared statements for better performance and protection against SQL injection.

## Security Considerations

1. **File Permissions**: The database file is secured with appropriate file permissions to prevent unauthorized access.

2. **Input Validation**: All user input is validated and sanitized before being used in database operations.

3. **Prepared Statements**: SQL queries use prepared statements to prevent SQL injection attacks.