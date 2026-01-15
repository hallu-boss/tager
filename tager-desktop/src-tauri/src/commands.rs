use tauri::{AppHandle, Manager, State};
use tokio::sync::Mutex;
use log;

use app_lib::tm::{FileEntry, TagEntry, TagerManager};


#[derive(serde::Serialize)]
pub struct FileEntryResponse {
    pub id: i64,
    pub abs_path: String,
    pub rel_path: String,
    pub file_name: String,
    pub tags: Vec<TagEntryResponse>,
    pub last_modified: String,
    pub created: String,
}

#[derive(serde::Serialize)]
pub struct TagEntryResponse {
    pub id: i64,
    pub name: String,
}

impl From<FileEntry> for FileEntryResponse {
    fn from(entry: FileEntry) -> Self {
        Self {
            id: entry.id,
            abs_path: entry.abs_path,
            rel_path: entry.rel_path,
            file_name: entry.file_name,
            tags: entry.tags.into_iter().map(|t| t.into()).collect(),
            last_modified: entry.last_modified,
            created: entry.created,
        }
    }
}

impl From<TagEntry> for TagEntryResponse {
    fn from(entry: TagEntry) -> Self {
        Self {
            id: entry.id,
            name: entry.name,
        }
    }
}

pub struct AppState {
    pub manager: Mutex<Option<TagerManager>>,
}

#[tauri::command]
pub async fn init_tager_manager(
    app_handle: AppHandle,
    path: String,
) -> Result<Vec<FileEntryResponse>, String> {
    let state: State<'_, AppState> = app_handle.state();
    
    log::info!("Inicjalizowanie TagerManager dla ścieżki: {}", path);
    
    // Utwórz i zainicjalizuj managera
    let mut manager = TagerManager::new(&path)
        .await
        .map_err(|e| format!("Nie udało się utworzyć manager: {}", e))?;
    
    manager.init()
        .await
        .map_err(|e| format!("Nie udało się zainicjalizować: {}", e))?;
    
    log::info!("Synchronizowanie plików...");
    manager.sync()
        .await
        .map_err(|e| format!("Nie udało się zsynchronizować: {}", e))?;
    
    log::info!("Pobieranie listy plików...");
    let files = manager.get_files(None, None)
        .await
        .map_err(|e| format!("Nie udało się pobrać plików: {}", e))?;
    
    // Zapisz managera w stanie
    let mut state_lock = state.manager.lock().await;
    *state_lock = Some(manager);
    
    log::info!("Zainicjalizowano TagerManager. Znaleziono {} plików.", files.len());
    
    // Konwertuj na odpowiedź
    let response: Vec<FileEntryResponse> = files.into_iter()
        .map(|f| f.into())
        .collect();
    
    Ok(response)
}

#[tauri::command]
pub async fn get_filtered_files(
    app_handle: AppHandle,
    name_filter: Option<String>,
    tag_filters: Option<Vec<String>>,
) -> Result<Vec<FileEntryResponse>, String> {
    let state: State<'_, AppState> = app_handle.state();
    let state_lock = state.manager.lock().await;
    
    match &*state_lock {
        Some(manager) => {
            let files = manager.get_files(name_filter, tag_filters)
                .await
                .map_err(|e| format!("Nie udało się pobrać plików: {}", e))?;
            
            let response: Vec<FileEntryResponse> = files.into_iter()
                .map(|f| f.into())
                .collect();
            
            Ok(response)
        }
        None => Err("Manager nie jest zainicjalizowany. Wywołaj init_tager_manager najpierw.".to_string()),
    }
}

#[tauri::command]
pub async fn get_files_without_tags(
    app_handle: AppHandle,
) -> Result<Vec<FileEntryResponse>, String> {
    let state: State<'_, AppState> = app_handle.state();
    let state_lock = state.manager.lock().await;
    
    match &*state_lock {
        Some(manager) => {
            let files = manager.get_files_without_tags()
                .await
                .map_err(|e| format!("Nie udało się pobrać plików bez tagów: {}", e))?;
            
            let response: Vec<FileEntryResponse> = files.into_iter()
                .map(|f| f.into())
                .collect();
            
            Ok(response)
        }
        None => Err("Manager nie jest zainicjalizowany. Wywołaj init_tager_manager najpierw.".to_string()),
    }
}

