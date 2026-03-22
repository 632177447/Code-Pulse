// 路径解析模块：处理跨语言的依赖路径转换与项目根目录识别

use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use super::constants::*;
use super::ignore::should_ignore;

pub fn resolve_path(base_dir: &Path, import_path: &str, ext: &str, project_root: &Path) -> Option<PathBuf> {
    // 忽略网络路径
    if import_path.starts_with("http://") || import_path.starts_with("https://") || import_path.starts_with("//") {
        return None;
    }

    let extensions = match ext {
        e if JS_TS_FAMILY.contains(&e) => JS_TS_FAMILY.to_vec(),
        "py" => vec!["py"],
        "rs" => vec!["rs"],
        "go" => vec!["go"],
        e if JAVA_KT_FAMILY.contains(&e) => JAVA_KT_FAMILY.to_vec(),
        e if C_CPP_FAMILY.contains(&e) => C_CPP_FAMILY.to_vec(),
        "cs" => vec!["cs"],
        "php" => vec!["php"],
        "rb" => vec!["rb"],
        e if STYLE_FAMILY.contains(&e) => STYLE_FAMILY.to_vec(),
        "html" => HTML_RESOLVE_EXTS.to_vec(),
        "md" => MD_RESOLVE_EXTS.to_vec(),
        _ => vec![],
    };

    let check_target = |t: &Path| -> Option<PathBuf> {
        if t.exists() && t.is_file() {
            return Some(t.to_path_buf());
        }
        for e in &extensions {
            let with_ext = t.with_extension(e);
            if with_ext.exists() {
                return Some(with_ext);
            }
        }
        
        if t.is_dir() {
            for e in &extensions {
                let index_path = t.join(format!("index.{}", e));
                if index_path.exists() {
                    return Some(index_path);
                }
            }
        }
        None
    };

    if import_path.starts_with("crate/") {
        check_target(&project_root.join("src").join(&import_path[6..]))
    } else if import_path.starts_with("@/") {
        check_target(&project_root.join("src").join(&import_path[2..]))
    } else if import_path.starts_with("~/") {
        check_target(&project_root.join(&import_path[2..]))
    } else if import_path.starts_with("/") {
        check_target(&project_root.join(&import_path[1..]))
    } else if import_path.starts_with(".") {
        check_target(&base_dir.join(import_path))
    } else {
        if let Some(res) = check_target(&base_dir.join(import_path)) {
            Some(res)
        } else if let Some(res) = check_target(&project_root.join(import_path)) {
            Some(res)
        } else {
            check_target(&project_root.join("src").join(import_path))
        }
    }
}

/// 检测项目根目录的 package.json 中是否安装了 Vue 自动引入组件插件
pub fn detect_auto_import_plugin(root: &Path) -> bool {
    let pkg_path = root.join("package.json");
    if let Ok(content) = fs::read_to_string(&pkg_path) {
        return content.contains("unplugin-vue-components")
            || content.contains("vite-plugin-components")
            || content.contains("@vite-plugin-components");
    }
    false
}

/// 扫描项目根目录（排除忽略目录），构建组件名 → 路径的索引
/// key 为 PascalCase 文件名（不含扩展名），val 为文件路径
pub fn build_component_index(
    root: &Path,
    ignore_names: &HashSet<String>,
    ignore_extensions: &HashSet<String>,
    ignore_filenames: &HashSet<String>,
    ignore_regexes: &[Regex],
) -> HashMap<String, PathBuf> {
    let mut index = HashMap::new();
    for entry in WalkDir::new(root)
        .into_iter()
        // 目录级别剪枝，避免深入 node_modules 等
        .filter_entry(|e| {
            if e.file_type().is_dir() {
                !should_ignore(e.path(), ignore_names, ignore_extensions, ignore_filenames, ignore_regexes)
            } else {
                true
            }
        })
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
        // 只索引 .vue 文件（自动引入插件的目标）
        if ext != "vue" { continue; }
        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
            // 文件名本身已是 PascalCase 时直接存入，忽略 index.vue 等特殊文件
            if stem == "index" || stem == "Index" { continue; }
            index.entry(stem.to_string()).or_insert_with(|| path.to_path_buf());
        }
    }
    index
}

pub fn find_project_root(start_path: &Path, manual_roots: &[PathBuf]) -> PathBuf {
    // 1. 如果用户手动指定了根目录，检查当前路径是否在其中之一的子树下
    for mr in manual_roots {
        if let (Ok(abs_start), Ok(abs_mr)) = (start_path.canonicalize(), mr.canonicalize()) {
            if abs_start.starts_with(&abs_mr) {
                return abs_mr;
            }
        } else if start_path.starts_with(mr) {
            return mr.to_path_buf();
        }
    }

    // 2. 增加对多种编程语言和构建工具根目录标识文件的支持，确保在不同类型的项目中都能准确识别根节点
    let mut current = start_path;
    loop {
        for marker in PROJECT_ROOT_MARKERS {
            if current.join(marker).exists() {
                // 找到标识文件后尝试规范化路径，确保后续相对路径解析（如 @/ 或 crate/）的基准一致
                if let Ok(canon) = current.canonicalize() {
                    return canon;
                }
                return current.to_path_buf();
            }
        }

        if let Some(parent) = current.parent() {
            current = parent;
        } else {
            break;
        }
    }
    
    // 如果递归到根部仍未找到标识，回退到当前目录或文件所在目录，并尝试获取其绝对路径
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
