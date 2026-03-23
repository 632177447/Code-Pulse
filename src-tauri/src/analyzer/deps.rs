use regex::Regex;
use rustpython_parser::{ast as py_ast, Parse};
use std::collections::HashSet;
use std::sync::OnceLock;
use swc_common::{sync::Lrc, FileName, SourceMap};
use swc_ecma_ast::{
    CallExpr, Callee, ExportAll, Expr, ImportDecl, Lit, NamedExport, TsImportEqualsDecl,
    TsModuleRef,
};
use swc_ecma_parser::{EsSyntax, Parser, StringInput, Syntax, TsSyntax};
use swc_ecma_visit::{Visit, VisitWith};

use super::constants::*;
use super::regex::*;

static EMBEDDED_SCRIPT_RE: OnceLock<Regex> = OnceLock::new();
static SCRIPT_LANG_RE: OnceLock<Regex> = OnceLock::new();

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

fn strip_import_alias(part: &str) -> String {
    part.trim()
        .split_once(" as ")
        .map(|(name, _)| name.trim())
        .unwrap_or_else(|| part.trim())
        .trim_matches(|c| c == '(' || c == ')')
        .to_string()
}

fn normalize_python_module(module: &str) -> String {
    let module = module.trim();
    if module.is_empty() {
        return String::new();
    }

    if module.starts_with('.') {
        let count = module.chars().take_while(|&c| c == '.').count();
        let prefix = if count == 1 {
            "./".to_string()
        } else {
            "../".repeat(count - 1)
        };
        let remainder = module[count..].trim().trim_matches('.');
        if remainder.is_empty() {
            prefix
        } else {
            format!("{}{}", prefix, remainder.replace('.', "/"))
        }
    } else {
        module.replace('.', "/")
    }
}

fn join_dependency_path(base: &str, child: &str) -> String {
    let child = child
        .trim()
        .trim_matches(|c| c == '(' || c == ')')
        .replace('.', "/");
    if child.is_empty() {
        return base.to_string();
    }
    if base.is_empty() {
        return child;
    }
    if base.ends_with('/') {
        return format!("{}{}", base, child);
    }
    format!("{}/{}", base, child)
}

fn get_embedded_script_re() -> &'static Regex {
    EMBEDDED_SCRIPT_RE.get_or_init(|| {
        Regex::new(r#"(?is)<script\b([^>]*)>(.*?)</script>"#).unwrap()
    })
}

fn get_script_lang_re() -> &'static Regex {
    SCRIPT_LANG_RE.get_or_init(|| {
        Regex::new(r#"(?i)\blang\s*=\s*["']?([a-z0-9_+-]+)["']?"#).unwrap()
    })
}

fn detect_embedded_script_ext(attrs: &str) -> &'static str {
    let lang = get_script_lang_re()
        .captures(attrs)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_ascii_lowercase());

    match lang.as_deref() {
        Some("ts") | Some("typescript") => "ts",
        Some("tsx") => "tsx",
        Some("jsx") => "jsx",
        Some("js") | Some("javascript") => "js",
        Some("mjs") => "mjs",
        Some("cjs") => "js",
        _ => "js",
    }
}

fn extract_embedded_script_blocks(content: &str) -> Vec<(String, String)> {
    let mut blocks = Vec::new();
    for caps in get_embedded_script_re().captures_iter(content) {
        let attrs = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        let code = caps.get(2).map(|m| m.as_str()).unwrap_or("");
        blocks.push((code.to_string(), detect_embedded_script_ext(attrs).to_string()));
    }
    blocks
}

fn build_js_ts_syntax(ext: &str) -> Syntax {
    match ext {
        "ts" => Syntax::Typescript(TsSyntax {
            decorators: true,
            ..Default::default()
        }),
        "tsx" => Syntax::Typescript(TsSyntax {
            tsx: true,
            decorators: true,
            ..Default::default()
        }),
        "jsx" => Syntax::Es(EsSyntax {
            jsx: true,
            decorators: true,
            ..Default::default()
        }),
        _ => Syntax::Es(EsSyntax {
            decorators: true,
            ..Default::default()
        }),
    }
}

