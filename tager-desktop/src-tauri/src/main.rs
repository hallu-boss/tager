
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            initialize_directory,
            get_files,
            add_tag,
            remove_tag,
            get_tags_for_file,
            search_files,
            get_all_tags,
            get_untagged_files,
            refresh_directory,
            read_directory_with_metadata,
            read_file_as_base64
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
