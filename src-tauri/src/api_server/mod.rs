pub mod types;
pub mod bridge;
pub mod handlers;

pub use types::*;
use bridge::handle_bridge_request;
use handlers::{handle_rust_health, handle_rust_context_placeholder};

use axum::{
    routing::{any, get, post},
    Router,
};
use std::net::SocketAddr;
use tokio::sync::oneshot;
use tauri::AppHandle;

pub async fn start_server(app: AppHandle, state: tauri::State<'_, ApiServerState>, port: u16) -> Result<(), String> {
    // 1. Stop existing server
    {
        let mut handle_opt = state.server_handle.lock().await;
        if let Some(tx) = handle_opt.take() {
            let _ = tx.send(()); 
        }
    }

    // 2. Prepare context
    let app_state = app.state::<crate::AppState>().inner().clone();
    let ctx = ServerContext {
        app_handle: app.clone(),
        app_state: Arc::new(app_state),
        pending_requests: state.pending_requests.clone(),
    };
    
    // 2. Build router with hybrid strategy
    let router = Router::new()
        // --- Native Rust Routes (Highest Priority) ---
        .route("/api/rust/health", get(handle_rust_health))
        .route("/api/rust/context", post(handle_rust_context_placeholder))
        
        // --- Frontend Bridge (Fallback for everything else) ---
        .fallback(any(move |state: axum::extract::State<ServerContext>, req| {
            handle_bridge_request(state.app_handle.clone(), state.pending_requests.clone(), req)
        }))
        .with_state(ctx);

    // Add CORS
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

    // 3. Save new handle
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
