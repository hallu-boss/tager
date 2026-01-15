use std::{
    collections::{HashMap, HashSet},
    io::{self, Read}, // Dodane
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
};

use sha2::{Digest, Sha256};
use walkdir::WalkDir;

use crate::tm::{EntryType, db::{DBFile, Database, DbError}, get_entry_type};
use std::fs;

pub struct TagEntry {
    pub id: i64,
    pub name: String,
}

pub struct FileEntry {
    pub id: i64,
    pub abs_path: String,
    pub rel_path: String,
    pub file_name: String,
    pub size: u64,
    pub r#type: EntryType,
    pub tags: Vec<TagEntry>,
    pub last_modified: String,
    pub created: String,
}

/// TagerManager - główna struktura zarządzająca systemem tagowania
pub struct TagerManager {
    /// Ścieżka do katalogu głównego
    root: PathBuf,
    /// Ścieżka do katalogu .tager w root
    tager_dir: PathBuf,
    /// Ścieżka do pliku bazy danych
    db_path: PathBuf,
    /// Ścieżka do pliku z hashem katalogu
    hash_path: PathBuf,
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
        let hash_path = tager_dir.join("root_hash");

        let db = Database::new_in_memory()
            .await
            .map_err(|e| format!("Nie udało się utworzyć tymczasowej bazy danych: {}", e))?;

