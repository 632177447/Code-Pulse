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

struct CollectedFile {
    path: String,
    raw_content: String,
    minimized_content: Option<String>,
    abs_path: PathBuf,
    direct_dependencies: Vec<PathBuf>,
    depth: usize,
    ext: String,
    mtime: Option<std::time::SystemTime>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileNode {
    pub path: String,
    pub content: String,
    pub abs_path: String,
    pub depth: usize,
    pub dependencies: Vec<String>,
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
    let mut result_blocks: Vec<CollectedFile> = Vec::new();

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

    // 预收集所有“根”文件路径，用于在处理依赖时判断其是否属于用户直接选中的文件集（如果是则深度设为 0）
    let mut all_root_files: HashSet<PathBuf> = HashSet::new();
    for p_str in &paths {
        let p = Path::new(p_str);
        if !p.exists() { continue; }
        if p.is_dir() {
            for entry in WalkDir::new(p)
                .into_iter()
                .filter_entry(|e| {
                    if e.file_type().is_dir() {
                        !should_ignore(e.path(), &ignore_names, &ignore_extensions, &ignore_filenames, &ignore_regexes)
                    } else {
                        true
                    }
                })
                .filter_map(|e| e.ok()) 
            {
                let e_path = entry.path();
                if e_path.is_file() {
                    let ext = e_path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
                    if included_types_set.contains(&ext) {
                        if !should_ignore(e_path, &ignore_names, &ignore_extensions, &ignore_filenames, &ignore_regexes) {
                            if let Ok(abs) = e_path.canonicalize() {
                                all_root_files.insert(abs);
                            }
                        }
                    }
                }
            }
        } else if p.is_file() {
            if let Ok(abs) = p.canonicalize() {
                all_root_files.insert(abs);
            }
        }
    }

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
            for entry in WalkDir::new(path)
                .into_iter()
                .filter_entry(|e| {
                    if e.file_type().is_dir() {
                        !should_ignore(e.path(), &ignore_names, &ignore_extensions, &ignore_filenames, &ignore_regexes)
                    } else {
                        true
                    }
                })
                .filter_map(|e| e.ok()) 
            {
                if let Some(ref h) = abort_handle {
                    if h.load(Ordering::SeqCst) {
                        return Ok(finalize_result(
                            result_blocks,
                            enable_minimization,
                            minimization_threshold,
                            minimization_depth_threshold,
                            &parse_cache,
                        ));
                    }
                }
                let e_path = entry.path();
                if e_path.is_file() {
                    let ext = e_path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
                    if included_types_set.contains(&ext) {
                        if should_ignore(e_path, &ignore_names, &ignore_extensions, &ignore_filenames, &ignore_regexes) {
                            continue;
                        }
                        // 初始深度统一设为 0
                        process_file(e_path, 0, max_depth, &mut visited, &mut result_blocks, &base_path, 
                            &ignore_names, &ignore_extensions, &ignore_filenames, &ignore_regexes,
                            &ignore_deep_names, &ignore_deep_extensions, &ignore_deep_filenames, &ignore_deep_regexes,
                            &included_types_set, abort_handle.as_ref(), &parse_cache, comp_index, &all_root_files);
                    }
                }
            }
        } else {
            process_file(path, 0, max_depth, &mut visited, &mut result_blocks, &base_path, 
                &ignore_names, &ignore_extensions, &ignore_filenames, &ignore_regexes,
                &ignore_deep_names, &ignore_deep_extensions, &ignore_deep_filenames, &ignore_deep_regexes,
                &included_types_set, abort_handle.as_ref(), &parse_cache, comp_index, &all_root_files);
        }
    }

    Ok(finalize_result(
        result_blocks,
        enable_minimization,
        minimization_threshold,
        minimization_depth_threshold,
        &parse_cache,
    ))
}

