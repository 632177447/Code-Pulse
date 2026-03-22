// 核心分析模块：协调文件扫描、依赖提取、路径解析与结果生成

mod constants;
mod regex;
mod deps;
mod resolve;
mod ignore;

use crate::minimizer;
use ::regex::Regex;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use walkdir::WalkDir;

use constants::*;
use deps::{extract_dependencies, extract_vue_component_tags};
use ignore::{parse_ignore_patterns, should_ignore};
use resolve::{build_component_index, detect_auto_import_plugin, find_project_root, resolve_path};

#[derive(Serialize)]
pub struct FileNode {
    pub path: String,
    pub content: String,
    pub abs_path: String,
}

pub fn analyze_dependencies(
    paths: Vec<String>, 
    max_depth: usize, 
    ignore_exts: String, 
    ignore_deep_parse: String, 
    included_types: Vec<String>, 
    project_roots: String, 
    enable_minimization: bool,
    minimization_threshold: usize,
    minimization_depth_threshold: usize,
    abort_handle: Option<Arc<AtomicBool>>,
    parse_cache: Arc<crate::cache::FileCache>
) -> Result<Vec<FileNode>, String> {
    let mut visited: HashSet<PathBuf> = HashSet::new();
    let mut result_blocks: Vec<FileNode> = Vec::new();
    let mut parsed_paths: Vec<String> = Vec::new();
    let mut current_total_size = 0;

    let manual_roots: Vec<PathBuf> = project_roots
        .split(|c| c == ',' || c == '\n' || c == '\r')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .collect();

    let included_types_set: HashSet<String> = if included_types.is_empty() {
        ALL_SUPPORTED_EXTS.iter().map(|&s| s.to_string()).collect()
    } else {
        included_types.into_iter().map(|s| s.to_lowercase()).collect()
    };

    let (ignore_names, ignore_extensions, ignore_filenames, ignore_regexes) = 
        parse_ignore_patterns(&ignore_exts, DEFAULT_IGNORE_PATTERNS);

    let (ignore_deep_names, ignore_deep_extensions, ignore_deep_filenames, ignore_deep_regexes) = 
        parse_ignore_patterns(&ignore_deep_parse, &[]);

    // 每个 project_root 对应的组件索引（懒初始化，避免重复扫描同一根目录）
    let mut component_indices: HashMap<PathBuf, Option<HashMap<String, PathBuf>>> = HashMap::new();

    for p_str in paths {
        let path = Path::new(&p_str);
        if !path.exists() { continue; }

        let base_path = find_project_root(path, &manual_roots);

        // 检测是否安装了自动引入插件，按根目录缓存结果
        let comp_index = component_indices
            .entry(base_path.clone())
            .or_insert_with(|| {
                if detect_auto_import_plugin(&base_path) {
                    Some(build_component_index(
                        &base_path,
                        &ignore_names,
                        &ignore_extensions,
                        &ignore_filenames,
                        &ignore_regexes,
                    ))
                } else {
                    None
                }
            })
            .as_ref();

        if path.is_dir() {
            for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
                if let Some(ref h) = abort_handle {
                    if h.load(Ordering::SeqCst) { return Ok(result_blocks); }
                }
                let e_path = entry.path();
                if e_path.is_file() {
                    let ext = e_path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
                    if included_types_set.contains(&ext) {
                        if should_ignore(e_path, &ignore_names, &ignore_extensions, &ignore_filenames, &ignore_regexes) {
                            continue;
                        }
                        process_file(e_path, 0, max_depth, &mut visited, &mut result_blocks, &mut parsed_paths, &base_path, 
                            &ignore_names, &ignore_extensions, &ignore_filenames, &ignore_regexes,
                            &ignore_deep_names, &ignore_deep_extensions, &ignore_deep_filenames, &ignore_deep_regexes,
                            &included_types_set, enable_minimization, minimization_threshold, minimization_depth_threshold, &mut current_total_size, abort_handle.as_ref(), &parse_cache, comp_index);
                    }
                }
            }
        } else {
            process_file(path, 0, max_depth, &mut visited, &mut result_blocks, &mut parsed_paths, &base_path, 
                &ignore_names, &ignore_extensions, &ignore_filenames, &ignore_regexes,
                &ignore_deep_names, &ignore_deep_extensions, &ignore_deep_filenames, &ignore_deep_regexes,
                &included_types_set, enable_minimization, minimization_threshold, minimization_depth_threshold, &mut current_total_size, abort_handle.as_ref(), &parse_cache, comp_index);
        }
    }

    Ok(result_blocks)
}

