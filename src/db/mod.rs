use sqlx::{Row, SqlitePool};
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("Błąd SQL: {0}")]
    Sql(#[from] sqlx::Error),
}

/// Core database structure focused on database operations only.
#[derive(Clone)]
pub struct Database {
    pool: SqlitePool,
}

pub enum FilesOrderBy {
    Id,
    Path,
}

#[derive(Debug, Clone)]
pub struct FileWithTags {
    pub id: i64,
    pub path: String,
    pub tags: Vec<String>,
}

impl Database {
    /// Create a new in-memory database instance.
    pub async fn new_in_memory() -> Result<Self, DbError> {
        let pool = SqlitePool::connect("sqlite::memory:").await?;
        let db = Self { pool };
        db.init_schema().await?;
        Ok(db)
    }

    /// Create a new file-based database instance.
    pub async fn new_file(db_path: &Path) -> Result<Self, DbError> {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| DbError::Sql(sqlx::Error::Io(e)))?;
        }

        let url = format!("sqlite://{}?mode=rwc", db_path.display());
        let pool = SqlitePool::connect(&url).await?;
        let db = Self { pool };
        db.init_schema().await?;
        Ok(db)
    }

    /// Initialize database schema.
    async fn init_schema(&self) -> Result<(), DbError> {
        let schema = r#"
        CREATE TABLE IF NOT EXISTS files (
            id INTEGER PRIMARY KEY,
            path TEXT NOT NULL UNIQUE
        );

        CREATE TABLE IF NOT EXISTS tags (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL UNIQUE
        );

        CREATE TABLE IF NOT EXISTS file_tags (
            file_id INTEGER NOT NULL,
            tag_id INTEGER NOT NULL,
            PRIMARY KEY (file_id, tag_id),
            FOREIGN KEY (file_id) REFERENCES files(id) ON DELETE CASCADE,
            FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
        );
        "#;

        sqlx::query(schema).execute(&self.pool).await?;
        Ok(())
    }

    /// Add a file to the database.
    pub async fn add_file(&self, relative_path: &str) -> Result<i64, DbError> {
        let result = sqlx::query(
            r#"
            INSERT INTO files (path) VALUES (?)
            ON CONFLICT(path) DO UPDATE SET path = excluded.path
            RETURNING id
            "#,
        )
        .bind(relative_path)
        .fetch_one(&self.pool)
        .await?;

        Ok(result.get("id"))
    }

    /// Add a file only if it doesn't exist (returns true if added).
    pub async fn add_file_if_not_exists(&self, relative_path: &str) -> Result<bool, DbError> {
        let result = sqlx::query(
            r#"
            INSERT OR IGNORE INTO files (path) VALUES (?)
            "#,
        )
        .bind(relative_path)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Check if a file exists in the database.
    pub async fn file_exists(&self, file_id: i64) -> Result<bool, DbError> {
        let exists: Option<(i64,)> = sqlx::query_as("SELECT id FROM files WHERE id = ?")
            .bind(file_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(exists.is_some())
    }

    /// Get or create a tag and return its ID.
    async fn get_or_create_tag(&self, tag_name: &str) -> Result<i64, DbError> {
        let tag_id: i64 = sqlx::query(
            r#"
            INSERT INTO tags (name) VALUES (?)
            ON CONFLICT(name) DO UPDATE SET name = excluded.name
            RETURNING id
            "#,
        )
        .bind(tag_name)
        .fetch_one(&self.pool)
        .await?
        .get("id");

        Ok(tag_id)
    }

    /// Assign a tag to a file by file ID.
    pub async fn assign_tag_to_file_by_id(
        &self,
        file_id: i64,
        tag_name: &str,
    ) -> Result<(), DbError> {
        if !self.file_exists(file_id).await? {
            return Err(sqlx::Error::RowNotFound.into());
        }

        let tag_id = self.get_or_create_tag(tag_name).await?;

        sqlx::query(
            r#"
            INSERT OR IGNORE INTO file_tags (file_id, tag_id) VALUES (?, ?)
            "#,
        )
        .bind(file_id)
        .bind(tag_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get all tags for a specific file path.
    pub async fn get_tags_for_file<P: AsRef<Path>>(
        &self,
        file_path: P,
    ) -> Result<Vec<String>, DbError> {
        let file_path = file_path.as_ref().to_string_lossy();
        let rows = sqlx::query(
            r#"
            SELECT t.name
            FROM tags t
            JOIN file_tags ft ON ft.tag_id = t.id
            JOIN files f ON f.id = ft.file_id
            WHERE f.path = ?
            "#,
        )
        .bind(&*file_path)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| r.get::<String, _>("name"))
            .collect())
    }

    /// Get all files that have no tags assigned.
    pub async fn get_untagged_files(
        &self,
        order_by: Option<FilesOrderBy>,
    ) -> Result<Vec<(i64, String)>, DbError> {
        let order_clause = match order_by {
            Some(FilesOrderBy::Id) => "ORDER BY f.id",
            Some(FilesOrderBy::Path) => "ORDER BY f.path",
            None => "",
        };

        let query = format!(
            r#"
            SELECT f.id, f.path
            FROM files f
            LEFT JOIN file_tags ft ON f.id = ft.file_id
            WHERE ft.file_id IS NULL
            {}
            "#,
            order_clause
        );

        let rows = sqlx::query(&query).fetch_all(&self.pool).await?;

        Ok(rows
            .into_iter()
            .map(|r| {
                let id = r.get::<i64, _>("id");
                let path = r.get::<String, _>("path");
                (id, path)
            })
            .collect())
    }

    /// Get all files that have a specific tag.
    pub async fn get_files_for_tag(
        &self,
        tag_name: &str,
        order_by: Option<FilesOrderBy>,
    ) -> Result<Vec<FileWithTags>, DbError> {
        let order_clause = match order_by {
            Some(FilesOrderBy::Id) => "ORDER BY f.id",
            Some(FilesOrderBy::Path) => "ORDER BY f.path",
            None => "",
        };

        let query = format!(
            r#"
            SELECT f.id, f.path, GROUP_CONCAT(t2.name, ',' ORDER BY t2.name) AS all_tags
            FROM files f
            JOIN file_tags ft1 ON ft1.file_id = f.id
            JOIN tags t1 ON t1.id = ft1.tag_id
            JOIN file_tags ft2 ON ft2.file_id = f.id
            JOIN tags t2 ON t2.id = ft2.tag_id
            WHERE t1.name = ?
            GROUP BY f.id, f.path
            {}
            "#,
            order_clause
        );

        let rows = sqlx::query(&query)
            .bind(tag_name)
            .fetch_all(&self.pool)
            .await?;

        let files = rows
            .into_iter()
            .map(|r| {
                let id = r.get::<i64, _>("id");
                let path = r.get::<String, _>("path");
                let all_tags_str = r.get::<Option<String>, _>("all_tags").unwrap_or_default();
                let tags = all_tags_str
                    .split(',')
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>();
                FileWithTags { id, path, tags }
            })
            .collect();

        Ok(files)
    }

    /// Get the connection pool (for advanced operations).
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}

#[cfg(test)]
mod tests;