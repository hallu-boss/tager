use sqlx::{Pool, Row, Sqlite, migrate::MigrateDatabase, sqlite::SqlitePoolOptions};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::tm::db::{DBFile, DBTag, DbError};
use crate::{i64_to_system_time, system_time_to_i64};

pub struct Database {
    pool: Pool<Sqlite>,
}

impl Database {
    /// Tworzy nową bazę danych w pamięci
    pub async fn new_in_memory() -> Result<Self, DbError> {
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect("sqlite::memory:")
            .await
            .map_err(DbError::Sql)?;

        let db = Self { pool };
        db.init_schema().await?;

        Ok(db)
    }

    /// Tworzy nową bazę danych z pliku
    pub async fn new_from_file<P: AsRef<Path>>(db_path: P) -> Result<Self, DbError> {
        let db_path = db_path.as_ref();

        // Utwórz katalog nadrzędny jeśli nie istnieje
        if let Some(parent) = db_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent).map_err(DbError::Io)?;
            }
        }

        // Utwórz bazę danych jeśli nie istnieje
        let db_url = format!("sqlite://{}", db_path.display());
        if !sqlx::Sqlite::database_exists(&db_url)
            .await
            .unwrap_or(false)
        {
            sqlx::Sqlite::create_database(&db_url)
                .await
                .map_err(DbError::Sql)?;
        }

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&db_url)
            .await
            .map_err(DbError::Sql)?;

        let db = Self { pool };
        db.init_schema().await?;

        Ok(db)
    }

    /// Inicjalizuje schemat bazy danych
    pub async fn init_schema(&self) -> Result<(), DbError> {
        let queries = vec![
            // Tabela files
            r#"
            CREATE TABLE IF NOT EXISTS files (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                path TEXT NOT NULL,
                size INTEGER NOT NULL,
                content_hash TEXT NOT NULL,
                last_modified INTEGER NOT NULL,
                last_accessed INTEGER NOT NULL,
                created INTEGER NOT NULL
            )
            "#,
            // Tabela tags
            r#"
            CREATE TABLE IF NOT EXISTS tags (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE
            )
            "#,
            // Tabela file_tags
            r#"
            CREATE TABLE IF NOT EXISTS file_tags (
                file_id INTEGER NOT NULL,
                tag_id INTEGER NOT NULL,
                PRIMARY KEY (file_id, tag_id),
                FOREIGN KEY (file_id) REFERENCES files(id) ON DELETE CASCADE,
                FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
            )
            "#,
            // Indeksy dla lepszej wydajności
            r#"
            CREATE INDEX IF NOT EXISTS idx_files_path ON files(path)
            "#,
            r#"
            CREATE INDEX IF NOT EXISTS idx_files_last_modified ON files(last_modified)
            "#,
            r#"
            CREATE INDEX IF NOT EXISTS idx_file_tags_file_id ON file_tags(file_id)
            "#,
            r#"
            CREATE INDEX IF NOT EXISTS idx_file_tags_tag_id ON file_tags(tag_id)
            "#,
        ];

        for query in queries {
            sqlx::query(query)
                .execute(&self.pool)
                .await
                .map_err(DbError::Sql)?;
        }

        Ok(())
    }

    // ========== CRUD OPERATIONS FOR FILES ==========

    /// Tworzy nowy rekord pliku
    pub async fn create_file(
        &self,
        path: PathBuf,
        size: u64,
        content_hash: String,
        last_modified: SystemTime,
        last_accessed: SystemTime,
        created: SystemTime,
    ) -> Result<i64, DbError> {
        let path_str = path.to_string_lossy().to_string();

        let row = sqlx::query(
            r#"
            INSERT INTO files (path, size, content_hash, last_modified, last_accessed, created)
            VALUES (?, ?, ?, ?, ?, ?)
            RETURNING id
            "#,
        )
        .bind(path_str)
        .bind(size as i64)
        .bind(content_hash)
        .bind(system_time_to_i64(last_modified))
        .bind(system_time_to_i64(last_accessed))
        .bind(system_time_to_i64(created))
        .fetch_one(&self.pool)
        .await
        .map_err(DbError::Sql)?;

        Ok(row.get::<i64, _>("id"))
    }

    /// Pobiera plik po ID
    pub async fn get_file(&self, id: i64) -> Result<Option<DBFile>, DbError> {
        let row = sqlx::query(
            r#"
            SELECT 
                id,
                path,
                size,
                content_hash,
                last_modified,
                last_accessed,
                created
            FROM files 
            WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(DbError::Sql)?;

        match row {
            Some(row) => {
                let file = DBFile {
                    id: row.get("id"),
                    path: PathBuf::from(row.get::<String, _>("path")),
                    size: row.get::<i64, _>("size") as u64,
                    content_hash: row.get("content_hash"),
                    last_modified: i64_to_system_time(row.get("last_modified")),
                    last_accessed: i64_to_system_time(row.get("last_accessed")),
                    created: i64_to_system_time(row.get("created")),
                };
                Ok(Some(file))
            }
            None => Ok(None),
        }
    }

    /// Pobiera wszystkie pliki (z opcjonalnymi parametrami paginacji)
    pub async fn get_all_files(
        &self,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<DBFile>, DbError> {
        let mut query = String::from(
            "SELECT 
                id,
                path,
                size,
                content_hash,
                last_modified,
                last_accessed,
                created
            FROM files 
            ORDER BY created DESC",
        );

        if let Some(limit) = limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }
        if let Some(offset) = offset {
            query.push_str(&format!(" OFFSET {}", offset));
        }

        let rows = sqlx::query(&query)
            .fetch_all(&self.pool)
            .await
            .map_err(DbError::Sql)?;

        let files: Vec<DBFile> = rows
            .into_iter()
            .map(|row| DBFile {
                id: row.get("id"),
                path: PathBuf::from(row.get::<String, _>("path")),
                size: row.get::<i64, _>("size") as u64,
                content_hash: row.get("content_hash"),
                last_modified: i64_to_system_time(row.get("last_modified")),
                last_accessed: i64_to_system_time(row.get("last_accessed")),
                created: i64_to_system_time(row.get("created")),
            })
            .collect();

        Ok(files)
    }

    /// Pobiera pliki po ścieżce (może zwrócić wiele wyników)
    pub async fn get_files_by_path(&self, path: PathBuf) -> Result<Vec<DBFile>, DbError> {
        let path_str = path.to_string_lossy().to_string();

        let rows = sqlx::query(
            r#"
            SELECT 
                id,
                path,
                size,
                content_hash,
                last_modified,
                last_accessed,
                created
            FROM files 
            WHERE path = ?
            ORDER BY last_modified DESC
            "#,
        )
        .bind(path_str)
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::Sql)?;

        let files: Vec<DBFile> = rows
            .into_iter()
            .map(|row| DBFile {
                id: row.get("id"),
                path: PathBuf::from(row.get::<String, _>("path")),
                size: row.get::<i64, _>("size") as u64,
                content_hash: row.get("content_hash"),
                last_modified: i64_to_system_time(row.get("last_modified")),
                last_accessed: i64_to_system_time(row.get("last_accessed")),
                created: i64_to_system_time(row.get("created")),
            })
            .collect();

        Ok(files)
    }

    /// Aktualizuje istniejący plik
    pub async fn update_file(
        &self,
        id: i64,
        path: Option<PathBuf>,
        size: Option<u64>,
        content_hash: Option<String>,
        last_modified: Option<SystemTime>,
        last_accessed: Option<SystemTime>,
        created: Option<SystemTime>,
    ) -> Result<bool, DbError> {
        // Sprawdź czy plik istnieje
        if !self.file_exists(id).await? {
            return Ok(false);
        }

        let mut updates = Vec::new();
        let mut query = "UPDATE files SET ".to_string();

        // Zbierz zmiany
        if path.is_some() {
            updates.push("path = ?");
        }
        if size.is_some() {
            updates.push("size = ?");
        }
        if content_hash.is_some() {
            updates.push("content_hash = ?");
        }
        if last_modified.is_some() {
            updates.push("last_modified = ?");
        }
        if last_accessed.is_some() {
            updates.push("last_accessed = ?");
        }
        if created.is_some() {
            updates.push("created = ?");
        }

        if updates.is_empty() {
            return Ok(true); // Brak zmian
        }

        query.push_str(&updates.join(", "));
        query.push_str(" WHERE id = ?");

        // Wykonaj zapytanie
        let mut query_builder = sqlx::query(&query);

        // Binduj parametry w odpowiedniej kolejności
        if let Some(p) = path {
            query_builder = query_builder.bind(p.to_string_lossy().to_string());
        }
        if let Some(s) = size {
            query_builder = query_builder.bind(s as i64);
        }
        if let Some(ch) = content_hash {
            query_builder = query_builder.bind(ch);
        }
        if let Some(lm) = last_modified {
            query_builder = query_builder.bind(system_time_to_i64(lm));
        }
        if let Some(la) = last_accessed {
            query_builder = query_builder.bind(system_time_to_i64(la));
        }
        if let Some(c) = created {
            query_builder = query_builder.bind(system_time_to_i64(c));
        }

        query_builder = query_builder.bind(id);

        let result = query_builder
            .execute(&self.pool)
            .await
            .map_err(DbError::Sql)?;

        Ok(result.rows_affected() > 0)
    }

    /// Usuwa plik po ID
    pub async fn delete_file(&self, id: i64) -> Result<bool, DbError> {
        let result = sqlx::query(
            r#"
            DELETE FROM files 
            WHERE id = ?
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(DbError::Sql)?;

        Ok(result.rows_affected() > 0)
    }

    /// Usuwa pliki po ścieżce (usuwa wszystkie pasujące)
    pub async fn delete_files_by_path(&self, path: PathBuf) -> Result<u64, DbError> {
        let path_str = path.to_string_lossy().to_string();

        let result = sqlx::query(
            r#"
            DELETE FROM files 
            WHERE path = ?
            "#,
        )
        .bind(path_str)
        .execute(&self.pool)
        .await
        .map_err(DbError::Sql)?;

        Ok(result.rows_affected())
    }

    /// Sprawdza czy plik istnieje po ID
    pub async fn file_exists(&self, id: i64) -> Result<bool, DbError> {
        let result: Option<(i64,)> = sqlx::query_as(
            r#"
            SELECT 1 FROM files WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(DbError::Sql)?;

        Ok(result.is_some())
    }

    /// Pobiera pliki po hashu (przydatne do znajdowania duplikatów)
    pub async fn get_files_by_hash(&self, content_hash: &str) -> Result<Vec<DBFile>, DbError> {
        let rows = sqlx::query(
            r#"
            SELECT 
                id,
                path,
                size,
                content_hash,
                last_modified,
                last_accessed,
                created
            FROM files 
            WHERE content_hash = ?
            ORDER BY path
            "#,
        )
        .bind(content_hash)
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::Sql)?;

        let files: Vec<DBFile> = rows
            .into_iter()
            .map(|row| DBFile {
                id: row.get("id"),
                path: PathBuf::from(row.get::<String, _>("path")),
                size: row.get::<i64, _>("size") as u64,
                content_hash: row.get("content_hash"),
                last_modified: i64_to_system_time(row.get("last_modified")),
                last_accessed: i64_to_system_time(row.get("last_accessed")),
                created: i64_to_system_time(row.get("created")),
            })
            .collect();

        Ok(files)
    }

    /// Pobiera ilość wszystkich plików w bazie
    pub async fn count_files(&self) -> Result<i64, DbError> {
        let row: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM files
            "#,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(DbError::Sql)?;

        Ok(row.0)
    }

    // ========== CRUD OPERATIONS FOR TAGS ==========

    /// Tworzy nowy tag (jeśli nie istnieje) i zwraca jego ID
    pub async fn create_tag(&self, name: &str) -> Result<i64, DbError> {
        // Sprawdź czy tag już istnieje
        if let Some(existing_id) = self.get_tag_id_by_name(name).await? {
            return Ok(existing_id);
        }

        let row = sqlx::query(
            r#"
        INSERT INTO tags (name)
        VALUES (?)
        RETURNING id
        "#,
        )
        .bind(name)
        .fetch_one(&self.pool)
        .await
        .map_err(DbError::Sql)?;

        Ok(row.get::<i64, _>("id"))
    }

    /// Pobiera tag po ID
    pub async fn get_tag(&self, id: i64) -> Result<Option<DBTag>, DbError> {
        let row = sqlx::query(
            r#"
        SELECT id, name
        FROM tags 
        WHERE id = ?
        "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(DbError::Sql)?;

        match row {
            Some(row) => {
                let tag = DBTag {
                    id: row.get("id"),
                    name: row.get("name"),
                };
                Ok(Some(tag))
            }
            None => Ok(None),
        }
    }

    /// Pobiera tag po nazwie
    pub async fn get_tag_by_name(&self, name: &str) -> Result<Option<DBTag>, DbError> {
        let row = sqlx::query(
            r#"
        SELECT id, name
        FROM tags 
        WHERE name = ?
        "#,
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await
        .map_err(DbError::Sql)?;

        match row {
            Some(row) => {
                let tag = DBTag {
                    id: row.get("id"),
                    name: row.get("name"),
                };
                Ok(Some(tag))
            }
            None => Ok(None),
        }
    }

    /// Pobiera ID tagu po nazwie
    pub async fn get_tag_id_by_name(&self, name: &str) -> Result<Option<i64>, DbError> {
        let row = sqlx::query(
            r#"
        SELECT id
        FROM tags 
        WHERE name = ?
        "#,
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await
        .map_err(DbError::Sql)?;

        match row {
            Some(row) => Ok(Some(row.get("id"))),
            None => Ok(None),
        }
    }

    /// Pobiera wszystkie tagi
    pub async fn get_all_tags(&self) -> Result<Vec<DBTag>, DbError> {
        let rows = sqlx::query(
            r#"
        SELECT id, name
        FROM tags 
        ORDER BY name
        "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::Sql)?;

        let tags: Vec<DBTag> = rows
            .into_iter()
            .map(|row| DBTag {
                id: row.get("id"),
                name: row.get("name"),
            })
            .collect();

        Ok(tags)
    }

    /// Aktualizuje nazwę tagu
    pub async fn update_tag(&self, id: i64, new_name: &str) -> Result<bool, DbError> {
        // Sprawdź czy tag istnieje
        if !self.tag_exists(id).await? {
            return Ok(false);
        }

        // Sprawdź czy nowa nazwa nie jest już używana
        if let Some(existing_tag) = self.get_tag_by_name(new_name).await? {
            if existing_tag.id != id {
                return Err(DbError::OperationFailed(
                    "Tag o podanej nazwie już istnieje".to_string(),
                ));
            }
        }

        let result = sqlx::query(
            r#"
        UPDATE tags 
        SET name = ?
        WHERE id = ?
        "#,
        )
        .bind(new_name)
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(DbError::Sql)?;

        Ok(result.rows_affected() > 0)
    }

    /// Usuwa tag po ID
    pub async fn delete_tag(&self, id: i64) -> Result<bool, DbError> {
        // Relacje w file_tags są skonfigurowane z ON DELETE CASCADE,
        // więc powiązania zostaną automatycznie usunięte
        let result = sqlx::query(
            r#"
        DELETE FROM tags 
        WHERE id = ?
        "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(DbError::Sql)?;

        Ok(result.rows_affected() > 0)
    }

    /// Usuwa tag po nazwie
    pub async fn delete_tag_by_name(&self, name: &str) -> Result<bool, DbError> {
        let result = sqlx::query(
            r#"
        DELETE FROM tags 
        WHERE name = ?
        "#,
        )
        .bind(name)
        .execute(&self.pool)
        .await
        .map_err(DbError::Sql)?;

        Ok(result.rows_affected() > 0)
    }

    /// Sprawdza czy tag istnieje po ID
    pub async fn tag_exists(&self, id: i64) -> Result<bool, DbError> {
        let result: Option<(i64,)> = sqlx::query_as(
            r#"
        SELECT 1 FROM tags WHERE id = ?
        "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(DbError::Sql)?;

        Ok(result.is_some())
    }

    /// Sprawdza czy tag istnieje po nazwie
    pub async fn tag_exists_by_name(&self, name: &str) -> Result<bool, DbError> {
        let result: Option<(i64,)> = sqlx::query_as(
            r#"
        SELECT 1 FROM tags WHERE name = ?
        "#,
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await
        .map_err(DbError::Sql)?;

        Ok(result.is_some())
    }

    /// Pobiera ilość wszystkich tagów w bazie
    pub async fn count_tags(&self) -> Result<i64, DbError> {
        let row: (i64,) = sqlx::query_as(
            r#"
        SELECT COUNT(*) FROM tags
        "#,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(DbError::Sql)?;

        Ok(row.0)
    }

    // ========== OPERATIONS FOR FILE-TAG RELATIONSHIPS ==========

    /// Dodaje tag do pliku
    pub async fn add_tag_to_file(&self, file_id: i64, tag_id: i64) -> Result<bool, DbError> {
        // Sprawdź czy plik i tag istnieją
        if !self.file_exists(file_id).await? || !self.tag_exists(tag_id).await? {
            return Ok(false);
        }

        // Sprawdź czy powiązanie już istnieje
        if self.file_has_tag(file_id, tag_id).await? {
            return Ok(true); // Powiązanie już istnieje
        }

        let result = sqlx::query(
            r#"
        INSERT INTO file_tags (file_id, tag_id)
        VALUES (?, ?)
        "#,
        )
        .bind(file_id)
        .bind(tag_id)
        .execute(&self.pool)
        .await
        .map_err(DbError::Sql)?;

        Ok(result.rows_affected() > 0)
    }

    /// Dodaje tag do pliku po nazwie tagu
    pub async fn add_tag_by_name_to_file(
        &self,
        file_id: i64,
        tag_name: &str,
    ) -> Result<bool, DbError> {
        // Pobierz lub utwórz tag
        let tag_id = self.create_tag(tag_name).await?;

        // Dodaj powiązanie
        self.add_tag_to_file(file_id, tag_id).await
    }

    /// Usuwa tag z pliku
    pub async fn remove_tag_from_file(&self, file_id: i64, tag_id: i64) -> Result<bool, DbError> {
        let result = sqlx::query(
            r#"
        DELETE FROM file_tags 
        WHERE file_id = ? AND tag_id = ?
        "#,
        )
        .bind(file_id)
        .bind(tag_id)
        .execute(&self.pool)
        .await
        .map_err(DbError::Sql)?;

        Ok(result.rows_affected() > 0)
    }

    /// Usuwa tag z pliku po nazwie tagu
    pub async fn remove_tag_by_name_from_file(
        &self,
        file_id: i64,
        tag_name: &str,
    ) -> Result<bool, DbError> {
        if let Some(tag_id) = self.get_tag_id_by_name(tag_name).await? {
            self.remove_tag_from_file(file_id, tag_id).await
        } else {
            Ok(false) // Tag nie istnieje
        }
    }

    /// Sprawdza czy plik ma określony tag
    pub async fn file_has_tag(&self, file_id: i64, tag_id: i64) -> Result<bool, DbError> {
        let result: Option<(i64,)> = sqlx::query_as(
            r#"
        SELECT 1 FROM file_tags 
        WHERE file_id = ? AND tag_id = ?
        "#,
        )
        .bind(file_id)
        .bind(tag_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(DbError::Sql)?;

        Ok(result.is_some())
    }

    /// Pobiera wszystkie tagi dla danego pliku
    pub async fn get_tags_for_file(&self, file_id: i64) -> Result<Vec<DBTag>, DbError> {
        let rows = sqlx::query(
            r#"
        SELECT t.id, t.name
        FROM tags t
        INNER JOIN file_tags ft ON t.id = ft.tag_id
        WHERE ft.file_id = ?
        ORDER BY t.name
        "#,
        )
        .bind(file_id)
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::Sql)?;

        let tags: Vec<DBTag> = rows
            .into_iter()
            .map(|row| DBTag {
                id: row.get("id"),
                name: row.get("name"),
            })
            .collect();

        Ok(tags)
    }

    /// Pobiera wszystkie pliki z określonym tagiem
    pub async fn get_files_with_tag(&self, tag_id: i64) -> Result<Vec<DBFile>, DbError> {
        let rows = sqlx::query(
            r#"
        SELECT f.id, f.path, f.size, f.content_hash, 
               f.last_modified, f.last_accessed, f.created
        FROM files f
        INNER JOIN file_tags ft ON f.id = ft.file_id
        WHERE ft.tag_id = ?
        ORDER BY f.path
        "#,
        )
        .bind(tag_id)
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::Sql)?;

        let files: Vec<DBFile> = rows
            .into_iter()
            .map(|row| DBFile {
                id: row.get("id"),
                path: PathBuf::from(row.get::<String, _>("path")),
                size: row.get::<i64, _>("size") as u64,
                content_hash: row.get("content_hash"),
                last_modified: i64_to_system_time(row.get("last_modified")),
                last_accessed: i64_to_system_time(row.get("last_accessed")),
                created: i64_to_system_time(row.get("created")),
            })
            .collect();

        Ok(files)
    }

    /// Pobiera wszystkie pliki z tagiem o określonej nazwie
    pub async fn get_files_with_tag_name(&self, tag_name: &str) -> Result<Vec<DBFile>, DbError> {
        if let Some(tag_id) = self.get_tag_id_by_name(tag_name).await? {
            self.get_files_with_tag(tag_id).await
        } else {
            Ok(Vec::new()) // Tag nie istnieje, zwróć pustą listę
        }
    }

    /// Pobiera wszystkie tagi wraz z liczbą plików
    pub async fn get_tags_with_file_count(&self) -> Result<Vec<(DBTag, i64)>, DbError> {
        let rows = sqlx::query(
            r#"
        SELECT t.id, t.name, COUNT(ft.file_id) as file_count
        FROM tags t
        LEFT JOIN file_tags ft ON t.id = ft.tag_id
        GROUP BY t.id, t.name
        ORDER BY t.name
        "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::Sql)?;

        let result: Vec<(DBTag, i64)> = rows
            .into_iter()
            .map(|row| {
                let tag = DBTag {
                    id: row.get("id"),
                    name: row.get("name"),
                };
                let count: i64 = row.get("file_count");
                (tag, count)
            })
            .collect();

        Ok(result)
    }

    /// Usuwa wszystkie tagi z pliku
    pub async fn remove_all_tags_from_file(&self, file_id: i64) -> Result<u64, DbError> {
        let result = sqlx::query(
            r#"
        DELETE FROM file_tags 
        WHERE file_id = ?
        "#,
        )
        .bind(file_id)
        .execute(&self.pool)
        .await
        .map_err(DbError::Sql)?;

        Ok(result.rows_affected())
    }

    /// Usuwa nieużywane tagi (bez przypisanych plików)
    pub async fn cleanup_unused_tags(&self) -> Result<u64, DbError> {
        let result = sqlx::query(
            r#"
        DELETE FROM tags 
        WHERE id NOT IN (SELECT DISTINCT tag_id FROM file_tags)
        "#,
        )
        .execute(&self.pool)
        .await
        .map_err(DbError::Sql)?;

        Ok(result.rows_affected())
    }

    /// Pobiera pulę połączeń do bazy danych (dla zaawansowanych operacji)
    pub fn pool(&self) -> &Pool<Sqlite> {
        &self.pool
    }
}
