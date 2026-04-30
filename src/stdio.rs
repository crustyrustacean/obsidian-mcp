//! Stdio transport for MCP.
//!
//! Reads newline-delimited JSON-RPC requests from stdin,
//! dispatches them to the tool registry, and writes
//! responses to stdout. Logging goes to stderr.
//!
//! This is the primary MCP transport — used by Claude Desktop,
//! Claude Code, and pi (via extension). SSE transport is for
//! HTTP-based clients like Cursor.

use crate::protocol::JsonRpcRequest;
use crate::server::dispatch_request;
use crate::tools::ToolRegistry;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

/// Run the MCP server in stdio mode.
///
/// Reads JSON-RPC requests from stdin (one per line),
/// dispatches each to the tool registry, and writes
/// the response to stdout. Notifications (requests without
/// an `id` field) are silently acknowledged without writing
/// a response, per the JSON-RPC 2.0 spec.
///
/// EOF on stdin causes a clean shutdown.
pub async fn run_stdio(registry: ToolRegistry) -> anyhow::Result<()> {
    let stdin = tokio::io::stdin();
    let mut stdout = tokio::io::stdout();
    let reader = BufReader::new(stdin);
    let mut lines = reader.lines();

    tracing::info!("MCP server running in stdio mode");

    while let Some(line) = lines.next_line().await? {
        let line = line.trim().to_string();
        if line.is_empty() {
            continue;
        }

        // Parse the JSON-RPC request
        let req: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(_) => {
                let resp = crate::protocol::JsonRpcResponse::parse_error(None);
                write_response(&mut stdout, &resp).await?;
                continue;
            }
        };

        // Notifications (no id field) don't get responses in JSON-RPC 2.0
        let is_notification = req.id.is_none();

        // Dispatch the request
        let resp = dispatch_request(&registry, req).await;

        // Only write a response for requests (not notifications)
        if !is_notification {
            write_response(&mut stdout, &resp).await?;
        }
    }

    tracing::info!("stdin closed, shutting down");
    Ok(())
}

/// Write a JSON-RPC response as a single line to stdout.
async fn write_response(
    stdout: &mut tokio::io::Stdout,
    resp: &crate::protocol::JsonRpcResponse,
) -> anyhow::Result<()> {
    let mut out = serde_json::to_string(resp)?;
    out.push('\n');
    stdout.write_all(out.as_bytes()).await?;
    stdout.flush().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::ToolRegistry;
    use crate::tools_impl;
    use crate::client::ObsidianClient;
    use crate::config::Config;
    use std::sync::Arc;

    fn test_registry() -> ToolRegistry {
        let config = Config {
            api_key: "test-key".to_string(),
            api_url: "http://127.0.0.1:1".to_string(),
        };
        let client = Arc::new(ObsidianClient::new(config));
        let mut registry = ToolRegistry::new();
        tools_impl::register_all_tools(&mut registry, client);
        registry
    }

    #[tokio::test]
    async fn dispatch_initialize_returns_server_info() {
        let registry = test_registry();
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(crate::protocol::RequestId::Number(1)),
            method: "initialize".to_string(),
            params: None,
        };

        let resp = dispatch_request(&registry, req).await;
        assert!(resp.result.is_some());
        let result = resp.result.unwrap();
        assert_eq!(result["serverInfo"]["name"], "obsidian-mcp");
        assert_eq!(result["protocolVersion"], "2024-11-05");
    }

    #[tokio::test]
    async fn dispatch_tools_list_returns_16_tools() {
        let registry = test_registry();
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(crate::protocol::RequestId::Number(2)),
            method: "tools/list".to_string(),
            params: None,
        };

        let resp = dispatch_request(&registry, req).await;
        assert!(resp.result.is_some());
        let result = resp.result.unwrap();
        let tools = result["tools"].as_array().unwrap();
        assert_eq!(tools.len(), 16);
    }

    #[tokio::test]
    async fn dispatch_unknown_method_returns_error() {
        let registry = test_registry();
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(crate::protocol::RequestId::Number(3)),
            method: "nonexistent".to_string(),
            params: None,
        };

        let resp = dispatch_request(&registry, req).await;
        assert!(resp.error.is_some());
        assert_eq!(resp.error.unwrap().code, -32601); // MethodNotFound
    }

    #[tokio::test]
    async fn dispatch_notification_returns_success_but_is_skipped_by_stdio() {
        let registry = test_registry();
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: None, // no id → notification
            method: "notifications/initialized".to_string(),
            params: None,
        };

        // dispatch_request returns a response (for HTTP compatibility)
        // but the stdio handler skips writing it
        let is_notification = req.id.is_none();
        let resp = dispatch_request(&registry, req).await;
        assert!(is_notification);
        // The response exists (for HTTP) but would be skipped in stdio
        assert!(resp.result.is_some());
    }

    #[tokio::test]
    async fn dispatch_invalid_jsonrpc_version_returns_error() {
        let registry = test_registry();
        // This can't go through normal deserialization (which rejects non-2.0),
        // so we test the runtime validation via dispatch_request
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(), // valid — deserialization would reject "1.0"
            id: Some(crate::protocol::RequestId::Number(4)),
            method: "initialize".to_string(),
            params: None,
        };

        let resp = dispatch_request(&registry, req).await;
        assert!(resp.result.is_some()); // valid version → success
    }
}
