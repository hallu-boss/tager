use std::path::{Path, PathBuf};

use crate::tm::db::{Database, DbError};

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
    pub async fn new<P: AsRef<Path>>(root: P) -> Self {
        let root = root.as_ref().to_path_buf();
        let tager_dir = root.join(".tager");
        let db_path = tager_dir.join("db.sqlite");
        
        // Tworzymy tymczasową bazę w pamięci jako placeholder
        // Zostanie zastąpiona podczas init()
        // Uwaga: W rzeczywistości możemy opóźnić tworzenie Database do init()
        // Ale dla prostoty tworzymy tymczasową instancję
        let db = Database::new_in_memory().await.unwrap_or_else(|_| {
            panic!("Nie udało się utworzyć tymczasowej bazy danych")
        });
        
        Self {
            root,
            tager_dir,
            db_path,
            db,
            is_initialized: false,
        }
    }
    
    /// Inicjalizuje system tagowania w katalogu
    /// - Tworzy katalog .tager jeśli nie istnieje
    /// - Tworzy bazę danych w pliku
    /// - Inicjalizuje schemat bazy danych
    pub async fn init(&mut self) -> Result<(), DbError> {
        // Utwórz katalog .tager jeśli nie istnieje
        if !self.tager_dir.exists() {
            std::fs::create_dir_all(&self.tager_dir)
                .map_err(DbError::Io)?;
            println!("Utworzono katalog: {}", self.tager_dir.display());
        }
        
        // Utwórz bazę danych w pliku
        self.db = Database::new_from_file(&self.db_path).await?;
        
        // Można dodać tutaj dodatkową inicjalizację
        // np. domyślne tagi, konfigurację, etc.
        
        self.is_initialized = true;
        Ok(())
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
    
    /// Pobiera ścieżkę do katalogu konfuguracyjnego tager
    pub fn tager_dir(&self) -> &PathBuf {
        &self.tager_dir
    }
    
    /// Pobiera ścieżkę do pliku bazy danych
    pub fn db_path(&self) -> &PathBuf {
        &self.db_path
    }
}