fn extract_string_literal(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Lit(Lit::Str(value)) => Some(value.value.to_string_lossy().into_owned()),
        _ => None,
    }
}

#[derive(Default)]
struct SwcDependencyCollector {
    deps: Vec<String>,
}

impl SwcDependencyCollector {
    fn push_specifier(&mut self, value: &str) {
        self.deps.push(value.to_string());
    }
}

impl Visit for SwcDependencyCollector {
    fn visit_import_decl(&mut self, node: &ImportDecl) {
        self.push_specifier(&node.src.value.to_string_lossy());
    }

    fn visit_named_export(&mut self, node: &NamedExport) {
        if let Some(src) = &node.src {
            self.push_specifier(&src.value.to_string_lossy());
        }
    }

    fn visit_export_all(&mut self, node: &ExportAll) {
        self.push_specifier(&node.src.value.to_string_lossy());
    }

    fn visit_ts_import_equals_decl(&mut self, node: &TsImportEqualsDecl) {
        if let TsModuleRef::TsExternalModuleRef(external) = &node.module_ref {
            self.push_specifier(&external.expr.value.to_string_lossy());
        }
    }

    fn visit_call_expr(&mut self, node: &CallExpr) {
        let should_collect = matches!(node.callee, Callee::Import(_))
            || matches!(
                &node.callee,
                Callee::Expr(expr)
                    if matches!(
                        expr.as_ref(),
                        Expr::Ident(ident) if ident.sym.as_ref() == "require"
                    )
            );

        if should_collect {
            if let Some(first_arg) = node.args.first() {
                if let Some(value) = extract_string_literal(first_arg.expr.as_ref()) {
                    self.push_specifier(&value);
                }
            }
        }

        node.visit_children_with(self);
    }
}

fn extract_js_ts_dependencies_by_ast(content: &str, ext: &str) -> Option<Vec<String>> {
    let cm: Lrc<SourceMap> = Default::default();
    let fm = cm.new_source_file(
        FileName::Custom(format!("embedded.{}", ext)).into(),
        content.to_string(),
    );
    let mut parser = Parser::new(build_js_ts_syntax(ext), StringInput::from(&*fm), None);
    let module = parser.parse_module().ok()?;

    let mut collector = SwcDependencyCollector::default();
    module.visit_with(&mut collector);
    Some(collector.deps)
}

fn level_to_count(level: Option<&py_ast::Int>) -> usize {
    level.map(|value| value.to_usize()).unwrap_or(0)
}

fn normalize_python_from_module(module: Option<&str>, level: usize) -> String {
    let prefix = match level {
        0 => String::new(),
        1 => "./".to_string(),
        _ => "../".repeat(level - 1),
    };

    match module {
        Some(module) if !module.trim().is_empty() => {
            format!("{}{}", prefix, module.trim().replace('.', "/"))
        }
        _ => prefix,
    }
}

fn collect_python_suite_dependencies(suite: &py_ast::Suite, deps: &mut Vec<String>) {
    for stmt in suite {
        collect_python_stmt_dependencies(stmt, deps);
    }
}

fn collect_python_handler_dependencies(handler: &py_ast::ExceptHandler, deps: &mut Vec<String>) {
    match handler {
        py_ast::ExceptHandler::ExceptHandler(handler) => {
            collect_python_suite_dependencies(&handler.body, deps);
        }
    }
}

