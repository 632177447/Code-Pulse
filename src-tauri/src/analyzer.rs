use regex::Regex;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use walkdir::WalkDir;

// js/ts: import { ... } from "..." | import "..." | require("...")
static JS_RE: OnceLock<Regex> = OnceLock::new();
// python: import ... | from ... import ...
static PY_RE: OnceLock<Regex> = OnceLock::new();
// rust: use ...; | mod ...;
static RS_RE: OnceLock<Regex> = OnceLock::new();

fn get_js_re() -> &'static Regex {
    JS_RE.get_or_init(|| {
        Regex::new(r#"(?:import.*from\s+['"]([^'"]+)['"]|require\(['"]([^'"]+)['"]\)|import\s+['"]([^'"]+)['"])"#).unwrap()
    })
}

fn get_py_re() -> &'static Regex {
    PY_RE.get_or_init(|| {
        Regex::new(r#"^\s*(?:import|from)\s+([a-zA-Z0-9_\.]+)"#).unwrap()
    })
}

fn get_rs_re() -> &'static Regex {
    RS_RE.get_or_init(|| {
        Regex::new(r#"^\s*(?:use|mod)\s+([a-zA-Z0-9_:]+)"#).unwrap()
    })
}

fn extract_dependencies(content: &str, ext: &str) -> Vec<String> {
    let mut deps = Vec::new();
    match ext {
        "js" | "jsx" | "ts" | "tsx" | "vue" | "svelte" => {
            let re = get_js_re();
            for cap in re.captures_iter(content) {
                if let Some(m) = cap.get(1).or(cap.get(2)).or(cap.get(3)) {
                    deps.push(m.as_str().to_string());
                }
            }
        }
        "py" => {
            let re = get_py_re();
            for cap in re.captures_iter(&content.replace("\r\n", "\n")) {
                if let Some(m) = cap.get(1) {
                    deps.push(m.as_str().replace('.', "/"));
                }
            }
        }
        "rs" => {
            let re = get_rs_re();
            for cap in re.captures_iter(&content.replace("\r\n", "\n")) {
                if let Some(m) = cap.get(1) {
                    deps.push(m.as_str().replace("::", "/"));
                }
            }
        }
        _ => {}
    }
    deps
}

fn resolve_path(base_dir: &Path, import_path: &str, ext: &str) -> Option<PathBuf> {
    if !import_path.starts_with(".") && !import_path.starts_with("/") {
        return None;
    }
    
    let target = base_dir.join(import_path);
    if target.exists() && target.is_file() {
        return Some(target);
    }
    
    let extensions = match ext {
        "js" | "jsx" | "ts" | "tsx" | "vue" => vec!["ts", "js", "tsx", "jsx", "vue"],
        "py" => vec!["py"],
        "rs" => vec!["rs"],
        _ => vec![],
    };

    for e in &extensions {
        let with_ext = target.with_extension(e);
        if with_ext.exists() {
            return Some(with_ext);
        }
    }
    
    if target.is_dir() {
        for e in &extensions {
            let index_path = target.join(format!("index.{}", e));
            if index_path.exists() {
                return Some(index_path);
            }
        }
    }

    None
}

fn find_project_root(start_path: &Path) -> PathBuf {
    let mut current = start_path;
    loop {
        if current.join("package.json").exists() || current.join("Cargo.toml").exists() || current.join(".git").exists() {
            if let Ok(canon) = current.canonicalize() {
                return canon;
            }
        }
        if let Some(parent) = current.parent() {
            current = parent;
        } else {
            break;
        }
    }
    
    let fallback = if start_path.is_dir() {
        start_path.to_path_buf()
    } else {
        start_path.parent().unwrap_or(Path::new("")).to_path_buf()
    };
    if let Ok(canon) = fallback.canonicalize() {
        canon
    } else {
        fallback
    }
}

pub fn analyze_dependencies(paths: Vec<String>, max_depth: usize) -> Result<String, String> {
    let mut visited: HashSet<PathBuf> = HashSet::new();
    let mut result_blocks: Vec<String> = Vec::new();

    let supported_exts = vec!["js", "jsx", "ts", "tsx", "vue", "svelte", "py", "rs"];

    for p_str in paths {
        let path = Path::new(&p_str);
        if !path.exists() { continue; }

        let base_path = find_project_root(path);

        if path.is_dir() {
            for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
                let e_path = entry.path();
                if e_path.is_file() {
                    let ext = e_path.extension().and_then(|e| e.to_str()).unwrap_or("");
                    if supported_exts.contains(&ext) {
                        let path_str = e_path.to_string_lossy();
                        if path_str.contains("node_modules") || path_str.contains(".git") || path_str.contains("dist") || path_str.contains("target") {
                            continue;
                        }
                        process_file(e_path, 0, max_depth, &mut visited, &mut result_blocks, &base_path);
                    }
                }
            }
        } else {
            process_file(path, 0, max_depth, &mut visited, &mut result_blocks, &base_path);
        }
    }

    Ok(result_blocks.join("\n\n"))
}

fn process_file(
    path: &Path, 
    current_depth: usize, 
    max_depth: usize, 
    visited: &mut HashSet<PathBuf>, 
    result_blocks: &mut Vec<String>,
    base_path: &Path
) {
    if current_depth > max_depth || !path.exists() { return; }
    
    let abs_path = match path.canonicalize() { Ok(p) => p, Err(_) => return };
    if visited.contains(&abs_path) || abs_path.as_os_str().is_empty() { return; }
    
    let path_str = abs_path.to_string_lossy();
    if path_str.contains("node_modules") || path_str.contains(".git") || path_str.contains("dist") || path_str.contains("target") {
        return;
    }
    
    visited.insert(abs_path.clone());
    
    if let Ok(content) = fs::read_to_string(&abs_path) {
        let mut display_path_str = abs_path.to_string_lossy().into_owned();
        if let Ok(rel_path) = abs_path.strip_prefix(base_path) {
            display_path_str = rel_path.to_string_lossy().replace("\\", "/");
        } else {
            display_path_str = display_path_str.replace("\\", "/");
            // Also try to strip UNC prefix if present
            if display_path_str.starts_with("//?/") {
                display_path_str = display_path_str[4..].to_string();
            }
        }

        result_blocks.push(format!(
            "========================================\n[FILE PATH]: {}\n(Dependency Layer: {})\n========================================\n[CONTENT START]\n{}\n[CONTENT END]", 
            display_path_str, current_depth, content
        ));
        
        let ext = abs_path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let base_dir = abs_path.parent().unwrap_or(Path::new(""));
        
        for dep in extract_dependencies(&content, ext) {
            if let Some(resolved) = resolve_path(base_dir, &dep, ext) {
                process_file(&resolved, current_depth + 1, max_depth, visited, result_blocks, base_path);
            }
        }
    }
}
