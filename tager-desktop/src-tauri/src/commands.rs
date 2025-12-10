use crate::db::{Database, FilesOrderBy};
use crate::manager::{TagerManager, FileMetadata};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

// Stan aplikacji
pub struct AppState {
    pub manager: Mutex<Option<TagerManager>>,
}

#[tauri::command]
pub async fn initialize_directory(root_dir: String) -> Result<usize, String> {
    let manager = TagerManager::new(&root_dir)
        .map_err(|e| format!("Nie udało się utworzyć managera: {}", e))?;
    
    manager.init_database()
        .await
        .map_err(|e| format!("Nie udało się zainicjalizować bazy danych: {}", e))?;
    
    let count = manager.rebuild()
        .await
        .map_err(|e| format!("Nie udało się przeskanować katalogu: {}", e))?;
    
    // Zapisz instancję managera do globalnego stanu
    let mut state = crate::APP_STATE.lock().await;
    *state = Some(manager);
    
    Ok(count)
}

#[tauri::command]
pub async fn get_files() -> Result<Vec<FileMetadata>, String> {
    let state = crate::APP_STATE.lock().await;
    let manager = state.as_ref()
        .ok_or("Katalog nie został zainicjalizowany. Najpierw wybierz katalog.")?;
    
    manager.get_files_with_metadata()
        .await
        .map_err(|e| format!("Nie udało się pobrać plików: {}", e))
}

#[tauri::command]
pub async fn add_tag(file_path: String, tag: String) -> Result<(), String> {
    let state = crate::APP_STATE.lock().await;
    let manager = state.as_ref()
        .ok_or("Katalog nie został zainicjalizowany")?;
    
    let db = TagerManager::get_database()
        .ok_or("Baza danych nie została zainicjalizowana")?;
    
    // Pobierz ścieżkę względną
    let relative_path = PathBuf::from(&file_path)
        .strip_prefix(manager.root_dir())
        .map_err(|e| format!("Nie udało się uzyskać ścieżki względnej: {}", e))?
        .to_string_lossy()
        .to_string();
    
    let file_id = db.get_file_id_by_path(&relative_path)
        .await
        .map_err(|e| format!("Nie udało się znaleźć pliku: {}", e))?;
    
    db.assign_tag_to_file_by_id(file_id, &tag)
        .await
        .map_err(|e| format!("Nie udało się dodać tagu: {}", e))
}

#[tauri::command]
pub async fn remove_tag(file_path: String, tag: String) -> Result<(), String> {
    let state = crate::APP_STATE.lock().await;
    let manager = state.as_ref()
        .ok_or("Katalog nie został zainicjalizowany")?;
    
    let db = TagerManager::get_database()
        .ok_or("Baza danych nie została zainicjalizowana")?;
    
    // Pobierz ścieżkę względną
    let relative_path = PathBuf::from(&file_path)
        .strip_prefix(manager.root_dir())
        .map_err(|e| format!("Nie udało się uzyskać ścieżki względnej: {}", e))?
        .to_string_lossy()
        .to_string();
    
    let file_id = db.get_file_id_by_path(&relative_path)
        .await
        .map_err(|e| format!("Nie udało się znaleźć pliku: {}", e))?;
    
    db.remove_tag_from_file_by_id(file_id, &tag)
        .await
        .map_err(|e| format!("Nie udało się usunąć tagu: {}", e))
}

#[tauri::command]
pub async fn get_tags_for_file(file_path: String) -> Result<Vec<String>, String> {
    let state = crate::APP_STATE.lock().await;
    let manager = state.as_ref()
        .ok_or("Katalog nie został zainicjalizowany")?;
    
    let db = TagerManager::get_database()
        .ok_or("Baza danych nie została zainicjalizowana")?;
    
    // Pobierz ścieżkę względną
    let relative_path = PathBuf::from(&file_path)
        .strip_prefix(manager.root_dir())
        .map_err(|e| format!("Nie udało się uzyskać ścieżki względnej: {}", e))?
        .to_string_lossy()
        .to_string();
    
    db.get_tags_for_file(&relative_path)
        .await
        .map_err(|e| format!("Nie udało się pobrać tagów: {}", e))
}

#[tauri::command]
pub async fn search_files(query: String) -> Result<Vec<FileMetadata>, String> {
    let state = crate::APP_STATE.lock().await;
    let manager = state.as_ref()
        .ok_or("Katalog nie został zainicjalizowany")?;
    
    let db = TagerManager::get_database()
        .ok_or("Baza danych nie została zainicjalizowana")?;
    
    // Pobierz wszystkie pliki
    let all_files = manager.get_files_with_metadata()
        .await
        .map_err(|e| format!("Nie udało się pobrać plików: {}", e))?;
    
    // Filtruj pliki na podstawie zapytania
    let filtered_files = all_files.into_iter()
        .filter(|file| {
            file.name.to_lowercase().contains(&query.to_lowercase()) ||
            file.tags.iter().any(|tag| tag.to_lowercase().contains(&query.to_lowercase()))
        })
        .collect();
    
    Ok(filtered_files)
}