fn collect_python_stmt_dependencies(stmt: &py_ast::Stmt, deps: &mut Vec<String>) {
    match stmt {
        py_ast::Stmt::Import(import_stmt) => {
            for alias in &import_stmt.names {
                let name = normalize_python_module(alias.name.as_ref());
                if !name.is_empty() {
                    deps.push(name);
                }
            }
        }
        py_ast::Stmt::ImportFrom(import_from_stmt) => {
            let base = normalize_python_from_module(
                import_from_stmt.module.as_ref().map(|module| module.as_ref()),
                level_to_count(import_from_stmt.level.as_ref()),
            );

            if import_from_stmt.module.is_some() && !base.is_empty() {
                deps.push(base.clone());
            }

            for alias in &import_from_stmt.names {
                let name = alias.name.as_ref();
                if name == "*" {
                    continue;
                }
                deps.push(join_dependency_path(&base, name));
            }
        }
        py_ast::Stmt::FunctionDef(stmt) => collect_python_suite_dependencies(&stmt.body, deps),
        py_ast::Stmt::AsyncFunctionDef(stmt) => collect_python_suite_dependencies(&stmt.body, deps),
        py_ast::Stmt::ClassDef(stmt) => collect_python_suite_dependencies(&stmt.body, deps),
        py_ast::Stmt::For(stmt) => {
            collect_python_suite_dependencies(&stmt.body, deps);
            collect_python_suite_dependencies(&stmt.orelse, deps);
        }
        py_ast::Stmt::AsyncFor(stmt) => {
            collect_python_suite_dependencies(&stmt.body, deps);
            collect_python_suite_dependencies(&stmt.orelse, deps);
        }
        py_ast::Stmt::While(stmt) => {
            collect_python_suite_dependencies(&stmt.body, deps);
            collect_python_suite_dependencies(&stmt.orelse, deps);
        }
        py_ast::Stmt::If(stmt) => {
            collect_python_suite_dependencies(&stmt.body, deps);
            collect_python_suite_dependencies(&stmt.orelse, deps);
        }
        py_ast::Stmt::With(stmt) => collect_python_suite_dependencies(&stmt.body, deps),
        py_ast::Stmt::AsyncWith(stmt) => collect_python_suite_dependencies(&stmt.body, deps),
        py_ast::Stmt::Match(stmt) => {
            for case in &stmt.cases {
                collect_python_suite_dependencies(&case.body, deps);
            }
        }
        py_ast::Stmt::Try(stmt) => {
            collect_python_suite_dependencies(&stmt.body, deps);
            collect_python_suite_dependencies(&stmt.orelse, deps);
            collect_python_suite_dependencies(&stmt.finalbody, deps);
            for handler in &stmt.handlers {
                collect_python_handler_dependencies(handler, deps);
            }
        }
        py_ast::Stmt::TryStar(stmt) => {
            collect_python_suite_dependencies(&stmt.body, deps);
            collect_python_suite_dependencies(&stmt.orelse, deps);
            collect_python_suite_dependencies(&stmt.finalbody, deps);
            for handler in &stmt.handlers {
                collect_python_handler_dependencies(handler, deps);
            }
        }
        _ => {}
    }
}

fn extract_python_dependencies_by_ast(content: &str) -> Option<Vec<String>> {
    let suite = py_ast::Suite::parse(content, "<embedded>").ok()?;
    let mut deps = Vec::new();
    collect_python_suite_dependencies(&suite, &mut deps);
    Some(deps)
}

fn extract_embedded_script_dependencies_by_ast(content: &str) -> Option<Vec<String>> {
    let blocks = extract_embedded_script_blocks(content);
    let mut deps = Vec::new();

    for (block_content, block_ext) in blocks {
        deps.extend(extract_js_ts_dependencies_by_ast(&block_content, &block_ext)?);
    }

    Some(deps)
}

