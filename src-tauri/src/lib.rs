mod analyzer;
mod minimizer;
use std::path::Path;
use std::fs;

#[tauri::command]
async fn generate_context(paths: Vec<String>, max_depth: usize, ignore_exts: String, ignore_deep_parse: String, included_types: Vec<String>, project_roots: String, enable_minimization: bool) -> Result<Vec<analyzer::FileNode>, String> {
    // 将 CPU 密集型的同步文件遍历移到专用 blocking 线程池
    // 避免占用 Tauri 的异步调度线程，从而解除对 webview IPC 通道的阻塞
    tauri::async_runtime::spawn_blocking(move || {
        analyzer::analyze_dependencies(paths, max_depth, ignore_exts, ignore_deep_parse, included_types, project_roots, enable_minimization)
    }).await.map_err(|e| e.to_string())?
}

fn copy_recursively(source: impl AsRef<Path>, destination: impl AsRef<Path>) -> std::io::Result<()> {
    fs::create_dir_all(&destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let match_path = entry.path();
        let dest_file = destination.as_ref().join(entry.file_name());
        if match_path.is_dir() {
            copy_recursively(match_path, dest_file)?;
        } else {
            fs::copy(match_path, dest_file)?;
        }
    }
    Ok(())
}

#[tauri::command]
fn copy_files_to_dest(sources: Vec<String>, dest_dir: String) -> Result<Vec<String>, String> {
    let mut new_paths = Vec::new();
    let dest_path = Path::new(&dest_dir);
    if !dest_path.exists() {
        if let Err(e) = fs::create_dir_all(dest_path) {
            return Err(e.to_string());
        }
    }

    for source in sources {
        let src_path = Path::new(&source);
        if !src_path.exists() {
            continue;
        }

        let file_name = src_path.file_name().unwrap_or_default();
        let target_path = dest_path.join(file_name);

        if src_path.is_dir() {
            if let Err(e) = copy_recursively(&src_path, &target_path) {
                return Err(format!("Failed to copy folder: {}", e));
            }
        } else {
            if let Err(e) = fs::copy(&src_path, &target_path) {
                return Err(format!("Failed to copy file: {}", e));
            }
        }
        
        new_paths.push(target_path.to_string_lossy().into_owned());
    }

    Ok(new_paths)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![generate_context, copy_files_to_dest])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
