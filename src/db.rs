use crate::config::{TAGER_DB_NAME, TAGER_DIR_NAME};
use sqlx::{Row, SqlitePool};
use std::path::{Path, PathBuf};
use thiserror::Error;
use walkdir::WalkDir;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("Błąd SQL: {0}")]
    Sql(#[from] sqlx::Error),
}

/// Struktura zarządzająca bazą danych tagów.
#[derive(Clone)]
pub struct Database {
    pool: SqlitePool,
    root_dir: PathBuf,
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
    pub async fn new_in_memory() -> Result<Self, DbError> {
        let pool = SqlitePool::connect("sqlite::memory:").await?;
        let db = Self {
            pool,
            root_dir: PathBuf::from("."),
        };
        db.init_schema().await?;
        Ok(db)
    }

    pub async fn new_file<P: AsRef<Path>>(
        root_dir: P,
        tager_dir: Option<P>,
    ) -> Result<Self, DbError> {
        let root_path = root_dir.as_ref();

        let tager_path = if let Some(dir) = tager_dir {
            dir.as_ref().to_path_buf()
        } else {
            root_path.join(TAGER_DIR_NAME)
        };

        let db_file_path = tager_path.join(TAGER_DB_NAME);

        if let Some(parent) = db_file_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| DbError::Sql(sqlx::Error::Io(e)))?;
        }

        let url = format!("sqlite://{}?mode=rwc", db_file_path.display());
        let pool = SqlitePool::connect(&url).await?;
        let db = Self {
            pool,
            root_dir: root_path.to_path_buf(),
        };
        db.init_schema().await?;
        Ok(db)
    }

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

    pub async fn assign_tag_to_file<P: AsRef<Path>>(
        &self,
        file_path: P,
        tag_name: &str,
    ) -> Result<(), DbError> {
        let file_path = file_path.as_ref().to_string_lossy();

        let file_id: i64 = sqlx::query(
            r#"
            INSERT INTO files (path) VALUES (?)
            ON CONFLICT(path) DO UPDATE SET path=excluded.path
            RETURNING id;
            "#,
        )
        .bind(&*file_path)
        .fetch_one(&self.pool)
        .await?
        .get("id");

        let tag_id: i64 = sqlx::query(
            r#"
            INSERT INTO tags (name) VALUES (?)
            ON CONFLICT(name) DO UPDATE SET name=excluded.name
            RETURNING id;
            "#,
        )
        .bind(tag_name)
        .fetch_one(&self.pool)
        .await?
        .get("id");

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
            WHERE f.path = ?;
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

    pub async fn get_untaged_files(
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
            {};
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
          -- ft1 + t1 filtrują pliki po wskazanym tagu

          JOIN file_tags ft2 ON ft2.file_id = f.id
          JOIN tags t2 ON t2.id = ft2.tag_id
          -- ft2 + t2 pobierają WSZYSTKIE tagi przypisane do danego pliku

          WHERE t1.name = ?
          GROUP BY f.id, f.path
          {};
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

    pub async fn rebuild(&self) -> Result<usize, DbError> {
        let mut added_count = 0;

        for entry in WalkDir::new(&self.root_dir)
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| {
                if e.file_type().is_dir() {
                    if let Some(name) = e.path().file_name() {
                        return name != ".tager";
                    }
                }
                true
            })
            .filter_map(|e| e.ok())
        {
            if !entry.file_type().is_file() {
                continue;
            }

            let abs_path = entry.path();

            let rel_path = match abs_path.strip_prefix(&self.root_dir) {
                Ok(p) => p,
                Err(_) => continue,
            };

            let rel_path = rel_path.to_string_lossy();

            let result = sqlx::query(
                r#"
            INSERT OR IGNORE INTO files (path) VALUES (?)
            "#,
            )
            .bind(&*rel_path)
            .execute(&self.pool)
            .await?;

            if result.rows_affected() > 0 {
                added_count += 1;
            }
        }

        Ok(added_count)
    }
}
