use crate::protocol::{validate_request, JsonRpcRequest, JsonRpcResponse, RequestId};
use crate::tools::ToolRegistry;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::Router;
use serde_json::Value;
use std::convert::Infallible;
use std::sync::Arc;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;

/// Maximum request body size (1 MB).
const MAX_BODY_SIZE: usize = 1_048_576;

/// Maximum number of concurrent SSE connections.
const MAX_SSE_CONNECTIONS: usize = 10;

/// Shared server state passed to all handlers.
#[derive(Clone)]
pub struct AppState {
    pub registry: Arc<ToolRegistry>,
    pub sse_connections: Arc<tokio::sync::Semaphore>,
    pub event_sender: Arc<tokio::sync::broadcast::Sender<String>>,
}

/// Build the Axum app with MCP routes.
pub fn app(registry: ToolRegistry) -> Router {
    let (event_sender, _) = tokio::sync::broadcast::channel(100);

    let state = AppState {
        registry: Arc::new(registry),
        sse_connections: Arc::new(tokio::sync::Semaphore::new(MAX_SSE_CONNECTIONS)),
        event_sender: Arc::new(event_sender),
    };

    Router::new()
        .route("/sse", get(sse_handler))
        .route("/mcp", post(mcp_handler))
        .with_state(state)
}

/// SSE endpoint — MCP clients connect here to receive server-initiated events.
async fn sse_handler(State(state): State<AppState>) -> Result<impl IntoResponse, StatusCode> {
    // Enforce connection limit using owned permit (lives for 'static)
    let permit = state
        .sse_connections
        .clone()
        .try_acquire_owned()
        .map_err(|_| StatusCode::TOO_MANY_REQUESTS)?;

    let receiver = state.event_sender.subscribe();
    let stream = BroadcastStream::new(receiver).filter_map(|result| {
        match result {
            Ok(msg) => Some(Ok::<_, Infallible>(Event::default().data(msg))),
            Err(_) => None,
        }
    });

    // Hold the permit for the lifetime of the stream — drops when connection closes
    let stream = stream.map(move |item| {
        let _permit = &permit;
        item
    });

    let sse = Sse::new(stream).keep_alive(KeepAlive::default());
    Ok(sse.into_response())
}

/// MCP message handler — receives JSON-RPC requests via POST.
async fn mcp_handler(State(state): State<AppState>, body: axum::body::Bytes) -> impl IntoResponse {
    // Check body size
    if body.len() > MAX_BODY_SIZE {
        let resp = JsonRpcResponse::error(
            RequestId::Null,
            crate::protocol::ErrorCode::InvalidRequest as i32,
            "Payload too large".to_string(),
            None,
        );
        return (StatusCode::PAYLOAD_TOO_LARGE, axum::Json(resp)).into_response();
    }

    // Parse JSON
    let req: JsonRpcRequest = match serde_json::from_slice(&body) {
        Ok(r) => r,
        Err(_) => {
            let resp = JsonRpcResponse::parse_error(None);
            return (StatusCode::OK, axum::Json(resp)).into_response();
        }
    };

    let resp = dispatch_request(&state.registry, req).await;
    (StatusCode::OK, axum::Json(resp)).into_response()
}

/// Dispatch a JSON-RPC request to the appropriate handler.
///
/// This is the core dispatch function shared by both SSE and stdio transports.
/// Returns a JSON-RPC response. For MCP notifications (requests without an id),
/// returns a success response that should be silently discarded by stdio callers.
pub async fn dispatch_request(registry: &ToolRegistry, req: JsonRpcRequest) -> JsonRpcResponse {
    if let Err(err_resp) = validate_request(&req) {
        return err_resp;
    }

    let id = req.id.clone().unwrap_or(RequestId::Null);

    // MCP notifications (no id field) are acknowledged silently
    if req.id.is_none() {
        return JsonRpcResponse::success(id, serde_json::json!({}));
    }

    match req.method.as_str() {
        "initialize" => handle_initialize(id).await,
        "tools/list" => handle_tools_list(id, registry).await,
        "tools/call" => handle_tools_call(id, req.params, registry).await,
        _ => JsonRpcResponse::method_not_found(id, &req.method),
    }
}

/// Handle the MCP initialize method.
async fn handle_initialize(id: RequestId) -> JsonRpcResponse {
    JsonRpcResponse::success(
        id,
        serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {}
            },
            "serverInfo": {
                "name": "obsidian-mcp",
                "version": env!("CARGO_PKG_VERSION")
            }
        }),
    )
}

/// Handle the tools/list method.
async fn handle_tools_list(id: RequestId, registry: &ToolRegistry) -> JsonRpcResponse {
    let tools: Vec<Value> = registry
        .list()
        .iter()
        .map(|t| {
            serde_json::json!({
                "name": t.name,
                "description": t.description,
                "inputSchema": t.input_schema,
            })
        })
        .collect();

    JsonRpcResponse::success(id, serde_json::json!({ "tools": tools }))
}

/// Handle the tools/call method.
async fn handle_tools_call(
    id: RequestId,
    params: Option<Value>,
    registry: &ToolRegistry,
) -> JsonRpcResponse {
    let params = match params {
        Some(p) => p,
        None => return JsonRpcResponse::invalid_params(id, "missing params"),
    };

    let tool_name = match params.get("name").and_then(|n| n.as_str()) {
        Some(n) => n.to_string(),
        None => {
            return JsonRpcResponse::invalid_params(id, "missing tool name in params");
        }
    };

    let arguments = params.get("arguments").cloned().unwrap_or(serde_json::json!({}));

    match registry.call(&tool_name, arguments).await {
        Ok(result) => JsonRpcResponse::success(id, result),
        Err(err) => {
            // Surface tool errors — include isError flag for MCP compliance
            let error_data = err.data.clone().unwrap_or(serde_json::json!({"isError": true}));
            JsonRpcResponse::error(id, err.code, err.message, Some(error_data))
        }
    }
}
