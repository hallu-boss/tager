use crate::db::{Database, DbError};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use once_cell::sync::OnceCell;

const CONFIG_DIR: &str = ".tager";
const DB_FILE: &str = "tager.db";

/// Globalna instancja bazy danych
static DATABASE: OnceCell<Database> = OnceCell::new();

/// Manages the tager configuration and operations.
pub struct TagerManager {
    root_dir: PathBuf,
}

impl TagerManager {
    /// Create a new TagerManager for the given root directory.
    pub fn new<P: AsRef<Path>>(root_dir: P) -> std::io::Result<Self> {
        let manager = Self {
            root_dir: root_dir.as_ref().to_path_buf(),
        };
        manager.ensure_config_dir()?;
        Ok(manager)
    }

    /// Initialize database for the current root directory
    pub async fn init_database(&self) -> Result<(), DbError> {
        let db_path = self.config_dir().join(DB_FILE);
        let db = Database::new_file(&db_path).await?;
        DATABASE.set(db).map_err(|_| DbError::Sql(sqlx::Error::Configuration("Database already initialized".into())))?;
        Ok(())
    }

    /// Get database instance
    pub fn get_database() -> Option<&'static Database> {
        DATABASE.get()
    }

    /// Get the configuration directory path.
    pub fn config_dir(&self) -> PathBuf {
        self.root_dir.join(CONFIG_DIR)
    }

    /// Ensure the configuration directory exists.
    fn ensure_config_dir(&self) -> std::io::Result<()> {
        let config_path = self.config_dir();
        if !config_path.exists() {
            std::fs::create_dir_all(&config_path)?;
        }
        Ok(())
    }

    /// Scan the filesystem and add all files to the database.
    /// The configuration directory is automatically excluded.
    /// Returns the number of new files added.
    pub async fn rebuild(&self) -> Result<usize, DbError> {
        let db = Self::get_database().ok_or(DbError::Sql(sqlx::Error::Configuration("Database not initialized".into())))?;
        
        let mut added_count = 0;

        for entry in WalkDir::new(&self.root_dir)
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| self.should_include_entry(e))
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

            let rel_path_str = rel_path.to_string_lossy();

            if db.add_file_if_not_exists(&rel_path_str).await? {
                added_count += 1;
            }
        }

        Ok(added_count)
    }

    /// Get all files in the root directory (without adding to database).
    pub fn get_all_files(&self) -> Vec<PathBuf> {
        WalkDir::new(&self.root_dir)
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| self.should_include_entry(e))
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .map(|e| e.path().to_path_buf())
            .collect()
    }

    /// Get relative paths of all files.
    pub fn get_all_relative_paths(&self) -> Vec<PathBuf> {
        WalkDir::new(&self.root_dir)
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| self.should_include_entry(e))
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter_map(|e| {
                e.path()
                    .strip_prefix(&self.root_dir)
                    .ok()
                    .map(|p| p.to_path_buf())
            })
            .collect()
    }

    /// Get all files with metadata for frontend
    pub async fn get_files_with_metadata(&self) -> Result<Vec<FileMetadata>, DbError> {
        let db = Self::get_database().ok_or(DbError::Sql(sqlx::Error::Configuration("Database not initialized".into())))?;
        
        // Pobierz wszystkie pliki z bazy danych
        let files = sqlx::query("SELECT id, path FROM files")
            .fetch_all(db.pool())
            .await?;
        
        let mut result = Vec::new();
        
        for row in files {
            let id: i64 = row.get("id");
            let path: String = row.get("path");
            let full_path = self.root_dir.join(&path);
            
            // Pobierz metadane pliku
            let metadata = match std::fs::metadata(&full_path) {
                Ok(md) => md,
                Err(_) => continue,
            };
            
            // Pobierz tagi dla tego pliku
            let tags = db.get_tags_for_file(&path).await?;
            
            // Określ typ pliku na podstawie rozszerzenia
            let extension = full_path.extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.to_lowercase());
            
            let file_type = match extension.as_deref() {
                Some("jpg") | Some("jpeg") | Some("png") | Some("gif") | Some("bmp") => "image",
                Some("mp4") | Some("avi") | Some("mov") | Some("mkv") => "video",
                Some("pdf") | Some("docx") | Some("doc") | Some("txt") => "document",
                _ => "other",
            };
            
            let file_metadata = FileMetadata {
                id: id.to_string(),
                name: full_path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(&path)
                    .to_string(),
                path: full_path.to_string_lossy().to_string(),
                tags,
                size: metadata.len(),
                modified: metadata.modified()
                    .map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs())
                    .unwrap_or(0),
                extension,
                file_type: file_type.to_string(),
                is_dir: false,
            };
            
            result.push(file_metadata);
        }
        
        Ok(result)
    }

    /// Check if a directory entry should be included in the scan.
    /// Automatically excludes the configuration directory.
    fn should_include_entry(&self, entry: &walkdir::DirEntry) -> bool {
        if entry.file_type().is_dir() {
            if let Some(name) = entry.path().file_name() {
                let name_str = name.to_string_lossy();
                // Exclude config directory
                if name_str == CONFIG_DIR {
                    return false;
                }
            }
        }
        true
    }

    /// Get the root directory.
    pub fn root_dir(&self) -> &Path {
        &self.root_dir
    }

    /// Get the configuration directory name.
    pub fn config_dir_name() -> &'static str {
        CONFIG_DIR
    }
}

/// Metadata dla pliku do wysłania do frontendu
#[derive(Debug, serde::Serialize)]
pub struct FileMetadata {
    pub id: String,
    pub name: String,
    pub path: String,
    pub tags: Vec<String>,
    pub size: u64,
    pub modified: u64,
    pub extension: Option<String>,
    pub file_type: String,
    pub is_dir: bool,
}