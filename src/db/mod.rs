use sqlx::{Error, Row, SqlitePool};

mod queries;
use queries::*;

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
        let pool = SqlitePool::connect("sqlite::memory:").await?;

        let db = Self { pool };
        db.init_schema().await?;

        Ok(db)
    }

    pub async fn from_file() -> Result<Self, Error> {
        let url = format!("sqlite://{}?mode=rwc", "/home/pawel/Desktop/tager/test.db");
        let pool = SqlitePool::connect(&url).await?;
        let db = Self { pool };
        db.init_schema().await?;
        Ok(db)
    }

    async fn init_schema(&self) -> Result<(), Error> {
        sqlx::query(DB_SCHEMA).execute(&self.pool).await?;

        Ok(())
    }

    pub async fn add_file(&self, path: &str) -> Result<i64, Error> {
        let result = sqlx::query(INSERT_FILE)
            .bind(path)
            .execute(&self.pool)
            .await?;

        Ok(result.last_insert_rowid())
    }

    pub async fn add_tag(&self, name: &str) -> Result<i64, Error> {
        let result = sqlx::query(INSERT_TAG)
            .bind(name)
            .execute(&self.pool)
            .await?;

        Ok(result.last_insert_rowid())
    }

    async fn file_exists(&self, file_id: i64) -> Result<bool, Error> {
        let file_exists = sqlx::query(FILE_EXISTS)
            .bind(file_id)
            .fetch_optional(&self.pool)
            .await?;

        if file_exists.is_none() {
            return Ok(false);
        }

        Ok(true)
    }

    async fn tag_exists(&self, tag_id: i64) -> Result<bool, Error> {
        let tag_exists = sqlx::query(TAG_EXISTS)
            .bind(tag_id)
            .fetch_optional(&self.pool)
            .await?;

        if tag_exists.is_none() {
            return Ok(false);
        }

        Ok(true)
    }

    pub async fn assign_tag_to_file(&self, tag_id: i64, file_id: i64) -> Result<(), Error> {
        let tag_exists = self.tag_exists(tag_id).await.unwrap();
        let file_exists = self.file_exists(file_id).await.unwrap();
        if !tag_exists || !file_exists {
            return Err(sqlx::Error::RowNotFound.into());
        }

        let res = sqlx::query(INSERT_INTO_FILE_TAGS)
            .bind(file_id)
            .bind(tag_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn get_file_tags(&self, file_id: i64) -> Result<Vec<String>, Error> {
        let rows = sqlx::query(GET_FILE_TAGS)
            .bind(file_id)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows
            .into_iter()
            .map(|r| r.get::<String, _>("name"))
            .collect())
    }

    pub async fn get_all_files(&self) -> Result<Vec<FileWithTags>, Error> {
        let rows = sqlx::query(GET_ALL_TAGS_FOR_ALL_FILES)
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
}

#[cfg(test)]
mod tests;
