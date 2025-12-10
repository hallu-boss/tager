use serde::Serialize;
use std::fs;
use std::time::UNIX_EPOCH;
use base64::{Engine as _, engine::general_purpose};


#[derive(Debug, Serialize)]
struct FileInfo {
    name: String,
    path: String,
    is_dir: bool,
    size: u64,
    modified: u64,
    extension: Option<String>
}

#[tauri::command]
async fn read_directory_with_metadata(path: String) -> Result<Vec<FileInfo>, String> {
    let entries = fs::read_dir(&path)
        .map_err(|e| format!("Błąd odczytu katalogu: {}", e))?;

    let mut files: Vec<FileInfo> = entries
        .filter_map(|entry_result| {
            let entry = entry_result.ok()?;
            let metadata = entry.metadata().ok()?;
            
            let name = entry.file_name().into_string().ok()?;
            let path_str = entry.path().to_str()?.to_string();
            
            let extension = entry.path()
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.to_lowercase());

            let modified = metadata.modified()
                .ok()
                .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
                .map(|duration| duration.as_secs())
                .unwrap_or(0);

            Some(FileInfo {
                name,
                path: path_str,
                is_dir: metadata.is_dir(),
                size: metadata.len(),
                modified,
                extension,
            })
        })
        .collect();

    files.sort_by(|a, b| {
        match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.cmp(&b.name),
        }
    });

    Ok(files)
}

/// Pobiera zawartość katalogu
#[tauri::command]
async fn read_directory(path: String) -> Result<Vec<String>, String> {
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

        let name = match entry.file_name().into_string() {
            Ok(name) => name,
            Err(_) => continue,
        };

        files.push(name);
    }

    files.sort();
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
    
    let base64_string = general_purpose::STANDARD.encode(&bytes);
    Ok(base64_string)
}

#[tauri::command]
async fn check_directory(path: String) -> Result<bool, String> {
    match fs::metadata(&path) {
        Ok(metadata) => Ok(metadata.is_dir()),
        Err(_) => Ok(false),
    }
}

#[derive(Debug, Serialize)]
struct DirectoryStats {
    total_files: u64,
    total_size: u64,
    directory_count: u64,
}

#[tauri::command]
async fn get_directory_stats(path: String) -> Result<DirectoryStats, String> {
    let mut total_files = 0;
    let mut total_size = 0;
    let mut dir_count = 0;
    
    fn walk_dir(path: &std::path::Path, files: &mut u64, size: &mut u64, dirs: &mut u64) -> std::io::Result<()> {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let metadata = entry.metadata()?;
            
            if metadata.is_dir() {
                *dirs += 1;
                walk_dir(&entry.path(), files, size, dirs)?;
            } else {
                *files += 1;
                *size += metadata.len();
            }
        }
        Ok(())
    }
    
    walk_dir(std::path::Path::new(&path), &mut total_files, &mut total_size, &mut dir_count)
        .map_err(|e| format!("Błąd podczas skanowania katalogu: {}", e))?;
    
    Ok(DirectoryStats {
        total_files,
        total_size,
        directory_count: dir_count,
    })
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            read_directory_with_metadata,
            read_directory,
            read_file_as_base64,
            check_directory,
            get_directory_stats
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
