mod config;
mod db;

use clap::{Parser, Subcommand};
use db::{Database, FilesOrderBy};
use tokio;

#[derive(Parser)]
#[command(name = "tager", about = "CLI do zarządzania tagami plików")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Inicjalizuje tager w podanym katalogu
    Init,
    /// Dodaje tag do pliku o wskazanym ID
    Add {
        /// ID pliku (z tabeli files)
        id: i64,
        /// Nazwa taga
        tag: String,
    },
    /// Wypisuje wszystkie pliki posiadające wskazany tag
    List {
        /// Nazwa taga
        tag: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => {
            let root = std::env::current_dir()?;
            let db = Database::new_file(&root, None).await?;
            let count = db.rebuild().await?;
            println!(
                "✅ Zainicjalizowano tager w {:?}, dodano {} nowych plików",
                root, count
            );
        }

        Commands::Add { id, tag } => {
            let db = Database::new_file(std::env::current_dir()?, None).await?;

            db.assign_tag_to_file_by_id(id, &tag).await?;
            println!("✅ Dodano tag '{}' do pliku ID={}", tag, id);
        }

        Commands::List { tag } => {
            let db = Database::new_file(std::env::current_dir()?, None).await?;
            let files = db.get_files_for_tag(&tag, Some(FilesOrderBy::Path)).await?;

            if files.is_empty() {
                println!("Brak plików z tagiem '{}'", tag);
            } else {
                println!("📄 Pliki z tagiem '{}':", tag);
                for file in files {
                    println!("  [{}] {}  {:?}", file.id, file.path, file.tags);
                }
            }
        }
    }

    Ok(())
}
