use std::{
    env,
    fs::{self, File},
    io::Write,
    path::PathBuf,
    process,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use tager::tm::TagerManager;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        eprintln!("Użycie: tager-test <ilość_plików> <rozmiar_MB>");
        process::exit(1);
    }

    let file_count: usize = args[1]
        .parse()
        .expect("Niepoprawna ilość plików");

    let file_size_mb: usize = args[2]
        .parse()
        .expect("Niepoprawny rozmiar pliku (MB)");

    // ===== 1. Utwórz katalog w /tmp =====
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let root: PathBuf = format!("/tmp/tager-test-{}", ts).into();
    fs::create_dir_all(&root).expect("Nie udało się utworzyć katalogu");

    println!("📁 Katalog testowy: {}", root.display());

    // ===== 2. Wygeneruj pliki =====
    generate_files(&root, file_count, file_size_mb)
        .expect("Błąd podczas generowania plików");

    println!(
        "📄 Utworzono {} plików po {} MB",
        file_count, file_size_mb
    );

    // ===== 3. TagerManager::new =====
    let mut manager = TagerManager::new(&root)
        .await
        .expect("Błąd tworzenia TagerManager");

    // ===== 4. init() + pomiar czasu =====
    let init_start = Instant::now();
    manager.init().await.expect("Błąd init()");
    let init_time = init_start.elapsed();

    println!("⏱️ init() zajęło: {:.3?}", init_time);

    // ===== 5. sync() + pomiar czasu =====
    let sync_start = Instant::now();
    manager.sync().await.expect("Błąd sync()");
    let sync_time = sync_start.elapsed();

    println!("⏱️ sync() zajęło: {:.3?}", sync_time);

    println!(" ilość plików w bd {}", manager.db().get_all_files(None, None).await.unwrap().len());

    let sync_start = Instant::now();
    manager.sync().await.expect("Błąd sync()");
    let sync_time = sync_start.elapsed();

    println!("⏱️ 2 sync() zajęło: {:.3?}", sync_time);

    println!("2- ilość plików w bd {}", manager.db().get_all_files(None, None).await.unwrap().len());

    println!("✅ Test zakończony");
}

/// Generuje pliki testowe
fn generate_files(
    root: &PathBuf,
    count: usize,
    size_mb: usize,
) -> Result<(), std::io::Error> {
    let size_bytes = size_mb * 1024 * 1024;

    for i in 0..count {
        let path = root.join(format!("file_{:05}.bin", i));
        let mut file = File::create(path)?;

        // Stała, przewidywalna zawartość (dobra do benchmarków)
        let buffer = vec![(i % 256) as u8; size_bytes];
        file.write_all(&buffer)?;
    }

    Ok(())
}
