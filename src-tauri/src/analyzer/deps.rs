// 依赖提取模块：从不同编程语言的文件内容中解析并提取依赖项

use std::collections::HashSet;

use super::constants::*;
use super::regex::*;

pub fn kebab_to_pascal(s: &str) -> String {
    s.split('-')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect()
}

pub fn extract_dependencies(content: &str, ext: &str) -> Vec<String> {
    let mut deps = Vec::new();
    let content_stripped = strip_comments(content, ext);
    let content_lf = content_stripped.replace("\r\n", "\n");
    match ext {
        e if JS_TS_FAMILY.contains(&e) => {
            let re = get_js_re();
            for cap in re.captures_iter(&content_lf) {
                if let Some(m) = cap.get(1).or(cap.get(2)).or(cap.get(3)).or(cap.get(4)).or(cap.get(5)) {
                    deps.push(m.as_str().to_string());
                }
            }
        }
        "py" => {
            let re = get_py_re();
            for cap in re.captures_iter(&content_lf) {
                if let Some(m) = cap.get(1) {
                    let mut s = m.as_str().to_string();
                    if s.starts_with(".") {
                        let count = s.chars().take_while(|&c| c == '.').count();
                        let prefix = if count == 1 { "./".to_string() } else { "../".repeat(count - 1) };
                        s = format!("{}{}", prefix, s[count..].replace('.', "/"));
                    } else {
                        s = s.replace('.', "/");
                    }
                    deps.push(s);
                }
            }
        }
        "rs" => {
            let re = get_rs_re();
            for cap in re.captures_iter(&content_lf) {
                if let Some(m) = cap.get(1) {
                    let mut s = m.as_str().replace("::", "/");
                    if s.starts_with("super/") {
                        s = s.replacen("super/", "../", 1);
                    } else if s.starts_with("self/") {
                        s = s.replacen("self/", "./", 1);
                    }
                    deps.push(s);
                }
            }
        }
        "go" => {
            let re = get_go_re();
            let str_re = get_str_re();
            for cap in re.captures_iter(&content_lf) {
                if let Some(block) = cap.get(1) {
                    for scap in str_re.captures_iter(block.as_str()) {
                        deps.push(scap.get(1).unwrap().as_str().to_string());
                    }
                } else if let Some(m) = cap.get(2) {
                    deps.push(m.as_str().to_string());
                }
            }
        }
        e if JAVA_KT_FAMILY.contains(&e) => {
            let re = get_java_re();
            for cap in re.captures_iter(&content_lf) {
                if let Some(m) = cap.get(1) {
                    deps.push(m.as_str().replace('.', "/"));
                }
            }
        }
        e if C_CPP_FAMILY.contains(&e) => {
            let re = get_cpp_re();
            for cap in re.captures_iter(&content_lf) {
                if let Some(m) = cap.get(1) {
                    deps.push(m.as_str().to_string());
                }
            }
        }
        "cs" => {
            let re = get_cs_re();
            for cap in re.captures_iter(&content_lf) {
                if let Some(m) = cap.get(1) {
                    deps.push(m.as_str().replace('.', "/"));
                }
            }
        }
        "php" => {
            let re = get_php_re();
            for cap in re.captures_iter(&content_lf) {
                if let Some(m) = cap.get(1).or(cap.get(2)) {
                    deps.push(m.as_str().replace('\\', "/"));
                }
            }
        }
        "rb" => {
            let re = get_rb_re();
            for cap in re.captures_iter(&content_lf) {
                if let Some(m) = cap.get(1) {
                    deps.push(m.as_str().to_string());
                }
            }
        }
        e if STYLE_FAMILY.contains(&e) => {
            let re = get_css_re();
            for cap in re.captures_iter(&content_lf) {
                if let Some(m) = cap.get(1).or(cap.get(2)) {
                    deps.push(m.as_str().to_string());
                }
            }
        }
        "html" => {
            let re = get_html_re();
            for cap in re.captures_iter(&content_lf) {
                if let Some(m) = cap.get(1) {
                    deps.push(m.as_str().to_string());
                }
            }
        }
        "md" => {
            let re = get_md_re();
            for cap in re.captures_iter(&content_lf) {
                if let Some(m) = cap.get(1) {
                    let link = m.as_str().trim();
                    if !link.is_empty() && !link.starts_with("http") && !link.starts_with("//") && !link.starts_with('#') {
                        let mut clean_link = link.to_string();
                        if let Some(idx) = clean_link.find(|c| c == '?' || c == '#') {
                            clean_link.truncate(idx);
                        }
                        deps.push(clean_link);
                    }
                }
            }
        }
        _ => {}
    }
    deps
}

/// 从 Vue 模板中提取出可能是自动引入组件的标签名（转为 PascalCase）
pub fn extract_vue_component_tags(content: &str) -> Vec<String> {
    let tag_re = get_vue_tag_re();
    let mut seen = HashSet::new();
    let mut tags = Vec::new();
    for cap in tag_re.captures_iter(content) {
        if let Some(m) = cap.get(1) {
            let tag = m.as_str();
            // 跳过已知组件库前缀
            if COMPONENT_LIB_PREFIXES.iter().any(|p| tag.starts_with(p)) {
                continue;
            }
            let pascal = if tag.contains('-') {
                // kebab-case → PascalCase，例如 my-component → MyComponent
                kebab_to_pascal(tag)
            } else {
                // 如果首字母不大写，视为原生 HTML 标签，直接跳过
                if !tag.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
                    continue;
                }
                tag.to_string()
            };
            if seen.insert(pascal.clone()) {
                tags.push(pascal);
            }
        }
    }
    tags
}

#[cfg(test)]
mod tests {
    use super::extract_dependencies;

    #[test]
    fn extract_dependencies_should_support_common_ts_patterns() {
        let content = r#"
import { Module } from '@nestjs/common';
import {
  DingTalkController,
} from './dingtalk.controller';
import type { DingTalkService } from "./dingtalk.service";
import './bootstrap';
export { createModule } from './module.factory';
import Config = require('./config');
const lazyModule = import('./lazy');
"#;

        let deps = extract_dependencies(content, "ts");

        assert_eq!(
            deps,
            vec![
                "@nestjs/common".to_string(),
                "./dingtalk.controller".to_string(),
                "./dingtalk.service".to_string(),
                "./bootstrap".to_string(),
                "./module.factory".to_string(),
                "./config".to_string(),
                "./lazy".to_string(),
            ]
        );
    }
}
