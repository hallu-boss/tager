use crate::db::{Database, DbError};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const CONFIG_DIR: &str = ".tager";

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
    pub async fn rebuild(&self, db: &Database) -> Result<usize, DbError> {
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

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_config_dir_path() {
//         let manager = TagerManager::new("/tmp/test");
//         assert_eq!(manager.config_dir(), PathBuf::from("/tmp/test/.tager"));
//     }

//     #[test]
//     fn test_config_dir_name() {
//         assert_eq!(TagerManager::config_dir_name(), ".tager");
//     }
// }