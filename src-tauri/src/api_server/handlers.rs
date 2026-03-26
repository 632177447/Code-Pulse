use axum::{extract::State, response::IntoResponse, Json};
use serde_json::json;
use super::types::ServerContext;

/// Native Rust health check
pub async fn handle_rust_health(
    State(ctx): State<ServerContext>,
) -> impl IntoResponse {
    Json(json!({
        "status": "ok",
        "engine": "rust",
        "port": ctx.app_handle.inner_hmac_key().is_some(), // Just a placeholder use of context
        "message": "This request was handled natively in Rust without bridge overhead."
    }))
}

/// Native placeholder for context generation
/// This demonstrates how it CAN access ctx.app_state
pub async fn handle_rust_context_placeholder(
    State(ctx): State<ServerContext>,
) -> impl IntoResponse {
    // In a real implementation, you would:
    // 1. Parse request body for paths/params
    // 2. Call analyzer::analyze_dependencies(ctx.app_state.clone(), ...)
    
    Json(json!({
        "status": "native_connected",
        "cache_entries": ctx.app_state.parse_cache.size(),
        "message": "Native Rust context generation is connected to the core engine."
    }))
}
