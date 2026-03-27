use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;

use super::types::{ContextQueryRequest, ContextRequest, OutlineNode, ServerContext};

pub async fn handle_health() -> impl IntoResponse {
    Json(json!({
        "status": "ok",
        "meta": {
            "engine": "rust",
            "timestamp": chrono_like_timestamp(),
        }
    }))
}

pub async fn handle_info() -> impl IntoResponse {
    Json(json!({
        "data": {
            "name": "CodePulse API Service",
            "version": env!("CARGO_PKG_VERSION"),
            "description": "Local code analysis and context service",
            "routes": [
                "/api/health",
                "/api/info",
                "/api/cache",
                "/api/context",
                "/api/context/text",
                "/api/context/render",
                "/api/context/abort",
                "/api/outline"
            ]
        },
        "meta": {
            "engine": "rust"
        }
    }))
}

pub async fn handle_get_cache(
    State(ctx): State<ServerContext>,
) -> impl IntoResponse {
    Json(json!({
        "entries": ctx.app_state.parse_cache.size()
    }))
}

pub async fn handle_clear_cache(
    State(ctx): State<ServerContext>,
) -> impl IntoResponse {
    ctx.app_state.parse_cache.clear();
    Json(json!({
        "status": "ok",
        "entries": 0
    }))
}

pub async fn handle_abort_context(
    State(ctx): State<ServerContext>,
) -> impl IntoResponse {
    ctx.app_state.abort_handle.store(true, std::sync::atomic::Ordering::SeqCst);
    Json(json!({
        "status": "aborting"
    }))
}

pub async fn handle_get_context(
    State(ctx): State<ServerContext>,
    Query(query): Query<ContextQueryRequest>,
) -> impl IntoResponse {
    execute_context(ctx, query.into()).await
}

pub async fn handle_post_context(
    State(ctx): State<ServerContext>,
    Json(payload): Json<ContextRequest>,
) -> impl IntoResponse {
    execute_context(ctx, payload).await
}

pub async fn handle_get_outline(
    State(ctx): State<ServerContext>,
    Query(query): Query<ContextQueryRequest>,
) -> impl IntoResponse {
    execute_outline(ctx, query.into()).await
}

pub async fn handle_post_outline(
    State(ctx): State<ServerContext>,
    Json(payload): Json<ContextRequest>,
) -> impl IntoResponse {
    execute_outline(ctx, payload).await
}

async fn execute_context(
    ctx: ServerContext,
    request: ContextRequest,
) -> impl IntoResponse {
    if request.paths.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": {
                    "message": "Missing required field: paths",
                    "details": "Use POST /api/context with JSON body or GET /api/context?path=..."
                }
            })),
        );
    }

    match crate::run_generate_context(ctx.app_state, request).await {
        Ok(nodes) => (
            StatusCode::OK,
            Json(json!({
                "data": nodes,
                "meta": {
                    "count": nodes.len(),
                    "engine": "rust"
                }
            })),
        ),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": {
                    "message": "Failed to generate context",
                    "details": error
                }
            })),
        ),
    }
}

async fn execute_outline(
    ctx: ServerContext,
    request: ContextRequest,
) -> impl IntoResponse {
    if request.paths.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": {
                    "message": "Missing required field: paths",
                    "details": "Use POST /api/outline with JSON body or GET /api/outline?path=..."
                }
            })),
        );
    }

    match crate::run_generate_context(ctx.app_state, request).await {
        Ok(nodes) => {
            let outline: Vec<OutlineNode> = nodes.into_iter().map(|node| OutlineNode {
                path: node.path,
                abs_path: node.abs_path,
                depth: node.depth,
                dependencies: node.dependencies,
            }).collect();

            (
                StatusCode::OK,
                Json(json!({
                    "data": outline,
                    "meta": {
                        "count": outline.len(),
                        "engine": "rust",
                    }
                })),
            )
        }
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": {
                    "message": "Failed to generate outline",
                    "details": error
                }
            })),
        ),
    }
}

fn chrono_like_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    now.to_string()
}
