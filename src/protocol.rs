use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

/// JSON-RPC 2.0 standard error codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    ParseError = -32700,
    InvalidRequest = -32600,
    MethodNotFound = -32601,
    InvalidParams = -32602,
    InternalError = -32603,
    // Server-defined errors start at -32000
    ServerError = -32000,
}

/// JSON-RPC 2.0 request ID — can be a number, string, or null.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RequestId {
    Number(i64),
    String(String),
    Null,
}

impl From<i64> for RequestId {
    fn from(n: i64) -> Self {
        RequestId::Number(n)
    }
}

impl From<&str> for RequestId {
    fn from(s: &str) -> Self {
        RequestId::String(s.to_string())
    }
}

/// JSON-RPC 2.0 request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    /// Must always be "2.0" — rejected at deserialization if not.
    #[serde(deserialize_with = "deserialize_jsonrpc_version")]
    pub jsonrpc: String,
    /// Request identifier
    pub id: Option<RequestId>,
    /// Method name
    pub method: String,
    /// Method parameters (optional)
    pub params: Option<Value>,
}

/// Custom deserializer: reject any jsonrpc field that isn't exactly "2.0".
fn deserialize_jsonrpc_version<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s != "2.0" {
        return Err(serde::de::Error::custom(
            "invalid jsonrpc version: must be \"2.0\"",
        ));
    }
    Ok(s)
}

/// JSON-RPC 2.0 error object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    /// Error code
    pub code: i32,
    /// Human-readable error message
    pub message: String,
    /// Additional error data (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// JSON-RPC 2.0 response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    /// Must always be "2.0"
    pub jsonrpc: String,
    /// Request identifier (echoed back)
    pub id: RequestId,
    /// Result (present on success)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    /// Error (present on failure)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

impl JsonRpcResponse {
    /// Create a successful response
    pub fn success(id: RequestId, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    /// Create an error response
    pub fn error(id: RequestId, code: i32, message: String, data: Option<Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code,
                message,
                data,
            }),
        }
    }

    /// Create a MethodNotFound error response
    pub fn method_not_found(id: RequestId, method: &str) -> Self {
        Self::error(
            id,
            ErrorCode::MethodNotFound as i32,
            format!("Method not found: {method}"),
            None,
        )
    }

    /// Create a ParseError response
    pub fn parse_error(id: Option<RequestId>) -> Self {
        Self::error(
            id.unwrap_or(RequestId::Null),
            ErrorCode::ParseError as i32,
            "Parse error".to_string(),
            None,
        )
    }

    /// Create an InvalidRequest error response
    pub fn invalid_request(id: Option<RequestId>) -> Self {
        Self::error(
            id.unwrap_or(RequestId::Null),
            ErrorCode::InvalidRequest as i32,
            "Invalid request".to_string(),
            None,
        )
    }

    /// Create an InvalidParams error response
    pub fn invalid_params(id: RequestId, detail: &str) -> Self {
        Self::error(
            id,
            ErrorCode::InvalidParams as i32,
            format!("Invalid params: {detail}"),
            None,
        )
    }

    /// Create an InternalError response
    pub fn internal_error(id: RequestId, detail: &str) -> Self {
        Self::error(
            id,
            ErrorCode::InternalError as i32,
            format!("Internal error: {detail}"),
            None,
        )
    }
}

/// Validate that a JsonRpcRequest has the required "2.0" version field.
/// Note: this is also enforced at deserialization time via `deserialize_jsonrpc_version`.
/// This function serves as a runtime defense-in-depth check.
#[allow(clippy::result_large_err)]
pub fn validate_request(req: &JsonRpcRequest) -> Result<(), JsonRpcResponse> {
    if req.jsonrpc != "2.0" {
        return Err(JsonRpcResponse::invalid_request(req.id.clone()));
    }
    Ok(())
}