fn process_file(
    path: &Path, 
    current_depth: usize, 
    max_depth: usize, 
    visited: &mut HashSet<PathBuf>, 
    result_blocks: &mut Vec<CollectedFile>,
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
    abort_handle: Option<&Arc<AtomicBool>>,
    parse_cache: &crate::cache::FileCache,
    component_index: Option<&HashMap<String, PathBuf>>,
    all_root_files: &HashSet<PathBuf>,
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
            let display_path_str = build_display_path(&abs_path, base_path);
            result_blocks.push(CollectedFile {
                path: display_path_str,
                raw_content: entry.raw_content.clone(),
                minimized_content: entry.minimized_content.clone(),
                abs_path: abs_path.clone(),
                direct_dependencies: Vec::new(),
                depth: current_depth,
                ext: file_ext.clone(),
                mtime: Some(t),
            });
            let result_index = result_blocks.len() - 1;

            // 缓存命中时仍需追踪依赖
            let content = entry.raw_content;
            let ext = abs_path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if !should_ignore(&abs_path, ignore_deep_names, ignore_deep_extensions, ignore_deep_filenames, ignore_deep_regexes) {
                let base_dir = abs_path.parent().unwrap_or(Path::new(""));
                let mut direct_dependencies = Vec::new();
                let mut direct_dependency_set = HashSet::new();
                for dep in extract_dependencies(&content, ext) {
                    if let Some(resolved) = resolve_path(base_dir, &dep, ext, base_path) {
                        if direct_dependency_set.insert(resolved.clone()) {
                            direct_dependencies.push(resolved.clone());
                        }
                        // 如果依赖项属于根文件集，深度保持为 0，否则才 +1
                        let next_depth = if all_root_files.contains(&resolved) { 0 } else { current_depth + 1 };
                        process_file(&resolved, next_depth, max_depth, visited, result_blocks, base_path,
                            ignore_names, ignore_extensions, ignore_filenames, ignore_regexes,
                            ignore_deep_names, ignore_deep_extensions, ignore_deep_filenames, ignore_deep_regexes,
                            included_types, abort_handle, parse_cache, component_index, all_root_files);
                    }
                }
                // Vue 组件自动引入：通过索引解析模板中未显式 import 的组件
                if ext == "vue" {
                    if let Some(index) = component_index {
                        for comp_name in extract_vue_component_tags(&content) {
                            if let Some(comp_path) = index.get(&comp_name) {
                                if direct_dependency_set.insert(comp_path.clone()) {
                                    direct_dependencies.push(comp_path.clone());
                                }
                                let next_depth = if all_root_files.contains(comp_path) { 0 } else { current_depth + 1 };
                                process_file(comp_path, next_depth, max_depth, visited, result_blocks, base_path,
                                    ignore_names, ignore_extensions, ignore_filenames, ignore_regexes,
                                    ignore_deep_names, ignore_deep_extensions, ignore_deep_filenames, ignore_deep_regexes,
                                    included_types, abort_handle, parse_cache, component_index, all_root_files);
                            }
                        }
                    }
                }
                result_blocks[result_index].direct_dependencies = direct_dependencies;
            }
            return;
        }
    }

    if let Ok(content) = fs::read_to_string(&abs_path) {
        let display_path_str = build_display_path(&abs_path, base_path);

        // 写入缓存（仅当能获取到 mtime 时）
        if let Some(t) = mtime {
            parse_cache.set(abs_path.clone(), t, content.clone());
        }

        result_blocks.push(CollectedFile {
            path: display_path_str.clone(),
            raw_content: content.clone(),
            minimized_content: None,
            abs_path: abs_path.clone(),
            direct_dependencies: Vec::new(),
            depth: current_depth,
            ext: file_ext.clone(),
            mtime,
        });
        let result_index = result_blocks.len() - 1;
        
        let ext = abs_path.extension().and_then(|e| e.to_str()).unwrap_or("");
        
        if !should_ignore(&abs_path, ignore_deep_names, ignore_deep_extensions, ignore_deep_filenames, ignore_deep_regexes) {
            let base_dir = abs_path.parent().unwrap_or(Path::new(""));
            let mut direct_dependencies = Vec::new();
            let mut direct_dependency_set = HashSet::new();
            for dep in extract_dependencies(&content, ext) {
                if let Some(resolved) = resolve_path(base_dir, &dep, ext, base_path) {
                    if direct_dependency_set.insert(resolved.clone()) {
                        direct_dependencies.push(resolved.clone());
                    }
                    let next_depth = if all_root_files.contains(&resolved) { 0 } else { current_depth + 1 };
                    process_file(&resolved, next_depth, max_depth, visited, result_blocks, base_path, 
                        ignore_names, ignore_extensions, ignore_filenames, ignore_regexes,
                        ignore_deep_names, ignore_deep_extensions, ignore_deep_filenames, ignore_deep_regexes,
                        included_types, abort_handle, parse_cache, component_index, all_root_files);
                }
            }
            // Vue 组件自动引入：通过索引解析模板中未显式 import 的组件
            if ext == "vue" {
                if let Some(index) = component_index {
                    for comp_name in extract_vue_component_tags(&content) {
                        if let Some(comp_path) = index.get(&comp_name) {
                            if direct_dependency_set.insert(comp_path.clone()) {
                                direct_dependencies.push(comp_path.clone());
                            }
                            let next_depth = if all_root_files.contains(comp_path) { 0 } else { current_depth + 1 };
                            process_file(comp_path, next_depth, max_depth, visited, result_blocks, base_path,
                                ignore_names, ignore_extensions, ignore_filenames, ignore_regexes,
                                ignore_deep_names, ignore_deep_extensions, ignore_deep_filenames, ignore_deep_regexes,
                                included_types, abort_handle, parse_cache, component_index, all_root_files);
                        }
                    }
                }
            }
            result_blocks[result_index].direct_dependencies = direct_dependencies;
        }
    }
}