#[tauri::command]
pub async fn get_all_tags() -> Result<Vec<String>, String> {
    let db = TagerManager::get_database()
        .ok_or("Baza danych nie została zainicjalizowana")?;
    
    let tags = sqlx::query("SELECT name FROM tags ORDER BY name")
        .fetch_all(db.pool())
        .await
        .map_err(|e| format!("Nie udało się pobrać tagów: {}", e))?
        .into_iter()
        .map(|row| row.get::<String, _>("name"))
        .collect();
    
    Ok(tags)
}

#[tauri::command]
pub async fn get_untagged_files() -> Result<Vec<FileMetadata>, String> {
    let state = crate::APP_STATE.lock().await;
    let manager = state.as_ref()
        .ok_or("Katalog nie został zainicjalizowany")?;
    
    let db = TagerManager::get_database()
        .ok_or("Baza danych nie została zainicjalizowana")?;
    
    let untagged = db.get_untagged_files(Some(FilesOrderBy::Path))
        .await
        .map_err(|e| format!("Nie udało się pobrać nieotagowanych plików: {}", e))?;
    
    let mut result = Vec::new();
    
    for (_, relative_path) in untagged {
        let full_path = manager.root_dir().join(&relative_path);
        
        // Pobierz metadane pliku
        let metadata = match std::fs::metadata(&full_path) {
            Ok(md) => md,
            Err(_) => continue,
        };
        
        let file_metadata = FileMetadata {
            id: "".to_string(),
            name: full_path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(&relative_path)
                .to_string(),
            path: full_path.to_string_lossy().to_string(),
            tags: vec![],
            size: metadata.len(),
            modified: metadata.modified()
                .map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs())
                .unwrap_or(0),
            extension: full_path.extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.to_lowercase()),
            file_type: "other".to_string(),
            is_dir: false,
        };
        
        result.push(file_metadata);
    }
    
    Ok(result)
}

#[tauri::command]
pub async fn refresh_directory() -> Result<usize, String> {
    let state = crate::APP_STATE.lock().await;
    let manager = state.as_ref()
        .ok_or("Katalog nie został zainicjalizowany")?;
    
    manager.rebuild()
        .await
        .map_err(|e| format!("Nie udało się odświeżyć katalogu: {}", e))
}

#[derive(Debug, serde::Serialize)]
struct FileInfo {
    name: String,
    path: String,
    is_dir: bool,
    size: u64,
    modified: u64,
    extension: Option<String>,
}

#[tauri::command]
async fn read_directory_with_metadata(path: String) -> Result<Vec<FileInfo>, String> {
    use std::fs;
    use std::time::UNIX_EPOCH;

    let entries = match fs::read_dir(&path) {
        Ok(entries) => entries,
        Err(e) => return Err(format!("Błąd odczytu katalogu: {}", e)),
    };

    let mut files = Vec::new();

    for entry_result in entries {
        let entry = match entry_result {
            Ok(entry) => entry,
            Err(_) => continue,
        };

        let metadata = match entry.metadata() {
            Ok(meta) => meta,
            Err(_) => continue,
        };

        let name = match entry.file_name().into_string() {
            Ok(name) => name,
            Err(_) => continue,
        };

        let path_str = match entry.path().to_str() {
            Some(path) => path.to_string(),
            None => continue,
        };

        let extension = entry.path()
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase());

        let modified = match metadata.modified() {
            Ok(time) => match time.duration_since(UNIX_EPOCH) {
                Ok(duration) => duration.as_secs(),
                Err(_) => 0,
            },
            Err(_) => 0,
        };

        let file_info = FileInfo {
            name,
            path: path_str,
            is_dir: metadata.is_dir(),
            size: metadata.len(),
            modified,
            extension,
        };

        files.push(file_info);
    }

    // Sortowanie: katalogi pierwsze, potem pliki alfabetycznie
    files.sort_by(|a, b| {
        if a.is_dir && !b.is_dir {
            std::cmp::Ordering::Less
        } else if !a.is_dir && b.is_dir {
            std::cmp::Ordering::Greater
        } else {
            a.name.cmp(&b.name)
        }
    });

    Ok(files)
}

#[tauri::command]
async fn read_file_as_base64(path: String) -> Result<String, String> {
    let max_size = 5 * 1024 * 1024; // 5MB limit
    
    let metadata = fs::metadata(&path)
        .map_err(|e| format!("Nie udało się odczytać metadanych: {}", e))?;
    
    if metadata.len() > max_size {
        return Err("Plik jest zbyt duży do wyświetlenia".to_string());
    }
    
    let bytes = fs::read(&path)
        .map_err(|e| format!("Nie udało się odczytać pliku: {}", e))?;
    
    let base64_string = base64::encode(&bytes);
    Ok(base64_string)
}