fn extract_dependencies_by_regex(content: &str, ext: &str) -> Vec<String> {
    let mut deps = Vec::new();
    let content_stripped = strip_comments(content, ext);
    let content_lf = content_stripped.replace("\r\n", "\n");

    match ext {
        e if JS_TS_FAMILY.contains(&e) => {
            let re = get_js_re();
            for cap in re.captures_iter(&content_lf) {
                if let Some(m) = cap
                    .get(1)
                    .or(cap.get(2))
                    .or(cap.get(3))
                    .or(cap.get(4))
                    .or(cap.get(5))
                {
                    deps.push(m.as_str().to_string());
                }
            }
        }
        "py" => {
            let import_re = get_py_import_re();
            let from_re = get_py_from_re();

            for cap in import_re.captures_iter(&content_lf) {
                if let Some(m) = cap.get(1) {
                    for part in m.as_str().split(',') {
                        let name = strip_import_alias(part);
                        if name.is_empty() {
                            continue;
                        }
                        deps.push(normalize_python_module(&name));
                    }
                }
            }

            for cap in from_re.captures_iter(&content_lf) {
                let base = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                let imports = cap.get(2).map(|m| m.as_str()).unwrap_or("");
                let normalized_base = normalize_python_module(base);
                let pure_relative = base.chars().all(|c| c == '.');

                if !normalized_base.is_empty() && !pure_relative {
                    deps.push(normalized_base.clone());
                }

                for part in imports.split(',') {
                    let name = strip_import_alias(part);
                    if name.is_empty() || name == "*" {
                        continue;
                    }
                    deps.push(join_dependency_path(&normalized_base, &name));
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
                    if !link.is_empty()
                        && !link.starts_with("http")
                        && !link.starts_with("//")
                        && !link.starts_with('#')
                    {
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

pub fn extract_dependencies(content: &str, ext: &str) -> Vec<String> {
    match ext {
        "vue" | "svelte" => extract_embedded_script_dependencies_by_ast(content)
            .unwrap_or_else(|| extract_dependencies_by_regex(content, ext)),
        "py" => extract_python_dependencies_by_ast(content)
            .unwrap_or_else(|| extract_dependencies_by_regex(content, ext)),
        e if JS_TS_FAMILY.contains(&e) => extract_js_ts_dependencies_by_ast(content, ext)
            .unwrap_or_else(|| extract_dependencies_by_regex(content, ext)),
        _ => extract_dependencies_by_regex(content, ext),
    }
}

pub fn extract_vue_component_tags(content: &str) -> Vec<String> {
    let tag_re = get_vue_tag_re();
    let mut seen = HashSet::new();
    let mut tags = Vec::new();
    for cap in tag_re.captures_iter(content) {
        if let Some(m) = cap.get(1) {
            let tag = m.as_str();
            if COMPONENT_LIB_PREFIXES.iter().any(|p| tag.starts_with(p)) {
                continue;
            }
            let pascal = if tag.contains('-') {
                kebab_to_pascal(tag)
            } else {
                if !tag
                    .chars()
                    .next()
                    .map(|c| c.is_uppercase())
                    .unwrap_or(false)
                {
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

    #[test]
    fn extract_dependencies_should_support_common_python_patterns() {
        let content = r#"
import os, app.config as config
from . import utils
from ..services import dingtalk_service, helpers as helper_alias
"#;

        let deps = extract_dependencies(content, "py");

        assert_eq!(
            deps,
            vec![
                "os".to_string(),
                "app/config".to_string(),
                "./utils".to_string(),
                "../services".to_string(),
                "../services/dingtalk_service".to_string(),
                "../services/helpers".to_string(),
            ]
        );
    }

    #[test]
    fn extract_dependencies_should_support_common_go_patterns() {
        let content = r#"
import alias "internal/pkg"
import (
    _ "side/effect"
    log "app/logger"
    "fmt"
)
"#;

        let deps = extract_dependencies(content, "go");

        assert_eq!(
            deps,
            vec![
                "internal/pkg".to_string(),
                "side/effect".to_string(),
                "app/logger".to_string(),
                "fmt".to_string(),
            ]
        );
    }

    #[test]
    fn extract_dependencies_should_support_common_rust_and_php_patterns() {
        let rust_content = r#"
pub mod parser;
pub(crate) mod lexer;
use crate::analyzer::resolve;
"#;
        let php_content = r#"
require_once('./bootstrap.php');
include_once "./helpers.php";
use App\Services\DingTalk as DingTalkService;
"#;

        assert_eq!(
            extract_dependencies(rust_content, "rs"),
            vec![
                "parser".to_string(),
                "lexer".to_string(),
                "crate/analyzer/resolve".to_string(),
            ]
        );

        assert_eq!(
            extract_dependencies(php_content, "php"),
            vec![
                "./bootstrap.php".to_string(),
                "./helpers.php".to_string(),
                "App/Services/DingTalk".to_string(),
            ]
        );
    }

    #[test]
    fn extract_dependencies_should_support_vue_script_blocks() {
        let content = r#"
<template>
  <div />
</template>
<script setup lang="ts">
import Foo from './Foo.vue';
const lazy = import('./lazy');
</script>
<script>
const helper = require('./helper');
</script>
"#;

        let deps = extract_dependencies(content, "vue");

        assert_eq!(
            deps,
            vec![
                "./Foo.vue".to_string(),
                "./lazy".to_string(),
                "./helper".to_string(),
            ]
        );
    }
}
