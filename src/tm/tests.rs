use sha2::Digest;
use sha2::Sha256;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::thread;
use std::time::{Duration, SystemTime};
use tempfile::TempDir;

use crate::tm::TagerManager;

// Helper function do tworzenia testowych plików
fn create_test_file(path: &PathBuf, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, content).unwrap();
}

// Test podstawowej funkcjonalności

#[tokio::test]
async fn test_tager_manager_basic_lifecycle() {
    let temp_dir = TempDir::new().unwrap();

    // 1. Tworzenie managera
    let mut manager = TagerManager::new(temp_dir.path()).await.unwrap();
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

/// Helper do tworzenia tymczasowego managera
async fn create_test_manager() -> (TagerManager, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let manager = TagerManager::new(temp_dir.path()).await.unwrap();
    (manager, temp_dir)
}

/// Helper do czekania na zmianę timestampa (potrzebne dla niektórych systemów plików)
fn wait_for_timestamp_change() {
    thread::sleep(Duration::from_millis(10));
}

/// Helper do tworzenia pliku z wymuszonym zapisem na dysk
fn create_synced_file(path: &Path, content: &[u8]) {
    let mut file = fs::File::create(path).unwrap();
    file.write_all(content).unwrap();
    file.sync_all().unwrap(); // Wymusza fizyczny zapis
}

#[tokio::test]
async fn test_sync_initial_synchronization() {
    // Przygotowanie
    let (mut manager, temp_dir) = create_test_manager().await;
    manager.init().await.unwrap();

    // Utwórz kilka plików w katalogu
    let file1_path = temp_dir.path().join("test1.txt");
    let file2_path = temp_dir.path().join("test2.txt");
    let subdir_path = temp_dir.path().join("subdir");
    fs::create_dir(&subdir_path).unwrap();
    let file3_path = subdir_path.join("test3.txt");

    // Zapisz pliki z wymuszonym zapisem na dysk
    create_synced_file(&file1_path, b"Hello, World!");
    create_synced_file(&file2_path, b"Another file    ");
    create_synced_file(&file3_path, b"File in subdirectory");

    // Krótkie opóźnienie dla pewności
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    // Wykonaj synchronizację
    manager.sync().await.unwrap();

    // Sprawdź czy wszystkie pliki zostały dodane
    let all_files = manager.db().get_all_files(None, None).await.unwrap();
    assert_eq!(all_files.len(), 3, "Powinny być 3 pliki w bazie");

    // Sprawdź czy pliki mają poprawne ścieżki
    let paths: Vec<String> = all_files
        .iter()
        .map(|f| f.path.to_string_lossy().to_string())
        .collect();

    assert!(paths.contains(&"test1.txt".to_string()));
    assert!(paths.contains(&"test2.txt".to_string()));
    assert!(paths.contains(&"subdir/test3.txt".to_string()));

    // Sprawdź czy hashe są poprawne
    let test1_content = fs::read(&file1_path).unwrap();
    let mut hasher = Sha256::new();
    hasher.update(&test1_content);
    let expected_hash = format!("{:x}", hasher.finalize());

    let test1_file = all_files
        .iter()
        .find(|f| f.path.ends_with("test1.txt"))
        .unwrap();

    assert_eq!(test1_file.content_hash, expected_hash);

    // Sprawdź czy metadane są zapisane
    let metadata = fs::metadata(&file1_path).unwrap();
    assert_eq!(test1_file.size, metadata.len());
}

#[tokio::test]
async fn test_sync_file_content_update() {
    // Przygotowanie
    let (mut manager, temp_dir) = create_test_manager().await;
    manager.init().await.unwrap();

    // Utwórz plik
    let file_path = temp_dir.path().join("test.txt");
    fs::write(&file_path, "Initial content").unwrap();

    // Pierwsza synchronizacja
    manager.sync().await.unwrap();

    // Pobierz początkowy rekord
    let initial_files = manager.db().get_all_files(None, None).await.unwrap();
    let initial_file = &initial_files[0];
    let initial_hash = initial_file.content_hash.clone();
    let initial_size = initial_file.size;
    let initial_modified = initial_file.last_modified;

    // Poczekaj na zmianę timestampa
    wait_for_timestamp_change();

    // Zmień zawartość pliku
    fs::write(&file_path, "Updated content - much longer than before").unwrap();

    // Synchronizuj ponownie
    manager.sync().await.unwrap();

    // Sprawdź czy rekord został zaktualizowany
    let updated_files = manager.db().get_all_files(None, None).await.unwrap();
    let updated_file = &updated_files[0];

    // Hash powinien być inny
    assert_ne!(updated_file.content_hash, initial_hash);

    // Rozmiar powinien być inny
    assert_ne!(updated_file.size, initial_size);
    assert_eq!(updated_file.size, 41); // Długość "Updated content - much longer than before"

    // Timestamp modyfikacji powinien być nowszy
    assert!(updated_file.last_modified >= initial_modified);

    // ID powinno być takie samo (ten sam rekord zaktualizowany)
    assert_eq!(updated_file.id, initial_file.id);

    // Sprawdź czy nie ma duplikatów
    assert_eq!(updated_files.len(), 1);
}

// #[tokio::test]
// async fn test_sync_file_rename() {
//     // Przygotowanie
//     let (mut manager, temp_dir) = create_test_manager().await;
//     manager.init().await.unwrap();

//     // Utwórz plik z unikalną zawartością (dla łatwiejszego śledzenia po hash)
//     let original_path = temp_dir.path().join("original.txt");
//     let unique_content = format!("Unique content: {}", SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_nanos());
//     fs::write(&original_path, &unique_content).unwrap();

//     // Pierwsza synchronizacja
//     manager.sync().await.unwrap();

//     // Pobierz początkowy rekord
//     let initial_files = manager.db().get_all_files(None, None).await.unwrap();
//     let initial_file = &initial_files[0];
//     let file_id = initial_file.id;
//     let file_hash = initial_file.content_hash.clone();

//     // Zmień nazwę pliku
//     let renamed_path = temp_dir.path().join("renamed.txt");
//     fs::rename(&original_path, &renamed_path).unwrap();

//     // Synchronizuj ponownie
//     manager.sync().await.unwrap();

//     // Sprawdź stan bazy
//     let updated_files = manager.db().get_all_files(None, None).await.unwrap();

//     // Powinien być tylko jeden plik
//     assert_eq!(updated_files.len(), 1);

//     let updated_file = &updated_files[0];

//     // ID powinno być takie samo
//     assert_eq!(updated_file.id, file_id);

//     // Hash powinien być taki sam (nie zmienialiśmy zawartości)
//     assert_eq!(updated_file.content_hash, file_hash);

//     // Ścieżka powinna być zaktualizowana
//     assert_eq!(updated_file.path.to_string_lossy(), "renamed.txt");

//     // Sprawdź czy można znaleźć plik po nowej ścieżce
//     let file_by_new_path = manager.db().get_file_by_path(PathBuf::from("renamed.txt")).await.unwrap();
//     assert!(file_by_new_path.is_some());

//     // Sprawdź czy nie można znaleźć po starej ścieżce
//     let file_by_old_path = manager.db().get_file_by_path(PathBuf::from("original.txt")).await.unwrap();
//     assert!(file_by_old_path.is_none());
// }

// #[tokio::test]
// async fn test_sync_file_deletion() {
//     // Przygotowanie
//     let (mut manager, temp_dir) = create_test_manager().await;
//     manager.init().await.unwrap();

//     // Utwórz kilka plików
//     let file1_path = temp_dir.path().join("keep.txt");
//     let file2_path = temp_dir.path().join("delete.txt");

//     fs::write(&file1_path, "File to keep").unwrap();
//     fs::write(&file2_path, "File to delete").unwrap();

//     // Pierwsza synchronizacja
//     manager.sync().await.unwrap();

//     // Sprawdź czy oba pliki są w bazie
//     let initial_files = manager.db().get_all_files(None, None).await.unwrap();
//     assert_eq!(initial_files.len(), 2);

//     // Usuń jeden plik
//     fs::remove_file(&file2_path).unwrap();

//     // Synchronizuj ponownie
//     manager.sync().await.unwrap();

//     // Sprawdź stan bazy
//     let final_files = manager.db().get_all_files(None, None).await.unwrap();

//     // Powinien być tylko jeden plik
//     assert_eq!(final_files.len(), 1);

//     // Tylko plik "keep.txt" powinien pozostać
//     let remaining_file = &final_files[0];
//     assert_eq!(remaining_file.path.to_string_lossy(), "keep.txt");

//     // Sprawdź czy plik "delete.txt" został usunięty z bazy
//     let deleted_file = manager.db().get_file_by_path(PathBuf::from("delete.txt")).await.unwrap();
//     assert!(deleted_file.is_none());
// }

// #[tokio::test]
// async fn test_sync_mixed_operations() {
//     // Test z wieloma operacjami jednocześnie
//     let (mut manager, temp_dir) = create_test_manager().await;
//     manager.init().await.unwrap();

//     // Stan początkowy
//     let file1_path = temp_dir.path().join("file1.txt");
//     let file2_path = temp_dir.path().join("file2.txt");

//     fs::write(&file1_path, "Content 1").unwrap();
//     fs::write(&file2_path, "Content 2").unwrap();

//     manager.sync().await.unwrap();

//     // Wykonaj wiele operacji
//     // 1. Zmień zawartość file1
//     fs::write(&file1_path, "Updated content 1").unwrap();

//     // 2. Zmień nazwę file2
//     let file2_new_path = temp_dir.path().join("file2_renamed.txt");
//     fs::rename(&file2_path, &file2_new_path).unwrap();

//     // 3. Utwórz nowy plik
//     let file3_path = temp_dir.path().join("file3_new.txt");
//     fs::write(&file3_path, "Content 3").unwrap();

//     // 4. Utwórz i usuń plik (nigdy nie powinien trafić do bazy)
//     let temp_file_path = temp_dir.path().join("temp.txt");
//     fs::write(&temp_file_path, "Temp content").unwrap();
//     fs::remove_file(&temp_file_path).unwrap();

//     // Synchronizuj
//     manager.sync().await.unwrap();

//     // Sprawdź końcowy stan
//     let final_files = manager.db().get_all_files(None, None).await.unwrap();

//     // Powinny być 3 pliki:
//     // - file1.txt (zaktualizowana zawartość)
//     // - file2_renamed.txt (przeniesiony)
//     // - file3_new.txt (nowy)
//     assert_eq!(final_files.len(), 3);

//     let paths: Vec<String> = final_files
//         .iter()
//         .map(|f| f.path.to_string_lossy().to_string())
//         .collect();

//     assert!(paths.contains(&"file1.txt".to_string()));
//     assert!(paths.contains(&"file2_renamed.txt".to_string()));
//     assert!(paths.contains(&"file3_new.txt".to_string()));

//     // Sprawdź czy temp.txt nie ma w bazie
//     assert!(!paths.contains(&"temp.txt".to_string()));
// }

// #[tokio::test]
// async fn test_sync_file_move_between_directories() {
//     // Test przenoszenia pliku między katalogami
//     let (mut manager, temp_dir) = create_test_manager().await;
//     manager.init().await.unwrap();

//     // Utwórz strukturę katalogów
//     let dir1 = temp_dir.path().join("dir1");
//     let dir2 = temp_dir.path().join("dir2");
//     fs::create_dir(&dir1).unwrap();
//     fs::create_dir(&dir2).unwrap();

//     // Plik w dir1
//     let file_path = dir1.join("file.txt");
//     let unique_content = format!("Unique: {}", SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_nanos());
//     fs::write(&file_path, &unique_content).unwrap();

//     // Pierwsza synchronizacja
//     manager.sync().await.unwrap();

//     // Przenieś plik do dir2
//     let new_path = dir2.join("file.txt");
//     fs::rename(&file_path, &new_path).unwrap();

//     // Synchronizuj
//     manager.sync().await.unwrap();

//     // Sprawdź
//     let files = manager.db().get_all_files(None, None).await.unwrap();
//     assert_eq!(files.len(), 1);

//     let file = &files[0];
//     assert_eq!(file.path.to_string_lossy(), "dir2/file.txt");
// }

// #[tokio::test]
// async fn test_sync_duplicate_files_different_paths() {
//     // Test: dwa różne pliki z tą samą zawartością
//     let (mut manager, temp_dir) = create_test_manager().await;
//     manager.init().await.unwrap();

//     // Utwórz dwa pliki z tą samą zawartością
//     let file1_path = temp_dir.path().join("file1.txt");
//     let file2_path = temp_dir.path().join("file2.txt");

//     let content = "Same content";
//     fs::write(&file1_path, content).unwrap();
//     fs::write(&file2_path, content).unwrap();

//     manager.sync().await.unwrap();

//     let files = manager.db().get_all_files(None, None).await.unwrap();
//     assert_eq!(files.len(), 2);

//     // Oba pliki powinny mieć ten sam hash
//     let hash1 = files[0].content_hash.clone();
//     let hash2 = files[1].content_hash.clone();
//     assert_eq!(hash1, hash2);

//     // Ale różne ID i ścieżki
//     assert_ne!(files[0].id, files[1].id);
//     assert_ne!(files[0].path, files[1].path);
// }

// #[tokio::test]
// async fn test_sync_large_file_hash() {
//     // Test hashowania dużego pliku
//     let (mut manager, temp_dir) = create_test_manager().await;
//     manager.init().await.unwrap();

//     // Utwórz duży plik (1MB)
//     let large_file_path = temp_dir.path().join("large.bin");
//     let mut file = File::create(&large_file_path).unwrap();

//     // Zapisz 1MB danych
//     let data = vec![0xAAu8; 1024 * 1024]; // 1MB
//     file.write_all(&data).unwrap();
//     drop(file);

//     manager.sync().await.unwrap();

//     let files = manager.db().get_all_files(None, None).await.unwrap();
//     assert_eq!(files.len(), 1);

//     let file_record = &files[0];
//     assert_eq!(file_record.size, 1024 * 1024);

//     // Sprawdź czy hash został obliczony poprawnie
//     // (Możesz dodać sprawdzenie konkretnego hash jeśli znasz oczekiwany wynik)
//     assert_eq!(file_record.content_hash.len(), 64); // SHA-256 w hex
// }

// #[tokio::test]
// async fn test_sync_without_initialization() {
//     // Test: próba synchronizacji bez inicjalizacji
//     let (manager, _temp_dir) = create_test_manager().await;
//     // Nie wywołujemy init()

//     let result = manager.sync().await;
//     assert!(result.is_err());
//     assert!(result.unwrap_err().contains("nie jest zainicjalizowany"));
// }

// #[tokio::test]
// async fn test_sync_empty_directory() {
//     // Test synchronizacji pustego katalogu
//     let (mut manager, _temp_dir) = create_test_manager().await;
//     manager.init().await.unwrap();

//     manager.sync().await.unwrap();

//     let files = manager.db().get_all_files(None, None).await.unwrap();
//     assert_eq!(files.len(), 0);
// }

// #[tokio::test]
// async fn test_sync_ignores_tager_directory() {
//     // Test: katalog .tager powinien być ignorowany
//     let (mut manager, temp_dir) = create_test_manager().await;
//     manager.init().await.unwrap();

//     // Utwórz plik w katalogu .tager
//     let tager_file = temp_dir.path().join(".tager").join("config.txt");
//     fs::write(&tager_file, "Should be ignored").unwrap();

//     // Utwórz normalny plik
//     let normal_file = temp_dir.path().join("normal.txt");
//     fs::write(&normal_file, "Should be tracked").unwrap();

//     manager.sync().await.unwrap();

//     let files = manager.db().get_all_files(None, None).await.unwrap();

//     // Powinien być tylko normal.txt
//     assert_eq!(files.len(), 1);
//     assert_eq!(files[0].path.to_string_lossy(), "normal.txt");
// }
