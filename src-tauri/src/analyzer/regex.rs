// 正则表达式模块：预定义用于匹配依赖项且高效剥离代码注释的正则规则

use regex::Regex;
use std::sync::OnceLock;

use super::constants::*;

// js/ts: import { ... } from "..." | import "..." | require("...")
static JS_RE: OnceLock<Regex> = OnceLock::new();
// python: import ... | from ... import ...
static PY_RE: OnceLock<Regex> = OnceLock::new();
// rust: use ...; | mod ...;
static RS_RE: OnceLock<Regex> = OnceLock::new();
// go: import "..." | import ( ... )
static GO_RE: OnceLock<Regex> = OnceLock::new();
// java/kotlin: import ...;
static JAVA_RE: OnceLock<Regex> = OnceLock::new();
// c/cpp: #include "..."
static CPP_RE: OnceLock<Regex> = OnceLock::new();
// csharp: using ...;
static CS_RE: OnceLock<Regex> = OnceLock::new();
// php: require '...'; | include '...'; | use ...;
static PHP_RE: OnceLock<Regex> = OnceLock::new();
// ruby: require '...' | require_relative '...'
static RB_RE: OnceLock<Regex> = OnceLock::new();
// css/scss/less: @import "..."; | @import url("...");
static CSS_RE: OnceLock<Regex> = OnceLock::new();
static STR_RE: OnceLock<Regex> = OnceLock::new();
// html: <script src="..."> | <link href="...">
static HTML_RE: OnceLock<Regex> = OnceLock::new();
// md: [text](link)
static MD_RE: OnceLock<Regex> = OnceLock::new();
// vue tags: <DependencyTreeSidebar ... | <dependency-tree-sidebar ...
static VUE_TAG_RE: OnceLock<Regex> = OnceLock::new();

// 注释剥离辅助正则
static C_STYLE_RE: OnceLock<Regex> = OnceLock::new();
static HASH_STYLE_RE: OnceLock<Regex> = OnceLock::new();
static HTML_COMMENT_RE: OnceLock<Regex> = OnceLock::new();

fn get_c_style_re() -> &'static Regex {
    C_STYLE_RE.get_or_init(|| {
        // 匹配 /*...*/ 或 //... 或 字符串（保持字符串不变以免误删内部路径）
        Regex::new(r#"(?s)/\*.*?\*/|//[^\n]*|"(?:\\.|[^"\\])*"|'(?:\\.|[^'\\])*'|`(?:\\.|[^`\\])*`"#).unwrap()
    })
}

fn get_hash_style_re() -> &'static Regex {
    HASH_STYLE_RE.get_or_init(|| {
        // 匹配 #... 或 字符串
        Regex::new(r#"(?s)#[^\n]*|"(?:\\.|[^"\\])*"|'(?:\\.|[^'\\])*'|"""(?:\\.|[^"])*"""|'''(?:\\.|[^'])*'''"#).unwrap()
    })
}

fn get_html_comment_re() -> &'static Regex {
    HTML_COMMENT_RE.get_or_init(|| {
        Regex::new(r#"(?s)<!--.*?-->"#).unwrap()
    })
}

pub fn strip_comments(content: &str, ext: &str) -> String {
    match ext {
        e if C_STYLE_COMMENT_EXTS.contains(&e) => {
            let re = get_c_style_re();
            re.replace_all(content, |caps: &regex::Captures| {
                let m = caps.get(0).unwrap().as_str();
                if m.starts_with('/') {
                    // 保留换行符以维持行数和锚点正确性
                    m.chars().map(|c| if c == '\n' { '\n' } else { ' ' }).collect::<String>()
                } else {
                    m.to_string()
                }
            }).into_owned()
        }
        e if HASH_STYLE_COMMENT_EXTS.contains(&e) => {
            let re = get_hash_style_re();
            re.replace_all(content, |caps: &regex::Captures| {
                let m = caps.get(0).unwrap().as_str();
                if m.starts_with('#') {
                    " ".to_string()
                } else {
                    m.to_string()
                }
            }).into_owned()
        }
        "html" => {
            let re = get_html_comment_re();
            re.replace_all(content, |caps: &regex::Captures| {
                let m = caps.get(0).unwrap().as_str();
                m.chars().map(|c| if c == '\n' { '\n' } else { ' ' }).collect::<String>()
            }).into_owned()
        }
        e if MIXED_STYLE_COMMENT_EXTS.contains(&e) => {
            // 先剥离 HTML 注释，再剥离 JS 注释
            let html_re = get_html_comment_re();
            let intermediate = html_re.replace_all(content, |caps: &regex::Captures| {
                let m = caps.get(0).unwrap().as_str();
                m.chars().map(|c| if c == '\n' { '\n' } else { ' ' }).collect::<String>()
            }).into_owned();
            let c_re = get_c_style_re();
            c_re.replace_all(&intermediate, |caps: &regex::Captures| {
                let m = caps.get(0).unwrap().as_str();
                if m.starts_with('/') {
                    m.chars().map(|c| if c == '\n' { '\n' } else { ' ' }).collect::<String>()
                } else {
                    m.to_string()
                }
            }).into_owned()
        }
        _ => content.to_string(),
    }
}

