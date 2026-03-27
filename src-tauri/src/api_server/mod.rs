pub mod types;
pub mod bridge;
pub mod handlers;

pub use types::*;
use bridge::handle_bridge_request;
use handlers::{
    handle_abort_context,
    handle_clear_cache,
    handle_get_cache,
    handle_get_context,
    handle_get_outline,
    handle_health,
    handle_info,
    handle_post_context,
    handle_post_outline,
};

use axum::{
    routing::{any, get, post},
    Router,
};
use std::net::SocketAddr;
use tokio::sync::oneshot;
use tauri::{AppHandle, Manager};

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
        app_state,
        pending_requests: state.pending_requests.clone(),
    };
    
    // 2. Build router with hybrid strategy
    let router = Router::new()
        .route("/api/health", get(handle_health))
        .route("/api/info", get(handle_info))
        .route("/api/cache", get(handle_get_cache).delete(handle_clear_cache))
        .route("/api/context", get(handle_get_context).post(handle_post_context))
        .route("/api/context/abort", post(handle_abort_context))
        .route("/api/outline", get(handle_get_outline).post(handle_post_outline))
        .fallback(any(handle_bridge_request))
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
