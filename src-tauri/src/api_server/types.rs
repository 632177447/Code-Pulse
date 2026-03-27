use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::AppHandle;
use tokio::sync::oneshot;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiRequest {
    pub id: String,
    pub url: String,
    pub method: String,
    pub headers: HashMap<String, String>,
    pub query: HashMap<String, String>,
    pub body: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse {
    pub status: u16,
    pub headers: Option<HashMap<String, String>>,
    pub body: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ContextRequest {
    pub paths: Vec<String>,
    pub max_depth: usize,
    pub ignore_exts: String,
    pub ignore_deep_parse: String,
    pub included_types: Vec<String>,
    pub project_roots: String,
    pub enable_minimization: bool,
    pub minimization_threshold: usize,
    pub minimization_depth_threshold: usize,
}

impl Default for ContextRequest {
    fn default() -> Self {
        Self {
            paths: Vec::new(),
            max_depth: 2,
            ignore_exts: ".git, node_modules, dist, target, build, .vscode, .idea, .next, .nuxt, .output, .vercel, .github, __pycache__, .venv, bin, obj, *.lock, *.log, *.tmp, *.temp, *.png, *.jpg, *.jpeg, *.gif, *.svg, *.ico, *.webp, *.mp4, *.avi, *.mkv, *.mov, *.webm, *.mp3, *.wav, *.flac, *.aac, *.ogg, *.zip, *.tar, *.gz, *.7z, *.rar, *.exe, *.dll, *.so, *.dylib".to_string(),
            ignore_deep_parse: "package.json, tsconfig.json, vite.config.ts, tauri.conf.json, README.md, Cargo.toml, go.mod, pom.xml, .env, *.test.ts, *.spec.ts".to_string(),
            included_types: vec![
                "vue".to_string(),
                "ts".to_string(),
                "tsx".to_string(),
                "js".to_string(),
                "py".to_string(),
                "json".to_string(),
                "css".to_string(),
                "scss".to_string(),
            ],
            project_roots: String::new(),
            enable_minimization: true,
            minimization_threshold: 8000,
            minimization_depth_threshold: 2,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct ContextQueryRequest {
    pub path: Option<String>,
    pub paths: Option<String>,
    pub max_depth: Option<usize>,
    pub ignore_exts: Option<String>,
    pub ignore_deep_parse: Option<String>,
    pub included_types: Option<String>,
    pub project_roots: Option<String>,
    pub enable_minimization: Option<bool>,
    pub minimization_threshold: Option<usize>,
    pub minimization_depth_threshold: Option<usize>,
}

impl From<ContextQueryRequest> for ContextRequest {
    fn from(value: ContextQueryRequest) -> Self {
        let mut request = ContextRequest::default();
        let mut paths = Vec::new();

        if let Some(path) = value.path {
            let trimmed = path.trim();
            if !trimmed.is_empty() {
                paths.push(trimmed.to_string());
            }
        }

        if let Some(multi_paths) = value.paths {
            paths.extend(
                multi_paths
                    .split(|c| c == ',' || c == '\n' || c == '\r')
                    .map(|item| item.trim())
                    .filter(|item| !item.is_empty())
                    .map(|item| item.to_string()),
            );
        }

        if !paths.is_empty() {
            request.paths = paths;
        }

        if let Some(max_depth) = value.max_depth {
            request.max_depth = max_depth;
        }

        if let Some(ignore_exts) = value.ignore_exts {
            request.ignore_exts = ignore_exts;
        }

        if let Some(ignore_deep_parse) = value.ignore_deep_parse {
            request.ignore_deep_parse = ignore_deep_parse;
        }

        if let Some(included_types) = value.included_types {
            request.included_types = included_types
                .split(|c| c == ',' || c == '\n' || c == '\r')
                .map(|item| item.trim().trim_start_matches('.'))
                .filter(|item| !item.is_empty())
                .map(|item| item.to_lowercase())
                .collect();
        }

        if let Some(project_roots) = value.project_roots {
            request.project_roots = project_roots;
        }

        if let Some(enable_minimization) = value.enable_minimization {
            request.enable_minimization = enable_minimization;
        }

        if let Some(minimization_threshold) = value.minimization_threshold {
            request.minimization_threshold = minimization_threshold;
        }

        if let Some(minimization_depth_threshold) = value.minimization_depth_threshold {
            request.minimization_depth_threshold = minimization_depth_threshold;
        }

        request
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OutlineNode {
    pub path: String,
    pub abs_path: String,
    pub depth: usize,
    pub dependencies: Vec<String>,
}

pub type PendingRequests = DashMap<String, oneshot::Sender<ApiResponse>>;

/// Context shared across all API handlers
#[derive(Clone)]
pub struct ServerContext {
    pub app_handle: AppHandle,
    pub app_state: crate::AppState,
    pub pending_requests: Arc<PendingRequests>,
}

pub struct ApiServerState {
    pub pending_requests: Arc<PendingRequests>,
    pub server_handle: Mutex<Option<oneshot::Sender<()>>>,
}

impl ApiServerState {
    pub fn new() -> Self {
        Self {
            pending_requests: Arc::new(DashMap::new()),
            server_handle: Mutex::new(None),
        }
    }
}
