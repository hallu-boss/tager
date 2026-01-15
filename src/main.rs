use std::env;
use std::process;

use tager::tm::TagerManager;


#[tokio::main]
async fn main() {
    // Pobierz argumenty CLI
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Użycie: tager <ścieżka_do_katalogu>");
        process::exit(1);
    }

    let root_path = &args[1];

    // Utwórz TagerManager
    let mut manager = match TagerManager::new(root_path).await {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Błąd inicjalizacji TagerManager: {}", e);
            process::exit(1);
        }
    };

    // Inicjalizacja (.tager + baza)
    if let Err(e) = manager.init().await {
        eprintln!("Błąd init(): {}", e);
        process::exit(1);
    }

    // Synchronizacja
    if let Err(e) = manager.sync().await {
        eprintln!("Błąd sync(): {}", e);
        process::exit(1);
    }

    println!("Synchronizacja zakończona pomyślnie ✅");
}
