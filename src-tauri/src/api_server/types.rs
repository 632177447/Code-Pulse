use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::oneshot;
use dashmap::DashMap;
use std::sync::Arc;
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

pub type PendingRequests = DashMap<String, oneshot::Sender<ApiResponse>>;

/// Context shared across all API handlers
#[derive(Clone)]
pub struct ServerContext {
    pub app_handle: AppHandle,
    pub app_state: Arc<crate::AppState>,
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
