use std::{
    collections::HashMap,
    io::Read, // Dodane
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
};

use sha2::{Digest, Sha256};
use walkdir::WalkDir;

use crate::tm::db::{DBFile, Database, DbError};
use std::fs;

/// TagerManager - główna struktura zarządzająca systemem tagowania
pub struct TagerManager {
    /// Ścieżka do katalogu głównego
    root: PathBuf,
    /// Ścieżka do katalogu .tager w root
    tager_dir: PathBuf,
    /// Ścieżka do pliku bazy danych
    db_path: PathBuf,
    /// Instancja bazy danych
    db: Database,
    /// Flaga inicjalizacji
    is_initialized: bool,
}

impl TagerManager {
    /// Tworzy nowy TagerManager dla danego katalogu
    /// Nie inicjalizuje bazy danych - do tego służy metoda `init()`
    pub async fn new<P: AsRef<Path>>(root: P) -> Result<Self, String> {
        let root = root.as_ref().to_path_buf();

        if !root.exists() {
            return Err(format!("Katalog '{}' nie istnieje", root.display()));
        }

        if !root.is_dir() {
            return Err(format!("'{}' nie jest katalogiem", root.display()));
        }

        let tager_dir = root.join(".tager");
        let db_path = tager_dir.join("db.sqlite");

        // Tworzymy tymczasową bazę w pamięci jako placeholder
        let db = Database::new_in_memory()
            .await
            .map_err(|e| format!("Nie udało się utworzyć tymczasowej bazy danych: {}", e))?;

        Ok(Self {
            root,
            tager_dir,
            db_path,
            db,
            is_initialized: false,
        })
    }

    /// Inicjalizuje system tagowania w katalogu
    /// - Tworzy katalog .tager jeśli nie istnieje
    /// - Tworzy bazę danych w pliku
    /// - Inicjalizuje schemat bazy danych
    pub async fn init(&mut self) -> Result<(), DbError> {
        // Utwórz katalog .tager jeśli nie istnieje
        if !self.tager_dir.exists() {
            std::fs::create_dir_all(&self.tager_dir).map_err(DbError::Io)?;
            println!("Utworzono katalog: {}", self.tager_dir.display());
        }

        // Utwórz bazę danych w pliku
        self.db = Database::new_from_file(&self.db_path).await?;

        self.is_initialized = true;
        Ok(())
    }

    /// Synchronizuje rekordy bazy danych z zawartością katalogu root
    pub async fn sync(&self) -> Result<(), String> {
        if !self.is_initialized {
            return Err(
                "TagerManager nie jest zainicjalizowany. Wywołaj init() przed sync()".to_string(),
            );
        }

        let fs_map = self.get_root_snapshot()?;
        let db_files = self
            .db
            .get_all_files(None, None)
            .await
            .map_err(|e| e.to_string())?;

        let mut db_map: HashMap<String, DBFile> = db_files
            .into_iter()
            .map(|f| (f.path.to_string_lossy().to_string(), f))
            .collect();

        // Tworzymy mapę hash -> lista ścieżek (może być wiele plików o tym samym hash)
        let mut hash_to_paths: HashMap<String, Vec<String>> = HashMap::new();
        for (path, db_file) in &db_map {
            hash_to_paths
                .entry(db_file.content_hash.clone())
                .or_insert_with(Vec::new)
                .push(path.clone());
        }

        // Dla każdego pliku w systemie plików
        for (path, fs_file) in &fs_map {
            match db_map.get(path) {
                Some(db_file) => {
                    // Aktualizacja jeśli się zmienił
                    if db_file.content_hash != fs_file.content_hash
                    {
                        self.db
                            .update_file(
                                db_file.id,
                                None,
                                Some(fs_file.size),
                                Some(fs_file.content_hash.clone()),
                                Some(fs_file.last_modified),
                                Some(fs_file.last_accessed),
                                Some(fs_file.created),
                            )
                            .await
                            .map_err(|e| e.to_string())?;
                    }
                    // Oznacz jako obsłużony - usuwamy z db_map
                    db_map.remove(path);
                }
                None => {
                    // Sprawdź czy istnieje plik o tym samym hashu w db_map
                    let mut potential_move = None;
                    if let Some(paths) = hash_to_paths.get(&fs_file.content_hash) {
                        // Sprawdź każdą ścieżkę z tym samym hashem
                        for old_path in paths {
                            if let Some(old_file) = db_map.get(old_path) {
                                // Potencjalne przeniesienie - upewnijmy się że to ten sam plik
                                if old_file.size == fs_file.size
                                    && old_file.content_hash == fs_file.content_hash
                                {
                                    // To prawdopodobnie przeniesiony plik
                                    potential_move = Some(old_file.id);
                                    // Usuwamy starą ścieżkę z db_map aby nie została usunięta
                                    db_map.remove(old_path);
                                    break;
                                }
                            }
                        }
                    }

                    if let Some(file_id) = potential_move {
                        // Plik przeniesiony - aktualizuj ścieżkę
                        self.db
                            .update_file(
                                file_id,
                                Some(fs_file.path.clone()),
                                None,
                                None,
                                None,
                                None,
                                None,
                            )
                            .await
                            .map_err(|e| e.to_string())?;
                    } else {
                        // Nowy plik
                        self.db
                            .create_file(
                                fs_file.path.clone(),
                                fs_file.size,
                                fs_file.content_hash.clone(),
                                fs_file.last_modified,
                                fs_file.last_accessed,
                                fs_file.created,
                            )
                            .await
                            .map_err(|e| e.to_string())?;
                    }
                }
            }
        }

        // Usuń pliki które zniknęły z systemu plików (nie zostały ani znalezione, ani przeniesione)
        for (_, db_file) in db_map {
            self.db
                .delete_file(db_file.id)
                .await
                .map_err(|e| e.to_string())?;
        }

        Ok(())
    }

