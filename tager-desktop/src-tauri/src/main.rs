#[tauri::command]
fn my_rust_function(name: String) -> String {
    format!("Witaj, {}!", name)
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![my_rust_function])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
