mod analyzer;

#[tauri::command]
fn generate_context(paths: Vec<String>, max_depth: usize, generate_tree: bool) -> Result<String, String> {
    analyzer::analyze_dependencies(paths, max_depth, generate_tree)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![generate_context])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