pub fn get_js_re() -> &'static Regex {
    JS_RE.get_or_init(|| {
        // 兼容多行 import/export、side-effect import、dynamic import 与 TS 的 import = require
        Regex::new(r#"(?ms)(?:^\s*(?:import|export)\b(?:[\s\w{},*$]+?)\bfrom\s+['"]([^'"]+)['"]|^\s*import\s+['"]([^'"]+)['"]|require\s*\(\s*['"]([^'"]+)['"]\s*\)|import\s*\(\s*['"]([^'"]+)['"]\s*\)|^\s*import\s+[A-Za-z_$][\w$]*\s*=\s*require\s*\(\s*['"]([^'"]+)['"]\s*\))"#).unwrap()
    })
}

pub fn get_py_re() -> &'static Regex {
    PY_RE.get_or_init(|| {
        Regex::new(r#"(?m)^\s*(?:import|from)\s+([a-zA-Z0-9_\.]+)"#).unwrap()
    })
}

pub fn get_rs_re() -> &'static Regex {
    RS_RE.get_or_init(|| {
        Regex::new(r#"(?m)^\s*(?:use|mod)\s+([a-zA-Z0-9_:]+)"#).unwrap()
    })
}

pub fn get_go_re() -> &'static Regex {
    GO_RE.get_or_init(|| {
        Regex::new(r#"(?m)^\s*import\s+(?:\(\s*([\s\S]*?)\s*\)|['"]([^'"]+)['"])"#).unwrap()
    })
}

pub fn get_str_re() -> &'static Regex {
    STR_RE.get_or_init(|| {
        Regex::new(r#"['"]([^'"]+)['"]"#).unwrap()
    })
}

pub fn get_java_re() -> &'static Regex {
    JAVA_RE.get_or_init(|| {
        Regex::new(r#"(?m)^\s*import\s+([a-zA-Z0-9_\.]+);?"#).unwrap()
    })
}

pub fn get_cpp_re() -> &'static Regex {
    CPP_RE.get_or_init(|| {
        Regex::new(r#"(?m)^\s*#include\s+["<]([^">]+)[">]"#).unwrap()
    })
}

pub fn get_cs_re() -> &'static Regex {
    CS_RE.get_or_init(|| {
        Regex::new(r#"(?m)^\s*using\s+(?:static\s+)?([a-zA-Z0-9_\.]+);"#).unwrap()
    })
}

pub fn get_php_re() -> &'static Regex {
    PHP_RE.get_or_init(|| {
        Regex::new(r#"(?m)^\s*(?:(?:require|include)(?:_once)?\s*['"]([^'"]+)['"]|use\s+([a-zA-Z0-9_\\]+);)"#).unwrap()
    })
}

pub fn get_rb_re() -> &'static Regex {
    RB_RE.get_or_init(|| {
        Regex::new(r#"(?m)^\s*require(?:_relative)?\s*['"]([^'"]+)['"]"#).unwrap()
    })
}

pub fn get_css_re() -> &'static Regex {
    CSS_RE.get_or_init(|| {
        Regex::new(r#"(?m)^\s*@import\s+(?:url\(['"]?([^'"]+?)['"]?\)|['"]([^'"]+)['"])"#).unwrap()
    })
}

pub fn get_html_re() -> &'static Regex {
    HTML_RE.get_or_init(|| {
        Regex::new(r#"(?i)<(?:script[^>]+src|link[^>]+href)\s*=\s*['"]([^'"]+)['"]"#).unwrap()
    })
}

pub fn get_md_re() -> &'static Regex {
    MD_RE.get_or_init(|| {
        Regex::new(r#"\[[^\]]*\]\(([^)]+)\)"#).unwrap()
    })
}

pub fn get_vue_tag_re() -> &'static Regex {
    VUE_TAG_RE.get_or_init(|| {
        // 匹配 HTML 标签名
        Regex::new(r#"(?m)<([a-zA-Z][a-zA-Z0-9-]*)[^>]*"#).unwrap()
    })
}