#[tauri::command]
pub async fn assign_tag_to_file(
    app_handle: AppHandle,
    file_path: String,
    tag_name: String,
) -> Result<(), String> {
    let state: State<'_, AppState> = app_handle.state();
    let state_lock = state.manager.lock().await;
    
    match &*state_lock {
        Some(manager) => {
            let path = std::path::Path::new(&file_path);
            manager.assign_tag_to_file(path, &tag_name)
                .await
                .map_err(|e| format!("Nie udało się przypisać tagu do pliku: {}", e))?;
            
            log::info!("Przypisano tag '{}' do pliku: {}", tag_name, file_path);
            Ok(())
        }
        None => Err("Manager nie jest zainicjalizowany. Wywołaj init_tager_manager najpierw.".to_string()),
    }
}

#[tauri::command]
pub async fn get_all_tags(
    app_handle: AppHandle,
) -> Result<Vec<TagEntryResponse>, String> {
    let state: State<'_, AppState> = app_handle.state();
    let state_lock = state.manager.lock().await;
    
    match &*state_lock {
        Some(manager) => {
            let tags = manager.get_all_tags()
                .await
                .map_err(|e| format!("Nie udało się pobrać tagów: {}", e))?;
            
            let response: Vec<TagEntryResponse> = tags.into_iter()
                .map(|t| t.into())
                .collect();
            
            Ok(response)
        }
        None => Err("Manager nie jest zainicjalizowany. Wywołaj init_tager_manager najpierw.".to_string()),
    }
}

#[tauri::command]
pub async fn sync_and_get_files(app_handle: AppHandle) -> Result<Vec<FileEntryResponse>, String> {
    let state: State<'_, AppState> = app_handle.state();
    let state_lock = state.manager.lock().await;
    
    match &*state_lock {
        Some(manager) => {
            log::info!("Synchronizowanie...");
            manager.sync()
                .await
                .map_err(|e| format!("Nie udało się zsynchronizować: {}", e))?;
            
            let files = manager.get_files(None, None)
                .await
                .map_err(|e| format!("Nie udało się pobrać plików: {}", e))?;
            
            let response: Vec<FileEntryResponse> = files.into_iter()
                .map(|f| f.into())
                .collect();
            
            Ok(response)
        }
        None => Err("Manager nie jest zainicjalizowany.".to_string()),
    }
}

// Dodatkowe pomocnicze komendy
#[tauri::command]
pub async fn get_manager_status(app_handle: AppHandle) -> Result<ManagerStatus, String> {
    let state: State<'_, AppState> = app_handle.state();
    let state_lock = state.manager.lock().await;
    
    match &*state_lock {
        Some(manager) => {
            let initialized = manager.is_initialized();
            let root = manager.root().to_string_lossy().to_string();
            let total_files = manager.db().count_files().await
                .unwrap_or(0);
            let total_tags = manager.db().count_tags().await
                .unwrap_or(0);
            
            Ok(ManagerStatus {
                initialized,
                root_path: root,
                total_files,
                total_tags,
            })
        }
        None => Ok(ManagerStatus {
            initialized: false,
            root_path: String::new(),
            total_files: 0,
            total_tags: 0,
        }),
    }
}

#[derive(serde::Serialize)]
pub struct ManagerStatus {
    pub initialized: bool,
    pub root_path: String,
    pub total_files: i64,
    pub total_tags: i64,
}

#[tauri::command]
pub async fn disconnect_manager(app_handle: AppHandle) -> Result<(), String> {
    let state: State<'_, AppState> = app_handle.state();
    let mut state_lock = state.manager.lock().await;
    
    if let Some(ref mut manager) = *state_lock {
        manager.disconnect();
    }
    
    *state_lock = None;
    log::info!("Manager został rozłączony.");
    
    Ok(())
}