use crate::db::{Database, FilesOrderBy};
use crate::tager_manager::TagerManager;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "tager")]
#[command(about = "A file tagging system", long_about = None)]
pub struct Cli {
    /// Root directory to work with (defaults to current directory)
    #[arg(short, long, default_value = ".")]
    root: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize the tager system in the current directory
    Init,
    
    /// List all files with their tags
    List {
        /// Show only files with a specific tag
        #[arg(short, long)]
        tag: Option<String>,
        
        /// Show only untagged files
        #[arg(short, long)]
        untagged: bool,
    },
    
    /// Add a tag to a file
    AddTag {
        /// Path to the file (relative to root)
        file: String,
        
        /// Tag name to add
        tag: String,
    },

    /// Rename tag
    RenameTag {
        // Current tag name
        old_name: String,

        /// New tag name
        new_name: String,
    }
}

impl Cli {
    /// Execute the CLI command
    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        match &self.command {
            Commands::Init => {
                self.init_command().await?;
            }
            Commands::List { tag, untagged } => {
                self.list_command(tag.clone(), *untagged).await?;
            }
            Commands::AddTag { file, tag } => {
                self.add_tag_command(file, tag).await?;
            }
            Commands::RenameTag { old_name, new_name } => {
                self.rename_tag_command(old_name, new_name).await?;
            }
        }
        Ok(())
    }

    /// Initialize the tager system
    async fn init_command(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Inicjalizacja tager w: {}", self.root.display());
        
        let manager = TagerManager::new(&self.root)?;
        let db_path = manager.config_dir().join("tager.db");
        let db = Database::new_file(&db_path).await?;
        
        let added = manager.rebuild(&db).await?;
        
        println!("✓ Katalog konfiguracyjny utworzony: {}", manager.config_dir().display());
        println!("✓ Baza danych utworzona: {}", db_path.display());
        println!("✓ Zindeksowano {} plików", added);
        
        Ok(())
    }

    /// List files with their tags
    async fn list_command(
        &self,
        tag_filter: Option<String>,
        show_untagged: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (manager, db) = self.init_manager_and_db().await?;
        
        if show_untagged {
            Self::list_untagged_files(&db).await?;
        } else if let Some(tag) = tag_filter {
            Self::list_files_with_tag(&db, &tag).await?;
        } else {
            Self::list_all_files(&manager, &db).await?;
        }
        
        Ok(())
    }

    /// Add a tag to a file
    async fn add_tag_command(
        &self,
        file_path: &str,
        tag: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (_manager, db) = self.init_manager_and_db().await?;
        
        // Find file by path
        let file_id = Self::find_file_id(&db, file_path).await?;
        
        // Add tag
        db.assign_tag_to_file_by_id(file_id, tag).await?;
        
        println!("✓ Tag '{}' dodany do pliku '{}'", tag, file_path);
        
        Ok(())
    }

    /// Rename a tag globally
    async fn rename_tag_command(
        &self,
        old_name: &str,
        new_name: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (_manager, db) = self.init_manager_and_db().await?;
        
        db.rename_tag(old_name, new_name).await
            .map_err(|_| {
                format!("Nie można zmienić nazwy tagu: '{}' nie istnieje lub '{}' już istnieje", 
                    old_name, new_name)
            })?;
        
        println!("✓ Tag '{}' zmieniony na '{}'", old_name, new_name);
        
        Ok(())
    }

    /// Initialize manager and database, rebuild if needed
    async fn init_manager_and_db(
        &self,
    ) -> Result<(TagerManager, Database), Box<dyn std::error::Error>> {
        let manager = TagerManager::new(&self.root)?;
        let db_path = manager.config_dir().join("tager.db");
        
        if !db_path.exists() {
            return Err("Baza danych nie istnieje. Uruchom najpierw 'tager init'".into());
        }
        
        let db = Database::new_file(&db_path).await?;
        
        // Rebuild to ensure database is up to date
        let added = manager.rebuild(&db).await?;
        if added > 0 {
            println!("ℹ Zaktualizowano bazę danych: dodano {} nowych plików", added);
        }
        
        Ok((manager, db))
    }

    /// List all files with their tags
    async fn list_all_files(
        manager: &TagerManager,
        db: &Database,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let files = manager.get_all_relative_paths();
        
        if files.is_empty() {
            println!("Brak plików do wyświetlenia");
            return Ok(());
        }
        
        println!("\nWszystkie pliki:\n");
        
        for file_path in files {
            let path_str = file_path.to_string_lossy();
            let tags = db.get_tags_for_file(&file_path).await?;
            
            if tags.is_empty() {
                println!("  {} (bez tagów)", path_str);
            } else {
                println!("  {} [{}]", path_str, tags.join(", "));
            }
        }
        
        Ok(())
    }

    /// List only untagged files
    async fn list_untagged_files(db: &Database) -> Result<(), Box<dyn std::error::Error>> {
        let files = db.get_untagged_files(Some(FilesOrderBy::Path)).await?;
        
        if files.is_empty() {
            println!("Brak plików bez tagów");
            return Ok(());
        }
        
        println!("\nPliki bez tagów:\n");
        
        for (id, path) in files {
            println!("  [{}] {}", id, path);
        }
        
        Ok(())
    }

    /// List files with a specific tag
    async fn list_files_with_tag(
        db: &Database,
        tag: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let files = db.get_files_for_tag(tag, Some(FilesOrderBy::Path)).await?;
        
        if files.is_empty() {
            println!("Brak plików z tagiem '{}'", tag);
            return Ok(());
        }
        
        println!("\nPliki z tagiem '{}':\n", tag);
        
        for file in files {
            println!("  {} [{}]", file.path, file.tags.join(", "));
        }
        
        Ok(())
    }

    /// Find file ID by path
    async fn find_file_id(
        db: &Database,
        file_path: &str,
    ) -> Result<i64, Box<dyn std::error::Error>> {
        use sqlx::Row;
        
        // Query database for file by path
        let result = sqlx::query("SELECT id FROM files WHERE path = ?")
            .bind(file_path)
            .fetch_optional(db.pool())
            .await?;
        
        match result {
            Some(row) => {
                let id: i64 = row.try_get("id")?;
                Ok(id)
            }
            None => Err(format!("Plik '{}' nie został znaleziony w bazie danych", file_path).into()),
        }
    }


}