    fn get_root_snapshot(&self) -> Result<HashMap<String, DBFile>, String> {
        let mut fs_map = HashMap::new();

        for entry in WalkDir::new(&self.root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter(|e| !e.path().starts_with(&self.tager_dir))
        // Pomijamy katalog .tager
        {
            let abs_path = entry.path();

            // Konwersja ścieżki względnej
            let rel_path = abs_path
                .strip_prefix(&self.root)
                .map_err(|e| format!("Błąd konwersji ścieżki {}: {}", abs_path.display(), e))?;

            let rel_path_str = rel_path.to_string_lossy().to_string();

            // Pobierz metadane
            let metadata = fs::metadata(abs_path)
                .map_err(|e| format!("Błąd odczytu metadanych {}: {}", abs_path.display(), e))?;

            // Oblicz hash
            let hash = Self::hash_file(abs_path)
                .map_err(|e| format!("Błąd hashowania {}: {}", abs_path.display(), e))?;

            // Czasy - bezpieczne pobieranie
            let last_modified = metadata
                .modified()
                .map_err(|e| format!("Błąd odczytu czasu modyfikacji: {}", e))?;
            let last_accessed = metadata
                .accessed()
                .map_err(|e| format!("Błąd odczytu czasu dostępu: {}", e))?;
            let created = metadata
                .created()
                .map_err(|e| format!("Błąd odczytu czasu utworzenia: {}", e))?;

            fs_map.insert(
                rel_path_str.clone(),
                DBFile {
                    id: 0,
                    path: PathBuf::from(rel_path_str),
                    size: metadata.size(),
                    content_hash: hash,
                    last_modified,
                    last_accessed,
                    created,
                },
            );
        }

        Ok(fs_map)
    }

    /// Oblicza hash SHA-256 pliku (strumieniowo dla dużych plików)
    fn hash_file(path: &Path) -> Result<String, String> {
        use std::fs::File;
        use std::io::BufReader;

        let file = File::open(path).map_err(|e| e.to_string())?;
        let mut reader = BufReader::new(file);
        let mut hasher = Sha256::new();
        let mut buffer = [0; 65536]; // 64KB buffer

        loop {
            let bytes_read = reader.read(&mut buffer).map_err(|e| e.to_string())?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }

        let result = hasher.finalize();
        Ok(format!("{:x}", result))
    }

    pub fn disconnect(&mut self) {
        self.is_initialized = false;
    }

    /// Pobiera referencję do bazy danych
    pub fn db(&self) -> &Database {
        &self.db
    }

    /// Pobiera mutowalną referencję do bazy danych
    pub fn db_mut(&mut self) -> &mut Database {
        &mut self.db
    }

    /// Pobiera referencje do ścieżki root
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Pobiera wartość inicjalizacji
    pub fn is_initialized(&self) -> bool {
        self.is_initialized
    }

    /// Pobiera ścieżkę do katalogu konfiguracyjnego tager
    pub fn tager_dir(&self) -> &PathBuf {
        &self.tager_dir
    }

    /// Pobiera ścieżkę do pliku bazy danych
    pub fn db_path(&self) -> &PathBuf {
        &self.db_path
    }
}
