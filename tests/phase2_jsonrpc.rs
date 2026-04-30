use serde_json::json;

/// Test: valid JSON-RPC 2.0 request deserializes correctly
#[test]
fn jsonrpc_valid_request_parses() {
    let raw = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/list",
        "params": {}
    });

    let req: obsidian_mcp::protocol::JsonRpcRequest =
        serde_json::from_value(raw).expect("should parse valid request");

    assert_eq!(req.jsonrpc, "2.0");
    assert_eq!(req.id, Some(1.into()));
    assert_eq!(req.method, "tools/list");
}

/// Test: JSON-RPC request with string id
#[test]
fn jsonrpc_request_with_string_id() {
    let raw = json!({
        "jsonrpc": "2.0",
        "id": "abc-123",
        "method": "tools/call",
        "params": { "name": "some_tool" }
    });

    let req: obsidian_mcp::protocol::JsonRpcRequest =
        serde_json::from_value(raw).expect("should parse request with string id");

    assert_eq!(req.id, Some("abc-123".into()));
}

/// Test: JSON-RPC request without params (params is optional)
#[test]
fn jsonrpc_request_without_params() {
    let raw = json!({
        "jsonrpc": "2.0",
        "id": 42,
        "method": "initialize"
    });

    let req: obsidian_mcp::protocol::JsonRpcRequest =
        serde_json::from_value(raw).expect("should parse request without params");

    assert_eq!(req.method, "initialize");
    assert!(req.params.is_none() || req.params.as_ref().map(|p| p.as_object().map(|o| o.is_empty()).unwrap_or(false)).unwrap_or(true));
}

/// Test: invalid JSON-RPC request (missing jsonrpc field) returns error
#[test]
fn jsonrpc_invalid_request_missing_version() {
    let raw = json!({
        "id": 1,
        "method": "tools/list"
    });

    let result: Result<obsidian_mcp::protocol::JsonRpcRequest, _> =
        serde_json::from_value(raw);

    assert!(result.is_err(), "should reject request missing jsonrpc version");
}

/// Test: invalid JSON-RPC request (wrong version) returns error
#[test]
fn jsonrpc_invalid_request_wrong_version() {
    let raw = json!({
        "jsonrpc": "1.0",
        "id": 1,
        "method": "tools/list"
    });

    let result: Result<obsidian_mcp::protocol::JsonRpcRequest, _> =
        serde_json::from_value(raw);

    assert!(result.is_err(), "should reject request with wrong jsonrpc version");
}

/// Test: JSON-RPC response serializes correctly
#[test]
fn jsonrpc_response_serializes() {
    let resp = obsidian_mcp::protocol::JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: 1.into(),
        result: Some(json!({"tools": []})),
        error: None,
    };

    let serialized = serde_json::to_value(&resp).expect("should serialize");
    assert_eq!(serialized["jsonrpc"], "2.0");
    assert_eq!(serialized["id"], 1);
    assert!(serialized.get("result").is_some());
    assert!(serialized.get("error").is_none());
}

/// Test: JSON-RPC error response serializes correctly
#[test]
fn jsonrpc_error_response_serializes() {
    let resp = obsidian_mcp::protocol::JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: 1.into(),
        result: None,
        error: Some(obsidian_mcp::protocol::JsonRpcError {
            code: -32601,
            message: "Method not found".to_string(),
            data: None,
        }),
    };

    let serialized = serde_json::to_value(&resp).expect("should serialize");
    assert_eq!(serialized["error"]["code"], -32601);
    assert_eq!(serialized["error"]["message"], "Method not found");
    assert!(serialized.get("result").is_none());
}

/// Test: JSON-RPC standard error codes are defined
#[test]
fn jsonrpc_standard_error_codes() {
    use obsidian_mcp::protocol::ErrorCode;
    assert_eq!(ErrorCode::ParseError as i32, -32700);
    assert_eq!(ErrorCode::InvalidRequest as i32, -32600);
    assert_eq!(ErrorCode::MethodNotFound as i32, -32601);
    assert_eq!(ErrorCode::InvalidParams as i32, -32602);
    assert_eq!(ErrorCode::InternalError as i32, -32603);
}
