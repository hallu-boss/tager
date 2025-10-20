pub const DB_SCHEMA: &str = r#"
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

pub const INSERT_INTO_FILE_TAGS: &str =  r#"
  INSERT OR IGNORE INTO file_tags (file_id, tag_id) VALUES (?, ?)
"#;