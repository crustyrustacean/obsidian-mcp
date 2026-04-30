use crate::protocol::{ErrorCode, JsonRpcError};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

/// A tool handler function. Takes JSON params, returns a JSON result or a JSON-RPC error.
pub type ToolHandler = Arc<
    dyn Fn(Value) -> Pin<Box<dyn Future<Output = Result<Value, JsonRpcError>> + Send>>
        + Send
        + Sync,
>;

/// Descriptor for an MCP tool, as returned by tools/list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDescriptor {
    /// Tool name (e.g., "obsidian_read_note")
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// JSON Schema for the tool's input parameters
    pub input_schema: Value,
}

/// Registry of MCP tools. Supports registration, listing, and dispatch.
pub struct ToolRegistry {
    descriptors: Vec<ToolDescriptor>,
    handlers: HashMap<String, ToolHandler>,
}

impl ToolRegistry {
    /// Create an empty tool registry.
    pub fn new() -> Self {
        Self {
            descriptors: Vec::new(),
            handlers: HashMap::new(),
        }
    }

    /// Register a tool descriptor (no handler — will return MethodNotFound if called).
    pub fn register(&mut self, descriptor: ToolDescriptor) {
        self.descriptors.push(descriptor);
    }

    /// Register a tool with both a descriptor and a handler function.
    pub fn register_with_handler(&mut self, descriptor: ToolDescriptor, handler: ToolHandler) {
        self.handlers.insert(descriptor.name.clone(), handler);
        self.descriptors.push(descriptor);
    }

    /// List all registered tool descriptors.
    pub fn list(&self) -> &[ToolDescriptor] {
        &self.descriptors
    }

    /// Call a tool by name with the given arguments.
    ///
    /// Returns the tool's result on success, or a JSON-RPC error on failure.
    /// Only explicitly registered handlers can be called — arbitrary strings
    /// are never executed (sandboxing).
    pub async fn call(&self, name: &str, arguments: Value) -> Result<Value, JsonRpcError> {
        let handler = match self.handlers.get(name) {
            Some(h) => h,
            None => {
                return Err(JsonRpcError {
                    code: ErrorCode::MethodNotFound as i32,
                    message: format!("Tool not found: {name}"),
                    data: None,
                });
            }
        };

        handler(arguments).await
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper: format a tool result as MCP content array.
pub fn tool_result_content(text: &str) -> Value {
    serde_json::json!({
        "content": [
            {
                "type": "text",
                "text": text
            }
        ]
    })
}

/// Helper: format a tool error as MCP content array with isError flag.
pub fn tool_error_content(message: &str) -> Value {
    serde_json::json!({
        "content": [
            {
                "type": "text",
                "text": message
            }
        ],
        "isError": true
    })
}
