use std::{path::Path, str::FromStr};

use sqlx::{sqlite::{SqliteConnectOptions, SqlitePoolOptions}, Error, Row, SqlitePool};

async fn init_schema(pool: &SqlitePool) -> Result<(), Error> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS files (
            id INTEGER PRIMARY KEY,
            path TEXT NOT NULL UNIQUE
        );"
    ).execute(pool).await?;
    
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS tags (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL UNIQUE
        );"
    ).execute(pool).await?;
    
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS file_tags (
            file_id INTEGER NOT NULL,
            tag_id INTEGER NOT NULL,
            PRIMARY KEY (file_id, tag_id),
            FOREIGN KEY (file_id) REFERENCES files(id) ON DELETE CASCADE,
            FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
        );"
    ).execute(pool).await?;
    
    Ok(())
}

pub struct Database {
    pool: SqlitePool,
}

#[derive(Debug)]
pub struct FileWithTags {
    pub id: i64,
    pub path: String,
    pub tags: Vec<String>,
}

impl Database {
    pub async fn new_in_memory() -> Result<Self, Error> {
        let options = SqliteConnectOptions::from_str("sqlite::memory:")?
        .foreign_keys(true);
    
    let pool = SqlitePoolOptions::new()
        .connect_with(options)
        .await?;
    
    init_schema(&pool).await?;
    Ok(Self { pool })
    }

    // TODO: parametrize source file
    pub async fn from_file() -> Result<Self, Error> {
        let url = format!("sqlite://{}?mode=rwc", "/home/pawel/Desktop/tager/test.db");
        let pool = SqlitePool::connect(&url).await?;
        init_schema(&pool).await?;
        let db = Self { pool };
        Ok(db)
    }

    pub async fn add_file(&self, path: &Path) -> Result<i64, Error> {
        let res = sqlx::query(
            r#"
        INSERT OR IGNORE INTO files (path) VALUES (?)
        "#,
        )
        .bind(path.to_string_lossy())
        .execute(&self.pool)
        .await?;

        if res.last_insert_rowid() != 0 {
            Ok(res.last_insert_rowid())
        } else {
            // plik już istnieje, pobierz jego id
            let id: i64 = sqlx::query("SELECT id FROM files WHERE path = ?")
                .bind(path.to_string_lossy())
                .fetch_one(&self.pool)
                .await?
                .get("id");
            Ok(id)
        }
    }

    pub async fn remove_file(&self, path: &Path) -> Result<u64, Error> {
        let res = sqlx::query("DELETE FROM files WHERE path = ?")
            .bind(path.to_string_lossy())
            .execute(&self.pool)
            .await?;

        Ok(res.rows_affected())
    }

    pub async fn add_tag(&self, name: &str) -> Result<i64, Error> {
        let res = sqlx::query(
            r#"
        INSERT OR IGNORE INTO tags (name) VALUES (?)
        "#,
        )
        .bind(name)
        .execute(&self.pool)
        .await?;

        if res.last_insert_rowid() != 0 {
            Ok(res.last_insert_rowid())
        } else {
            // tag już istnieje, pobierz jego id
            let id: i64 = sqlx::query("SELECT id FROM tags WHERE name = ?")
                .bind(name)
                .fetch_one(&self.pool)
                .await?
                .get("id");
            Ok(id)
        }
    }

    pub async fn assign_tag_to_file(&self, tag_name: &str, file_path: &Path) -> Result<(), Error> {
        // Pobierz lub utwórz plik i tag
        let file_id = self.add_file(file_path).await?;
        let tag_id = self.add_tag(tag_name).await?;

        // Wstaw do file_tags, jeśli nie istnieje
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

    pub async fn get_file_tags(&self, file_path: &Path) -> Result<Vec<String>, Error> {
        let rows = sqlx::query(
            r#"
            SELECT t.name FROM tags t
            JOIN file_tags ft ON ft.tag_id = t.id
            JOIN files f ON f.id = ft.file_id
            WHERE f.path = ?;
            "#,
        )
        .bind(file_path.to_string_lossy())
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| r.get::<String, _>("name"))
            .collect())
    }

    pub async fn get_tag_files(&self, tag_name: &str) -> Result<Vec<String>, Error> {
        let rows = sqlx::query(
            r#"
            SELECT f.path FROM tags t
            JOIN file_tags ft ON ft.tag_id = t.id
            JOIN files f ON f.id = ft.file_id
            WHERE t.name = ?;
            "#,
        )
        .bind(tag_name)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| r.get::<String, _>("path"))
            .collect())
    }
}

#[cfg(test)]
mod tests;
