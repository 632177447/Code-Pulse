use axum::{
    body::Body,
    extract::Request,
    extract::State,
    response::{IntoResponse, Response},
};
use serde_json::json;
use tauri::Emitter;
use tokio::sync::oneshot;
use uuid::Uuid;
use std::collections::HashMap;

use super::types::{ApiRequest, ServerContext};

pub async fn handle_bridge_request(
    State(ctx): State<ServerContext>,
    req: Request,
) -> impl IntoResponse {
    let id = Uuid::new_v4().to_string();
    
    // Extract method and URI
    let method = req.method().to_string();
    let url = req.uri().to_string();
    
    // Better query parsing (can be further optimized with axum::extract::Query if needed)
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
            .header("Content-Type", "application/json")
            .body(Body::from(json!({"error": "Failed to read body"}).to_string()))
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
    ctx.pending_requests.insert(id.clone(), tx);

    // Emit to frontend
    if let Err(e) = ctx.app_handle.emit("api-request", &api_req) {
        ctx.pending_requests.remove(&id);
        return Response::builder()
            .status(500)
            .header("Content-Type", "application/json")
            .body(Body::from(json!({
                "error": "Failed to emit event to frontend",
                "details": e.to_string()
            }).to_string()))
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
            builder.body(Body::from(api_res.body)).unwrap_or_else(|_| {
                Response::builder()
                    .status(500)
                    .header("Content-Type", "application/json")
                    .body(Body::from(json!({"error": "Internal Server Error during response building"}).to_string()))
                    .unwrap()
            })
        }
        _ => {
            ctx.pending_requests.remove(&id);
            Response::builder()
                .status(504)
                .header("Content-Type", "application/json")
                .body(Body::from(json!({
                    "error": "Gateway Timeout",
                    "details": "Frontend did not respond within 30 seconds"
                }).to_string()))
                .unwrap()
        }
    }
}
