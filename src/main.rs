use std::{path::Path};
use tager::db::Database;

const ROOT_DIR: &str = "/home/pawel/Desktop/test-data";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root_path = Path::new(ROOT_DIR);

    let db = Database::new_file( &root_path, None).await?;
    println!("✅ Baza danych zainicjalizowana");

    let count = db.rebuild().await?;
    println!("🔄 Skanowanie katalogu zakończone {count}");

    let files = db.get_untaged_files().await?;

    for (id, file) in &files {
        println!(" - {}. {}", id, file);
    }

    Ok(())
}