        Ok(Self {
            root,
            tager_dir,
            db_path,
            hash_path,
            db,
            is_initialized: false,
        })
    }

    /// Inicjalizuje system tagowania w katalogu
    /// - Tworzy katalog .tager jeśli nie istnieje
    /// - Tworzy bazę danych w pliku
    /// - Inicjalizuje schemat bazy danych
    pub async fn init(&mut self) -> Result<(), DbError> {
        if !self.tager_dir.exists() {
            fs::create_dir_all(&self.tager_dir).map_err(DbError::Io)?;
            println!("Utworzono katalog: {}", self.tager_dir.display());
        }

        self.db = Database::new_from_file(&self.db_path).await?;
        self.is_initialized = true;
        
        Ok(())
    }

    /// Oblicza hash całego katalogu root (z wyłączeniem .tager)
    fn calculate_root_hash(&self) -> Result<String, String> {
        let mut hasher = Sha256::new();
        let mut file_hashes = Vec::new();

        // Zbierz i posortuj wszystkie pliki dla deterministycznego wyniku
        for entry in WalkDir::new(&self.root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter(|e| !e.path().starts_with(&self.tager_dir))
        {
            let hash = Self::hash_file(entry.path())
                .map_err(|e| format!("Błąd hashowania {}: {}", entry.path().display(), e))?;
            
            // Dodaj ścieżkę względną i hash do listy
            let rel_path = entry.path()
                .strip_prefix(&self.root)
                .map_err(|e| format!("Błąd konwersji ścieżki: {}", e))?
                .to_string_lossy();
            
            file_hashes.push((rel_path.to_string(), hash));
        }

        // Posortuj dla deterministycznego hasha
        file_hashes.sort_by(|a, b| a.0.cmp(&b.0));

        // Dodaj wszystkie posortowane pary (ścieżka, hash) do głównego hashera
        for (path, hash) in file_hashes {
            hasher.update(path.as_bytes());
            hasher.update(hash.as_bytes());
        }

        let result = hasher.finalize();
        Ok(format!("{:x}", result))
    }

    /// Oblicza i zapisuje hash katalogu root
    fn calculate_and_save_root_hash(&self) -> Result<(), String> {
        let hash = self.calculate_root_hash()?;
        self.write_root_hash(&hash)
    }

    /// Zapisuje hash katalogu root
    fn write_root_hash(&self, hash: &str) -> Result<(), String> {
        fs::write(&self.hash_path, hash)
            .map_err(|e| format!("Nie udało się zapisać hash: {}", e))
    }

 /// Synchronizuje rekordy bazy danych z zawartością katalogu root
    pub async fn sync(&self) -> Result<(), String> {
        if !self.is_initialized {
            return Err(
                "TagerManager nie jest zainicjalizowany. Wywołaj init() przed sync()".to_string(),
            );
        }

        // Sprawdź czy hash się zgadza
        if self.should_skip_sync()? {
            println!("Hash katalogu root się nie zmienił. Pomijam synchronizację.");
            return Ok(());
        }

        // Przeprowadź pełną synchronizację
        self.full_sync().await?;

        // Oblicz i zapisz nowy hash
        self.calculate_and_save_root_hash()?;

        Ok(())
    }

    /// Sprawdza czy synchronizacja jest potrzebna porównując hashe
    fn should_skip_sync(&self) -> Result<bool, String> {
        let current_hash = self.calculate_root_hash()
            .map_err(|e| format!("Nie udało się obliczyć aktualnego hash: {}", e))?;
        
        let saved_hash = self.read_root_hash()
            .map_err(|e| format!("Nie udało się odczytać zapisanego hash: {}", e))?;
        
        match saved_hash {
            Some(saved) => Ok(saved == current_hash),
            None => Ok(false), // Brak zapisanego hash = zawsze wykonaj sync
        }
    }

    /// Czyta zapisany hash katalogu root
    fn read_root_hash(&self) -> Result<Option<String>, io::Error> {
        if self.hash_path.exists() {
            fs::read_to_string(&self.hash_path).map(Some)
        } else {
            Ok(None)
        }
    }

    /// Pełna synchronizacja (oryginalna logika)
    async fn full_sync(&self) -> Result<(), String> {
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

        // Tworzymy mapę hash -> lista ścieżek
        let mut hash_to_paths: HashMap<String, Vec<String>> = HashMap::new();
        for (path, db_file) in &db_map {
            hash_to_paths
                .entry(db_file.content_hash.clone())
                .or_insert_with(Vec::new)
                .push(path.clone());
        }

        // Synchronizacja plików
        for (path, fs_file) in &fs_map {
            match db_map.get(path) {
                Some(db_file) => {
                    if db_file.content_hash != fs_file.content_hash {
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
                    db_map.remove(path);
                }
                None => {
                    let mut potential_move = None;
                    if let Some(paths) = hash_to_paths.get(&fs_file.content_hash) {
                        for old_path in paths {
                            if let Some(old_file) = db_map.get(old_path) {
                                if old_file.size == fs_file.size
                                    && old_file.content_hash == fs_file.content_hash
                                {
                                    potential_move = Some(old_file.id);
                                    db_map.remove(old_path);
                                    break;
                                }
                            }
                        }
                    }

                    if let Some(file_id) = potential_move {
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

        // Usuń pozostałe pliki z bazy
        for (_, db_file) in db_map {
            self.db
                .delete_file(db_file.id)
                .await
                .map_err(|e| e.to_string())?;
        }

        Ok(())
    }

     /// Przypisuje tag do pliku (lub tworzy nowy tag jeśli nie istnieje)
    pub async fn assign_tag_to_file(
        &self,
        file_path: &Path,
        tag_name: &str,
    ) -> Result<(), DbError> {
        if !self.is_initialized {
            return Err(DbError::OperationFailed(
                "TagerManager nie jest zainicjalizowany. Wywołaj init() przed użyciem".to_string(),
            ));
        }

        // Znajdź plik po ścieżce
        let files = self.db.get_files_by_path(file_path.to_path_buf()).await?;
        if files.is_empty() {
            return Err(DbError::OperationFailed(
                format!("Plik '{}' nie istnieje w bazie danych", file_path.display())
            ));
        }

        // Używamy pierwszego znalezionego pliku
        let file = &files[0];
        
        // Dodaj tag do pliku (tworzy tag jeśli nie istnieje)
        self.db.add_tag_by_name_to_file(file.id, tag_name).await?;
        
        Ok(())
    }

    /// Zmienia nazwę istniejącego tagu
    pub async fn rename_tag(&self, old_name: &str, new_name: &str) -> Result<(), DbError> {
        if !self.is_initialized {
            return Err(DbError::OperationFailed(
                "TagerManager nie jest zainicjalizowany".to_string(),
            ));
        }

        // Sprawdź czy tag istnieje
        let tag = self.db.get_tag_by_name(old_name).await?;
        if tag.is_none() {
            return Err(DbError::OperationFailed(
                format!("Tag '{}' nie istnieje", old_name)
            ));
        }

        let tag = tag.unwrap();
        
        // Sprawdź czy nowa nazwa nie jest już używana
        if let Some(_) = self.db.get_tag_by_name(new_name).await? {
            return Err(DbError::OperationFailed(
                format!("Tag '{}' już istnieje", new_name)
            ));
        }

        // Zmień nazwę
        self.db.update_tag(tag.id, new_name).await?;
        
        Ok(())
    }

    /// Usuwa tag z pliku
    pub async fn remove_tag_from_file(
        &self,
        file_path: &Path,
        tag_name: &str,
    ) -> Result<(), DbError> {
        if !self.is_initialized {
            return Err(DbError::OperationFailed(
                "TagerManager nie jest zainicjalizowany".to_string(),
            ));
        }

        // Znajdź plik po ścieżce
        let files = self.db.get_files_by_path(file_path.to_path_buf()).await?;
        if files.is_empty() {
            return Err(DbError::OperationFailed(
                format!("Plik '{}' nie istnieje w bazie danych", file_path.display())
            ));
        }

        let file = &files[0];
        
        // Usuń tag z pliku
        self.db.remove_tag_by_name_from_file(file.id, tag_name).await?;
        
        Ok(())
    }

    /// Całkowicie usuwa tag z systemu (wraz z powiązaniami)
    pub async fn delete_tag_completely(&self, tag_name: &str) -> Result<(), DbError> {
        if !self.is_initialized {
            return Err(DbError::OperationFailed(
                "TagerManager nie jest zainicjalizowany".to_string(),
            ));
        }

        // Usuń tag (metoda w bazie usuwa też powiązania dzięki CASCADE)
        self.db.delete_tag_by_name(tag_name).await?;
        
        Ok(())
    }

     /// Pobiera wszystkie tagi w systemie
    pub async fn get_all_tags(&self) -> Result<Vec<TagEntry>, DbError> {
        if !self.is_initialized {
            return Err(DbError::OperationFailed(
                "TagerManager nie jest zainicjalizowany".to_string(),
            ));
        }

        let db_tags = self.db.get_all_tags().await?;
        let tags: Vec<TagEntry> = db_tags
            .into_iter()
            .map(|t| TagEntry { id: t.id, name: t.name })
            .collect();
        
        Ok(tags)
    }

     /// Pobiera listę plików z możliwością filtrowania po nazwie i tagach
    pub async fn get_files(
        &self,
        name_filter: Option<String>,
        tag_filters: Option<Vec<String>>,
    ) -> Result<Vec<FileEntry>, DbError> {
        if !self.is_initialized {
            return Err(DbError::OperationFailed(
                "TagerManager nie jest zainicjalizowany".to_string(),
            ));
        }

        // Pobierz wszystkie pliki z bazy
        let mut all_files = self.db.get_all_files(None, None).await?;
        
        // Filtruj po nazwie jeśli podano
        if let Some(name_filter) = name_filter {
            all_files.retain(|file| {
                let file_name = file.path.file_name()
                    .map(|n| n.to_string_lossy().to_lowercase())
                    .unwrap_or_default();
                file_name.contains(&name_filter.to_lowercase())
            });
        }
        
        // Filtruj po tagach jeśli podano
        if let Some(tag_filters) = tag_filters {
            if !tag_filters.is_empty() {
                let filtered_files = self.get_files_by_tags(&tag_filters).await?;
                
                // Zachowaj tylko pliki, które są w obu listach
                let filtered_ids: HashSet<i64> = filtered_files.iter().map(|f| f.id).collect();
                all_files.retain(|file| filtered_ids.contains(&file.id));
            }
        }
        
        // Konwertuj DBFile na FileEntry
        let mut file_entries = Vec::new();
        for file in all_files {
            // Pobierz tagi dla pliku
            let db_tags = self.db.get_tags_for_file(file.id).await?;
            let tags: Vec<TagEntry> = db_tags
                .into_iter()
                .map(|t| TagEntry { id: t.id, name: t.name })
                .collect();
            
            let abs_path = self.root.join(&file.path);
            
            let file_entry = FileEntry {
                id: file.id,
                abs_path: abs_path.to_string_lossy().to_string(),
                rel_path: file.path.to_string_lossy().to_string(),
                file_name: file.path.file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default(),
                size: file.size,
                tags,
                last_modified: format!("{:?}", file.last_modified),
                created: format!("{:?}", file.created),
                r#type: get_entry_type(file.path.as_path()),
            };
            
            file_entries.push(file_entry);
        }
        
        Ok(file_entries)
    }

    /// Pomocnicza metoda do pobierania plików z określonymi tagami
    async fn get_files_by_tags(&self, tags: &[String]) -> Result<Vec<DBFile>, DbError> {
        if tags.is_empty() {
            return Ok(Vec::new());
        }
        
        let mut result_files: Option<Vec<DBFile>> = None;
        
        for tag in tags {
            let files_with_tag = self.db.get_files_with_tag_name(tag).await?;
            
            if result_files.is_none() {
                result_files = Some(files_with_tag);
            } else {
                let current_files = result_files.take().unwrap();
                let current_ids: HashSet<i64> = current_files.iter().map(|f| f.id).collect();
                let new_ids: HashSet<i64> = files_with_tag.iter().map(|f| f.id).collect();
                
                // Przecięcie zbiorów - pliki które mają WSZYSTKIE wymagane tagi
                let intersection_ids: HashSet<_> = current_ids.intersection(&new_ids).cloned().collect();
                
                let mut intersection_files = Vec::new();
                for file in current_files {
                    if intersection_ids.contains(&file.id) {
                        intersection_files.push(file);
                    }
                }
                
                result_files = Some(intersection_files);
            }
        }
        
        Ok(result_files.unwrap_or_default())
    }

    /// Pobiera listę plików bez żadnych tagów
    pub async fn get_files_without_tags(&self) -> Result<Vec<FileEntry>, DbError> {
        if !self.is_initialized {
            return Err(DbError::OperationFailed(
                "TagerManager nie jest zainicjalizowany".to_string(),
            ));
        }

        // Pobierz wszystkie pliki z bazy
        let all_files = self.db.get_all_files(None, None).await?;
        
        let mut files_without_tags = Vec::new();
        
        for file in all_files {
            // Pobierz tagi dla pliku
            let tags = self.db.get_tags_for_file(file.id).await?;
            
            // Jeśli plik nie ma tagów, dodaj go do wyników
            if tags.is_empty() {
                let abs_path = self.root.join(&file.path);
                
                let file_entry = FileEntry {
                    id: file.id,
                    abs_path: abs_path.to_string_lossy().to_string(),
                    rel_path: file.path.to_string_lossy().to_string(),
                    file_name: file.path.file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default(),
                    size: file.size,
                    tags: Vec::new(), // Puste tagi
                    last_modified: format!("{:?}", file.last_modified),
                    created: format!("{:?}", file.created),
                    r#type: get_entry_type(file.path.as_path())
                };
                
                files_without_tags.push(file_entry);
            }
        }
        
        Ok(files_without_tags)
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
