mod analyzer;
mod minimizer;
mod cache;
mod api_server;
use std::path::{Path};
use std::fs;
use std::sync::{Arc};
use std::sync::atomic::{AtomicBool, Ordering};
use api_server::types::ContextRequest;

#[derive(Clone)]
pub struct AppState {
    pub abort_handle: Arc<AtomicBool>,
    pub parse_cache: Arc<cache::FileCache>,
}

async fn run_generate_context(
    app_state: AppState,
    request: ContextRequest,
) -> Result<Vec<analyzer::FileNode>, String> {
    app_state.abort_handle.store(false, Ordering::SeqCst);
    let abort_handle = app_state.abort_handle.clone();
    let parse_cache = app_state.parse_cache.clone();

    // 将 CPU 密集型的同步文件遍历移到专用 blocking 线程池
    // 避免占用 Tauri 的异步调度线程，从而解除对 webview IPC 通道的阻塞
    tauri::async_runtime::spawn_blocking(move || {
        analyzer::analyze_dependencies(
            request.paths,
            request.max_depth,
            request.ignore_exts,
            request.ignore_deep_parse,
            request.included_types,
            request.project_roots,
            request.enable_minimization,
            request.minimization_threshold,
            request.minimization_depth_threshold,
            Some(abort_handle),
            parse_cache,
        )
    }).await.map_err(|e| e.to_string())?
}

#[tauri::command]
async fn generate_context(
    state: tauri::State<'_, AppState>,
    paths: Vec<String>, 
    max_depth: usize, 
    ignore_exts: String, 
    ignore_deep_parse: String, 
    included_types: Vec<String>, 
    project_roots: String, 
    enable_minimization: bool,
    minimization_threshold: usize,
    minimization_depth_threshold: usize
) -> Result<Vec<analyzer::FileNode>, String> {
    run_generate_context(
        state.inner().clone(),
        ContextRequest {
            paths,
            max_depth,
            ignore_exts,
            ignore_deep_parse,
            included_types,
            project_roots,
            enable_minimization,
            minimization_threshold,
            minimization_depth_threshold,
        },
    ).await
}

#[tauri::command]
fn abort_generate_context(state: tauri::State<'_, AppState>) {
    state.abort_handle.store(true, Ordering::SeqCst);
}

#[tauri::command]
fn clear_cache(state: tauri::State<'_, AppState>) {
    state.parse_cache.clear();
}

#[tauri::command]
async fn start_api_server(app: tauri::AppHandle, state: tauri::State<'_, api_server::ApiServerState>, port: u16) -> Result<(), String> {
    api_server::start_server(app, state, port).await
}

#[tauri::command]
async fn stop_api_server(state: tauri::State<'_, api_server::ApiServerState>) -> Result<(), String> {
    api_server::stop_server(state).await
}

#[tauri::command]
async fn api_response(state: tauri::State<'_, api_server::ApiServerState>, id: String, response: api_server::ApiResponse) -> Result<(), String> {
    if let Some((_, tx)) = state.pending_requests.remove(&id) {
        let _ = tx.send(response);
    }
    Ok(())
}

#[derive(serde::Deserialize)]
#[serde(tag = "action")]
enum AiCommand {
    #[serde(rename = "write")]
    Write { path: String, content: String },
    #[serde(rename = "patch")]
    Patch { path: String, search: String, replace: String },
    #[serde(rename = "delete")]
    Delete { path: String },
    #[serde(rename = "move")]
    Move { path: String, target: String },
}

fn resolve_safe_path(path_str: &str, project_roots: &[String]) -> Result<std::path::PathBuf, String> {
    let path = Path::new(path_str);
    
    // 防止路径包含 .. 导致越权访问
    if path.components().any(|c| matches!(c, std::path::Component::ParentDir)) {
        return Err(format!("Permission denied (traversal): {}", path_str));
    }

    if !path.is_absolute() {
        return Err(format!("Permission denied (absolute path required): {}", path_str));
    }

    if project_roots.iter().any(|root| path.starts_with(Path::new(root))) {
        Ok(path.to_path_buf())
    } else {
        Err(format!("Permission denied (outside root): {}", path_str))
    }
}

#[tauri::command]
async fn execute_ai_commands(commands_json: String, project_roots: Vec<String>) -> Result<(), String> {
    let commands: Vec<AiCommand> = serde_json::from_str(&commands_json)
        .map_err(|e| format!("Invalid JSON format: {}", e))?;

    for cmd in commands {
        match cmd {
            AiCommand::Write { path, content } => {
                let safe_path = resolve_safe_path(&path, &project_roots)?;
                if let Some(parent) = safe_path.parent() {
                    fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
                }
                fs::write(safe_path, content).map_err(|e| format!("Failed to write file: {}", e))?;
            }
            AiCommand::Patch { path, search, replace } => {
                let safe_path = resolve_safe_path(&path, &project_roots)?;
                let content = fs::read_to_string(&safe_path).map_err(|e| format!("Failed to read file: {}", e))?;
                let new_content = content.replace(&search, &replace);
                if content == new_content && !search.is_empty() {
                    return Err(format!("Search string not found in file: {}", path));
                }
                fs::write(&safe_path, new_content).map_err(|e| format!("Failed to update file: {}", e))?;
            }
            AiCommand::Delete { path } => {
                let safe_path = resolve_safe_path(&path, &project_roots)?;
                fs::remove_file(safe_path).map_err(|e| format!("Failed to delete file: {}", e))?;
            }
            AiCommand::Move { path, target } => {
                let safe_path = resolve_safe_path(&path, &project_roots)?;
                let safe_target = resolve_safe_path(&target, &project_roots)?;
                
                if let Some(parent) = safe_target.parent() {
                    fs::create_dir_all(parent).map_err(|e| format!("Failed to create target directory: {}", e))?;
                }
                fs::rename(safe_path, safe_target).map_err(|e| format!("Failed to move file: {}", e))?;
            }
        }
    }
    Ok(())
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
        .manage(AppState {
            abort_handle: Arc::new(AtomicBool::new(false)),
            parse_cache: Arc::new(cache::FileCache::new()),
        })
        .manage(api_server::ApiServerState::new())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .invoke_handler(tauri::generate_handler![
            generate_context, 
            copy_files_to_dest,
            abort_generate_context,
            clear_cache,
            start_api_server,
            stop_api_server,
            api_response,
            execute_ai_commands
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