fn process_file(
    path: &Path, 
    current_depth: usize, 
    max_depth: usize, 
    visited: &mut HashSet<PathBuf>, 
    result_blocks: &mut Vec<FileNode>,
    parsed_paths: &mut Vec<String>,
    base_path: &Path,
    ignore_names: &HashSet<String>,
    ignore_extensions: &HashSet<String>,
    ignore_filenames: &HashSet<String>,
    ignore_regexes: &[Regex],
    ignore_deep_names: &HashSet<String>,
    ignore_deep_extensions: &HashSet<String>,
    ignore_deep_filenames: &HashSet<String>,
    ignore_deep_regexes: &[Regex],
    included_types: &HashSet<String>,
    enable_minimization: bool,
    minimization_threshold: usize,
    minimization_depth_threshold: usize,
    current_total_size: &mut usize,
    abort_handle: Option<&Arc<AtomicBool>>,
    parse_cache: &crate::cache::FileCache,
    component_index: Option<&HashMap<String, PathBuf>>,
) {
    if let Some(h) = abort_handle {
        if h.load(Ordering::SeqCst) { return; }
    }
    if current_depth > max_depth || !path.exists() { return; }
    
    let abs_path = match path.canonicalize() { Ok(p) => p, Err(_) => return };
    if visited.contains(&abs_path) || abs_path.as_os_str().is_empty() { return; }

    let file_ext = abs_path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    if !included_types.is_empty() && !included_types.contains(&file_ext) {
        return;
    }
    
    if should_ignore(&abs_path, ignore_names, ignore_extensions, ignore_filenames, ignore_regexes) {
        return;
    }
    
    visited.insert(abs_path.clone());

    // 尝试获取文件修改时间并查询缓存
    let mtime = fs::metadata(&abs_path).ok().and_then(|m| m.modified().ok());

    if let Some(t) = mtime {
        if let Some(entry) = parse_cache.get(&abs_path, t) {
            parsed_paths.push(entry.display_path.clone());
            *current_total_size += entry.content.len();
            result_blocks.push(FileNode {
                path: entry.display_path.clone(),
                content: entry.content.clone(),
                abs_path: abs_path.to_string_lossy().into_owned(),
            });

            // 缓存命中时仍需追踪依赖
            if let Ok(content) = fs::read_to_string(&abs_path) {
                let ext = abs_path.extension().and_then(|e| e.to_str()).unwrap_or("");
                if !should_ignore(&abs_path, ignore_deep_names, ignore_deep_extensions, ignore_deep_filenames, ignore_deep_regexes) {
                    let base_dir = abs_path.parent().unwrap_or(Path::new(""));
                    for dep in extract_dependencies(&content, ext) {
                        if let Some(resolved) = resolve_path(base_dir, &dep, ext, base_path) {
                            process_file(&resolved, current_depth + 1, max_depth, visited, result_blocks, parsed_paths, base_path,
                                ignore_names, ignore_extensions, ignore_filenames, ignore_regexes,
                                ignore_deep_names, ignore_deep_extensions, ignore_deep_filenames, ignore_deep_regexes,
                                included_types, enable_minimization, minimization_threshold, minimization_depth_threshold, current_total_size, abort_handle, parse_cache, component_index);
                        }
                    }
                    // Vue 组件自动引入：通过索引解析模板中未显式 import 的组件
                    if ext == "vue" {
                        if let Some(index) = component_index {
                            for comp_name in extract_vue_component_tags(&content) {
                                if let Some(comp_path) = index.get(&comp_name) {
                                    process_file(comp_path, current_depth + 1, max_depth, visited, result_blocks, parsed_paths, base_path,
                                        ignore_names, ignore_extensions, ignore_filenames, ignore_regexes,
                                        ignore_deep_names, ignore_deep_extensions, ignore_deep_filenames, ignore_deep_regexes,
                                        included_types, enable_minimization, minimization_threshold, minimization_depth_threshold, current_total_size, abort_handle, parse_cache, component_index);
                                }
                            }
                        }
                    }
                }
            }
            return;
        }
    }

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

        parsed_paths.push(display_path_str.clone());

        let mut final_content = content.clone();
        if enable_minimization && (*current_total_size + content.len() > minimization_threshold) && current_depth >= minimization_depth_threshold {
            // Only minimize for JS/TS/Rust/Go/Java/C++ etc. (bracket-based languages)
            let ext = abs_path.extension().and_then(|e| e.to_str()).unwrap_or("");
            match ext {
                e if C_STYLE_COMMENT_EXTS.contains(&e) => {
                    final_content = minimizer::minimize_code(&content);
                }
                _ => {}
            }
        }

        let formatted_content = format!(
            "========================================\n[FILE PATH]: {}\n(Dependency Layer: {})\n========================================\n[CONTENT START]\n{}\n[CONTENT END]",
            display_path_str, current_depth, final_content
        );

        *current_total_size += formatted_content.len();

        // 写入缓存（仅当能获取到 mtime 时）
        if let Some(t) = mtime {
            parse_cache.set(abs_path.clone(), t, display_path_str.clone(), formatted_content.clone());
        }

        result_blocks.push(FileNode {
            path: display_path_str.clone(),
            content: formatted_content,
            abs_path: abs_path.to_string_lossy().into_owned(),
        });
        
        let ext = abs_path.extension().and_then(|e| e.to_str()).unwrap_or("");
        
        if !should_ignore(&abs_path, ignore_deep_names, ignore_deep_extensions, ignore_deep_filenames, ignore_deep_regexes) {
            let base_dir = abs_path.parent().unwrap_or(Path::new(""));
            for dep in extract_dependencies(&content, ext) {
                if let Some(resolved) = resolve_path(base_dir, &dep, ext, base_path) {
                    process_file(&resolved, current_depth + 1, max_depth, visited, result_blocks, parsed_paths, base_path, 
                        ignore_names, ignore_extensions, ignore_filenames, ignore_regexes,
                        ignore_deep_names, ignore_deep_extensions, ignore_deep_filenames, ignore_deep_regexes,
                        included_types, enable_minimization, minimization_threshold, minimization_depth_threshold, current_total_size, abort_handle, parse_cache, component_index);
                }
            }
            // Vue 组件自动引入：通过索引解析模板中未显式 import 的组件
            if ext == "vue" {
                if let Some(index) = component_index {
                    for comp_name in extract_vue_component_tags(&content) {
                        if let Some(comp_path) = index.get(&comp_name) {
                            process_file(comp_path, current_depth + 1, max_depth, visited, result_blocks, parsed_paths, base_path,
                                ignore_names, ignore_extensions, ignore_filenames, ignore_regexes,
                                ignore_deep_names, ignore_deep_extensions, ignore_deep_filenames, ignore_deep_regexes,
                                included_types, enable_minimization, minimization_threshold, minimization_depth_threshold, current_total_size, abort_handle, parse_cache, component_index);
                        }
                    }
                }
            }
        }
    }
}
