use sqlx::{Error, SqlitePool};

const DB_SCHEMA: &str = r#"
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

pub async fn init_schema(pool: &SqlitePool) -> Result<(), Error> {
  sqlx::query(DB_SCHEMA).execute(pool).await?;
  Ok(())
}

pub const INSERT_FILE: &str = r#"
  INSERT INTO files (path) VALUES (?)
"#;

pub const INSERT_TAG: &str = r#"
  INSERT INTO tags (name) VALUES (?)
"#;

pub const FILE_EXISTS: &str = r#"
  SELECT id FROM files WHERE id = ?
"#;

pub const TAG_EXISTS: &str = r#"
  SELECT id FROM tags WHERE id = ?
"#;

pub const INSERT_INTO_FILE_TAGS: &str = r#"
  INSERT OR IGNORE INTO file_tags (file_id, tag_id) VALUES (?, ?)
"#;

pub const GET_FILE_TAGS: &str = r#"
  SELECT t.name FROM tags t
  JOIN file_tags ft ON ft.tag_id = t.id
  JOIN files f ON f.id = ft.file_id
  WHERE f.id = ?;
"#;

pub const GET_ALL_TAGS_FOR_ALL_FILES: &str = r#"
  SELECT f.id, f.path, GROUP_CONCAT(t1.name, ',' ORDER BY t1.name) AS all_tags
  FROM files f
  JOIN file_tags ft1 ON ft1.file_id = f.id
  JOIN tags t1 ON t1.id = ft1.tag_id
  GROUP BY f.id, f.path;
"#;
