use sqlx::{Error, SqlitePool};

mod queries;
use queries::*;

pub struct Database {
  pool: SqlitePool
}

impl Database {
  pub async fn new_in_memory() -> Result<Self, Error> {
    let pool = SqlitePool::connect("sqlite::memory:").await?;

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

  pub async fn assign_tag_to_file(&self, tag_id: i64, file_id: i64) -> Result<i64, Error> {
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

    Ok(res.last_insert_rowid())
  }
}

#[cfg(test)]
mod tests;