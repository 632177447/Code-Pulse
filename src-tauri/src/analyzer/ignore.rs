// 忽略规则模块：处理分析过程中的文件/目录忽略逻辑

use regex::Regex;
use std::collections::HashSet;
use std::path::Path;

pub fn parse_ignore_patterns(raw: &str, defaults: &[&str]) -> (HashSet<String>, HashSet<String>, HashSet<String>, Vec<Regex>) {
    let mut names = HashSet::new();
    let mut exts = HashSet::new();
    let mut fnames = HashSet::new();
    let mut regexes = Vec::new();

    let mut all_patterns: Vec<String> = defaults.iter().map(|&s| s.to_string()).collect();
    if !raw.is_empty() {
        for p in raw.split(|c| c == ',' || c == '\n' || c == '\r') {
            let s = p.trim().to_string();
            if !s.is_empty() {
                all_patterns.push(s);
            }
        }
    }

    for s in all_patterns {
        if s.contains('*') {
            let mut escaped = regex::escape(&s);
            escaped = escaped.replace("\\*", ".*");
            let pattern = format!("^{}$", escaped);
            if let Ok(re) = Regex::new(&pattern) {
                regexes.push(re);
            }
        } else if s.starts_with('.') {
            exts.insert(s.to_lowercase());
        } else if s.contains('.') {
            fnames.insert(s);
        } else {
            names.insert(s);
        }
    }
    (names, exts, fnames, regexes)
}

pub fn should_ignore(
    path: &Path, 
    ignore_names: &HashSet<String>, 
    ignore_extensions: &HashSet<String>, 
    ignore_filenames: &HashSet<String>,
    ignore_regexes: &[Regex]
) -> bool {
    let fname = path.file_name().and_then(|f| f.to_str()).unwrap_or("");
    let fname_lower = fname.to_lowercase();

    // 1. Check dot-prefixed patterns (suffix match)
    for ext in ignore_extensions {
        if fname_lower.ends_with(ext) {
            return true;
        }
    }

    // 2. Check full filename match
    if ignore_filenames.contains(fname) {
        return true;
    }

    // 3. Check regexes against filename
    for re in ignore_regexes {
        if re.is_match(fname) {
            return true;
        }
    }

    // 4. Check each component for patterns (directory/file match)
    for component in path.components() {
        if let Some(comp_str) = component.as_os_str().to_str() {
            if ignore_names.contains(comp_str) {
                return true;
            }
            for re in ignore_regexes {
                if re.is_match(comp_str) {
                    return true;
                }
            }
        }
    }

    false
}
