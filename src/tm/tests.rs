use std::path::PathBuf;
use std::fs::{create_dir_all, write};
use tempfile::TempDir;

use crate::tm::TagerManager;

// Helper function do tworzenia testowych plików
fn create_test_file(path: &PathBuf, content: &str) {
    if let Some(parent) = path.parent() {
        create_dir_all(parent).unwrap();
    }
    write(path, content).unwrap();
}

// Test podstawowej funkcjonalności

#[tokio::test]
async fn test_tager_manager_basic_lifecycle() {
    let temp_dir = TempDir::new().unwrap();

    // 1. Tworzenie managera
    let mut manager = TagerManager::new(temp_dir.path()).await;
    assert!(!manager.is_initialized());
    assert!(!manager.tager_dir().exists());

    // 2. Inicjalizacja
    manager.init().await.unwrap();
    assert!(manager.is_initialized());
    assert!(manager.tager_dir().exists());
    assert!(manager.db_path().exists());

    // 3. Weryfikacja bazy danych
    let db = manager.db();
    let file_count = db.count_files().await.unwrap();
    let tag_count = db.count_tags().await.unwrap();
    assert_eq!(file_count, 0);
    assert_eq!(tag_count, 0);

    // 4. Zniszczenie systemu
    manager.disconnect();
    assert!(!manager.is_initialized());
}
