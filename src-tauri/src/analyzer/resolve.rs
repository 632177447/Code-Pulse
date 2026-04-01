// 路径解析模块：处理跨语言的依赖路径转换与项目根目录识别

use regex::Regex;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use walkdir::WalkDir;

use super::constants::*;
use super::ignore::should_ignore;
use super::regex::strip_comments;

static PATH_ALIAS_CACHE: OnceLock<Mutex<HashMap<PathBuf, Vec<PathAlias>>>> = OnceLock::new();

#[derive(Clone, Debug)]
struct PathAlias {
    pattern: String,
    base_dir: PathBuf,
    targets: Vec<String>,
}

fn get_path_alias_cache() -> &'static Mutex<HashMap<PathBuf, Vec<PathAlias>>> {
    PATH_ALIAS_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn sanitize_import_path(import_path: &str) -> &str {
    let trimmed = import_path.trim();
    let end = trimmed.find(|c| c == '?' || c == '#').unwrap_or(trimmed.len());
    &trimmed[..end]
}

fn append_extension(path: &Path, ext: &str) -> PathBuf {
    let mut os = OsString::from(path.as_os_str());
    os.push(".");
    os.push(ext);
    PathBuf::from(os)
}

fn strip_json_trailing_commas(content: &str) -> String {
    let mut result = String::with_capacity(content.len());
    let mut chars = content.chars().peekable();
    let mut in_string = false;
    let mut escape = false;

    while let Some(ch) = chars.next() {
        if in_string {
            result.push(ch);
            if escape {
                escape = false;
            } else if ch == '\\' {
                escape = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }

        match ch {
            '"' => {
                in_string = true;
                result.push(ch);
            }
            ',' => {
                let mut lookahead = chars.clone();
                while let Some(next) = lookahead.next() {
                    if next.is_whitespace() {
                        continue;
                    }
                    if next == '}' || next == ']' {
                        break;
                    }
                    result.push(ch);
                    break;
                }
            }
            _ => result.push(ch),
        }
    }

    result
}

fn parse_jsonc(content: &str) -> Option<Value> {
    let stripped = strip_comments(content, "ts");
    let normalized = strip_json_trailing_commas(&stripped);
    serde_json::from_str(&normalized).ok()
}

fn merge_json_values(base: Value, overlay: Value) -> Value {
    match (base, overlay) {
        (Value::Object(mut base_map), Value::Object(overlay_map)) => {
            for (key, value) in overlay_map {
                let merged = match base_map.remove(&key) {
                    Some(base_value) => merge_json_values(base_value, value),
                    None => value,
                };
                base_map.insert(key, merged);
            }
            Value::Object(base_map)
        }
        (_, overlay) => overlay,
    }
}

fn resolve_extended_config_path(config_path: &Path, extends_value: &str) -> Option<PathBuf> {
    if extends_value.is_empty() {
        return None;
    }

    let config_dir = config_path.parent().unwrap_or(Path::new(""));
    let raw_path = PathBuf::from(extends_value);
    let candidate = if raw_path.is_absolute() {
        raw_path
    } else {
        config_dir.join(raw_path)
    };

    if candidate.extension().is_some() {
        if candidate.exists() {
            return Some(candidate);
        }
    } else {
        let json_candidate = candidate.with_extension("json");
        if json_candidate.exists() {
            return Some(json_candidate);
        }
        let nested_candidate = candidate.join("tsconfig.json");
        if nested_candidate.exists() {
            return Some(nested_candidate);
        }
    }

    None
}

fn load_config_value(config_path: &Path, visited: &mut HashSet<PathBuf>) -> Option<Value> {
    let cache_key = config_path.canonicalize().unwrap_or_else(|_| config_path.to_path_buf());
    if !visited.insert(cache_key) {
        return None;
    }

    let content = fs::read_to_string(config_path).ok()?;
    let current = parse_jsonc(&content)?;

    let extends_value = current
        .get("extends")
        .and_then(|value| value.as_str())
        .map(|value| value.to_string());

    if let Some(extends_value) = extends_value {
        if let Some(parent_path) = resolve_extended_config_path(config_path, &extends_value) {
            if let Some(parent_value) = load_config_value(&parent_path, visited) {
                return Some(merge_json_values(parent_value, current));
            }
        }
    }

    Some(current)
}

fn match_alias_pattern(pattern: &str, import_path: &str) -> Option<String> {
    if let Some((prefix, suffix)) = pattern.split_once('*') {
        if import_path.starts_with(prefix)
            && import_path.ends_with(suffix)
            && import_path.len() >= prefix.len() + suffix.len()
        {
            return Some(import_path[prefix.len()..import_path.len() - suffix.len()].to_string());
        }
        return None;
    }

    if import_path == pattern {
        Some(String::new())
    } else {
        None
    }
}

fn apply_alias_target_pattern(pattern: &str, matched: &str) -> String {
    if let Some((prefix, suffix)) = pattern.split_once('*') {
        format!("{}{}{}", prefix, matched, suffix)
    } else {
        pattern.to_string()
    }
}

fn load_path_aliases(project_root: &Path) -> Vec<PathAlias> {
    let cache_key = project_root.canonicalize().unwrap_or_else(|_| project_root.to_path_buf());

    if let Ok(cache) = get_path_alias_cache().lock() {
        if let Some(aliases) = cache.get(&cache_key) {
            return aliases.clone();
        }
    }

    let mut aliases = Vec::new();

    for config_name in ["tsconfig.json", "jsconfig.json"] {
        let config_path = project_root.join(config_name);
        if !config_path.exists() {
            continue;
        }

        let mut visited = HashSet::new();
        let Some(config_value) = load_config_value(&config_path, &mut visited) else {
            continue;
        };

        let compiler_options = config_value
            .get("compilerOptions")
            .and_then(|value| value.as_object());
        let base_url = compiler_options
            .and_then(|options| options.get("baseUrl"))
            .and_then(|value| value.as_str())
            .unwrap_or(".");
        let base_dir = config_path
            .parent()
            .unwrap_or(Path::new(""))
            .join(base_url);

        let Some(paths) = compiler_options
            .and_then(|options| options.get("paths"))
            .and_then(|value| value.as_object())
        else {
            continue;
        };

        for (pattern, target_value) in paths {
            let Some(targets) = target_value.as_array() else {
                continue;
            };

            let resolved_targets: Vec<String> = targets
                .iter()
                .filter_map(|item| item.as_str().map(|value| value.to_string()))
                .collect();

            if resolved_targets.is_empty() {
                continue;
            }

            aliases.push(PathAlias {
                pattern: pattern.to_string(),
                base_dir: base_dir.clone(),
                targets: resolved_targets,
            });
        }
    }

    if let Ok(mut cache) = get_path_alias_cache().lock() {
        cache.insert(cache_key, aliases.clone());
    }

    aliases
}

pub fn resolve_path(base_dir: &Path, import_path: &str, ext: &str, project_root: &Path) -> Option<PathBuf> {
    let import_path = sanitize_import_path(import_path);
    if import_path.is_empty() {
        return None;
    }

    // 忽略网络路径
    if import_path.starts_with("http://") || import_path.starts_with("https://") || import_path.starts_with("//") {
        return None;
    }

    // 忽略各语言标准库或内置模块
    let is_std = match ext {
        "rs" => RS_STD_LIBS.iter().any(|&lib| import_path == lib || import_path.starts_with(&format!("{}/", lib))),
        "py" => PY_STD_LIBS.iter().any(|&lib| import_path == lib || import_path.starts_with(&format!("{}/", lib))),
        "go" => GO_STD_LIBS.iter().any(|&lib| import_path == lib || import_path.starts_with(&format!("{}/", lib))),
        e if JAVA_KT_FAMILY.contains(&e) => JAVA_STD_LIBS.iter().any(|&lib| import_path == lib || import_path.starts_with(&format!("{}/", lib))),
        e if JS_TS_FAMILY.contains(&e) => {
            import_path.starts_with("node:") || NODE_STD_LIBS.iter().any(|&lib| import_path == lib || import_path.starts_with(&format!("{}/", lib)))
        }
        _ => false,
    };
    if is_std {
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

        let has_non_standard_suffix = t
            .extension()
            .and_then(|e| e.to_str())
            .map(|suffix| !extensions.contains(&suffix))
            .unwrap_or(false);

        for e in &extensions {
            if has_non_standard_suffix {
                let appended_ext = append_extension(t, e);
                if appended_ext.exists() {
                    return Some(appended_ext);
                }
            }

            let with_ext = t.with_extension(e);
            if with_ext.exists() {
                return Some(with_ext);
            }
        }
        
        if t.is_dir() {
            if ext == "py" {
                let init_path = t.join("__init__.py");
                if init_path.exists() {
                    return Some(init_path);
                }
            }

            if ext == "rs" {
                let mod_path = t.join("mod.rs");
                if mod_path.exists() {
                    return Some(mod_path);
                }
            }

            for e in &extensions {
                let index_path = t.join(format!("index.{}", e));
                if index_path.exists() {
                    return Some(index_path);
                }
            }
        }
        None
    };

    let resolve_from_aliases = || -> Option<PathBuf> {
        for alias in load_path_aliases(project_root) {
            let Some(matched) = match_alias_pattern(&alias.pattern, import_path) else {
                continue;
            };

            for target_pattern in &alias.targets {
                let candidate = alias
                    .base_dir
                    .join(apply_alias_target_pattern(target_pattern, &matched));
                if let Some(resolved) = check_target(&candidate) {
                    return Some(resolved);
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
        if let Some(res) = resolve_from_aliases() {
            Some(res)
        } else if let Some(res) = check_target(&base_dir.join(import_path)) {
            Some(res)
        } else if let Some(res) = check_target(&project_root.join(import_path)) {
            Some(res)
        } else {
            check_target(&project_root.join("src").join(import_path))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::resolve_path;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn create_test_root(prefix: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("codepulse-{}-{}", prefix, stamp));
        fs::create_dir_all(root.join("src").join("modules")).unwrap();
        fs::write(root.join("package.json"), "{}").unwrap();
        root
    }

    fn write_tsconfig(root: &PathBuf, content: &str) {
        fs::write(root.join("tsconfig.json"), content).unwrap();
    }

    #[test]
    fn resolve_path_should_prefer_appending_extension_for_dotted_ts_stem() {
        let root = create_test_root("resolve-dotted");
        let base_dir = root.join("src").join("modules");
        let fallback = base_dir.join("dingtalk.ts");
        let target = base_dir.join("dingtalk.controller.ts");
        fs::write(&fallback, "export {};").unwrap();
        fs::write(&target, "export {};").unwrap();

        let resolved = resolve_path(&base_dir, "./dingtalk.controller", "ts", &root);

        assert_eq!(resolved, Some(target.clone()));

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn resolve_path_should_strip_query_before_resolving_ts_file() {
        let root = create_test_root("resolve-query");
        let base_dir = root.join("src").join("modules");
        let target = base_dir.join("lazy.service.ts");
        fs::write(&target, "export {};").unwrap();

        let resolved = resolve_path(&base_dir, "./lazy.service?raw", "ts", &root);

        assert_eq!(resolved, Some(target.clone()));

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn resolve_path_should_support_python_package_init_file() {
        let root = create_test_root("resolve-python-init");
        let base_dir = root.join("src").join("modules");
        let package_dir = base_dir.join("services");
        fs::create_dir_all(&package_dir).unwrap();
        let target = package_dir.join("__init__.py");
        fs::write(&target, "").unwrap();

        let resolved = resolve_path(&base_dir, "./services", "py", &root);

        assert_eq!(resolved, Some(target.clone()));

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn resolve_path_should_support_rust_mod_rs() {
        let root = create_test_root("resolve-rust-mod");
        let base_dir = root.join("src").join("modules");
        let module_dir = base_dir.join("parser");
        fs::create_dir_all(&module_dir).unwrap();
        let target = module_dir.join("mod.rs");
        fs::write(&target, "").unwrap();

        let resolved = resolve_path(&base_dir, "./parser", "rs", &root);

        assert_eq!(resolved, Some(target.clone()));

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn resolve_path_should_support_tsconfig_path_alias_for_vue_imports() {
        let root = create_test_root("resolve-tsconfig-alias");
        let base_dir = root.join("src").join("pages");
        fs::create_dir_all(&base_dir).unwrap();
        fs::create_dir_all(root.join("src").join("renderer").join("stores")).unwrap();

        write_tsconfig(
            &root,
            r#"
{
  "compilerOptions": {
    "baseUrl": ".",
    "paths": {
      "@renderer/*": ["src/renderer/*"]
    }
  }
}
"#,
        );

        let target = root.join("src").join("renderer").join("stores").join("index.ts");
        fs::write(&target, "export {};").unwrap();

        let resolved = resolve_path(&base_dir, "@renderer/stores", "vue", &root);

        assert_eq!(resolved, Some(target.clone()));

        let _ = fs::remove_dir_all(&root);
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