fn build_display_path(abs_path: &Path, base_path: &Path) -> String {
    let mut display_path_str = abs_path.to_string_lossy().into_owned();
    if let Ok(rel_path) = abs_path.strip_prefix(base_path) {
        display_path_str = rel_path.to_string_lossy().replace("\\", "/");
    } else {
        display_path_str = display_path_str.replace("\\", "/");
        if display_path_str.starts_with("//?/") {
            display_path_str = display_path_str[4..].to_string();
        }
    }
    display_path_str
}

fn format_file_content(path: &str, depth: usize, content: &str) -> String {
    format!(
        "========================================\n[FILE PATH]: {}\n(Dependency Layer: {})\n========================================\n[CONTENT START]\n{}\n[CONTENT END]",
        path, depth, content
    )
}

fn finalize_result(
    result_blocks: Vec<CollectedFile>,
    enable_minimization: bool,
    minimization_threshold: usize,
    minimization_depth_threshold: usize,
    parse_cache: &crate::cache::FileCache,
) -> Vec<FileNode> {
    let display_path_map: HashMap<PathBuf, String> = result_blocks
        .iter()
        .map(|file| (file.abs_path.clone(), file.path.clone()))
        .collect();
    let total_unminimized_size: usize = result_blocks
        .iter()
        .map(|file| format_file_content(&file.path, file.depth, &file.raw_content).len())
        .sum();
    let should_minimize = enable_minimization && total_unminimized_size >= minimization_threshold;

    result_blocks.into_iter().map(|mut file| {
        let final_content = if should_minimize
            && file.depth >= minimization_depth_threshold
            && (C_STYLE_COMMENT_EXTS.contains(&file.ext.as_str()) || MIXED_STYLE_COMMENT_EXTS.contains(&file.ext.as_str()))
        {
            if let Some(cached) = file.minimized_content.take() {
                cached
            } else {
                let minimized = match file.ext.as_str() {
                    ext if MIXED_STYLE_COMMENT_EXTS.contains(&ext) => minimizer::minimize_mixed_code(&file.raw_content),
                    _ => minimizer::minimize_code(&file.raw_content),
                };
                if let Some(mtime) = file.mtime {
                    parse_cache.set_minimized(&file.abs_path, mtime, minimized.clone());
                }
                minimized
            }
        } else {
            file.raw_content
        };

        FileNode {
            path: file.path.clone(),
            content: format_file_content(&file.path, file.depth, &final_content),
            abs_path: file.abs_path.to_string_lossy().into_owned(),
            depth: file.depth,
            dependencies: file.direct_dependencies
                .into_iter()
                .filter_map(|dependency| display_path_map.get(&dependency).cloned())
                .collect(),
        }
    }).collect()
}
