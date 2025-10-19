use std::{fs, path::Path};
use tager::db::Database;

const ROOT_DIR: &str = "/home/pawel/Desktop/test-data";
const TAGER_DIR_NAME: &str = ".tager";
const TAGER_DB_NAME: &str = "tager.db";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root_path = Path::new(ROOT_DIR);
    let tager_dir = root_path.join(TAGER_DIR_NAME);

    if !tager_dir.exists() {
        fs::create_dir(&tager_dir)?;
        println!("📁 Utworzono katalog bazy danych: {}", tager_dir.display());
    }

    let db_file = tager_dir.join(TAGER_DB_NAME);

    let db = Database::new_file(&db_file, &root_path).await?;
    println!("✅ Baza danych zainicjalizowana w {}", db_file.display());

    let count = db.rebuild().await?;
    println!("🔄 Skanowanie katalogu zakończone {count}");

    Ok(())
}
