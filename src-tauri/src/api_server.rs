use axum::{
    extract::Request,
    response::Response,
    routing::any,
    Router,
};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::{oneshot, Mutex};
use uuid::Uuid;

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

pub async fn start_server(app: AppHandle, state: tauri::State<'_, ApiServerState>, port: u16) -> Result<(), String> {
    // 1. stop existing server
    {
        let mut handle_opt = state.server_handle.lock().await;
        if let Some(tx) = handle_opt.take() {
            let _ = tx.send(()); // abort the previous server
        }
    }

    let pending_requests = state.pending_requests.clone();
    
    // 2. build router
    let app_for_path = app.clone();
    let pending_for_path = pending_requests.clone();
    let app_for_root = app.clone();
    let pending_for_root = pending_requests.clone();

    let router = Router::new()
        .route("/{*path}", any(move |req: Request| async move { handle_request(app_for_path, pending_for_path, req).await }))
        .route("/", any(move |req: Request| async move { handle_request(app_for_root, pending_for_root, req).await }));

    use tower_http::cors::CorsLayer;
    let router = router.layer(CorsLayer::permissive());

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(&addr).await.map_err(|e| e.to_string())?;

    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

    tokio::spawn(async move {
        let _ = axum::serve(listener, router)
            .with_graceful_shutdown(async move {
                let _ = shutdown_rx.await;
            })
            .await;
    });

    // 3. save new handle
    {
        let mut handle_opt = state.server_handle.lock().await;
        *handle_opt = Some(shutdown_tx);
    }

    Ok(())
}

pub async fn stop_server(state: tauri::State<'_, ApiServerState>) -> Result<(), String> {
    let mut handle_opt = state.server_handle.lock().await;
    if let Some(tx) = handle_opt.take() {
        let _ = tx.send(());
    }
    Ok(())
}

async fn handle_request(
    app: AppHandle,
    pending: Arc<PendingRequests>,
    req: Request,
) -> Response<axum::body::Body> {
    let id = Uuid::new_v4().to_string();
    
    // Extract method, uri
    let method = req.method().to_string();
    let url = req.uri().to_string();
    
    // Extract query
    let query_string = req.uri().query().unwrap_or("");
    let mut query = HashMap::new();
    for pair in query_string.split('&') {
        if pair.is_empty() { continue; }
        let mut parts = pair.splitn(2, '=');
        let key = parts.next().unwrap_or("").to_string();
        let value = parts.next().unwrap_or("").to_string();
        query.insert(key, value);
    }
    
    // Extract headers
    let mut headers_map = HashMap::new();
    for (k, v) in req.headers().iter() {
        if let Ok(val) = v.to_str() {
            headers_map.insert(k.as_str().to_string(), val.to_string());
        }
    }

    // Extract body
    let body_bytes = match axum::body::to_bytes(req.into_body(), usize::MAX).await {
        Ok(b) => b,
        Err(_) => return Response::builder()
            .status(400)
            .body(axum::body::Body::from("Failed to read body"))
            .unwrap(),
    };
    
    let body_str = String::from_utf8(body_bytes.to_vec()).ok();

    let api_req = ApiRequest {
        id: id.clone(),
        url,
        method,
        headers: headers_map,
        query,
        body: body_str,
    };

    let (tx, rx) = oneshot::channel();
    pending.insert(id.clone(), tx);

    // emit to frontend
    if let Err(e) = app.emit("api-request", &api_req) {
        pending.remove(&id);
        return Response::builder()
            .status(500)
            .body(axum::body::Body::from(format!("Failed to emit event: {}", e)))
            .unwrap();
    }

    // Wait for response with timeout
    match tokio::time::timeout(std::time::Duration::from_secs(30), rx).await {
        Ok(Ok(api_res)) => {
            let mut builder = Response::builder().status(api_res.status);
            if let Some(h) = api_res.headers {
                for (k, v) in h {
                    builder = builder.header(k, v);
                }
            }
            builder.body(axum::body::Body::from(api_res.body)).unwrap_or_else(|_| {
                Response::builder().status(500).body(axum::body::Body::from("Internal Server Error")).unwrap()
            })
        }
        _ => {
            pending.remove(&id);
            Response::builder()
                .status(504)
                .body(axum::body::Body::from("Gateway Timeout"))
                .unwrap()
        }
    }
}
