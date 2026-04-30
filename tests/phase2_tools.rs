use serde_json::{json, Value};
use std::sync::{Arc, Mutex};

/// Test: tools/list returns valid tool descriptors
#[test]
fn tool_registry_list_returns_descriptors() {
    let mut registry = obsidian_mcp::tools::ToolRegistry::new();

    registry.register(obsidian_mcp::tools::ToolDescriptor {
        name: "obsidian_read_note".to_string(),
        description: "Read a note from the vault".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "path": { "type": "string", "description": "Path to the note" }
            },
            "required": ["path"]
        }),
    });

    registry.register(obsidian_mcp::tools::ToolDescriptor {
        name: "obsidian_search".to_string(),
        description: "Search the vault".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "query": { "type": "string" }
            },
            "required": ["query"]
        }),
    });

    let tools = registry.list();
    assert_eq!(tools.len(), 2);

    // Each tool must have name, description, inputSchema
    for tool in tools {
        assert!(!tool.name.is_empty());
        assert!(!tool.description.is_empty());
        assert!(tool.input_schema.is_object());
    }

    let names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
    assert!(names.contains(&"obsidian_read_note"));
    assert!(names.contains(&"obsidian_search"));
}

/// Test: tools/call dispatches to registered tool
#[tokio::test]
async fn tool_registry_call_dispatches_to_handler() {
    let called = Arc::new(Mutex::new(false));
    let called_clone = called.clone();

    let handler: obsidian_mcp::tools::ToolHandler = Arc::new(move |_params: Value| {
        let called_clone = called_clone.clone();
        Box::pin(async move {
            *called_clone.lock().unwrap() = true;
            Ok(json!("tool result"))
        })
    });

    let mut registry = obsidian_mcp::tools::ToolRegistry::new();
    registry.register_with_handler(
        obsidian_mcp::tools::ToolDescriptor {
            name: "test_tool".to_string(),
            description: "A test tool".to_string(),
            input_schema: json!({"type": "object", "properties": {}}),
        },
        handler,
    );

    let result = registry.call("test_tool", json!({"foo": "bar"})).await;
    assert!(result.is_ok(), "tool call should succeed");
    assert!(*called.lock().unwrap(), "handler should have been invoked");
}

/// Test: tools/call with unknown tool name returns MethodNotFound
#[tokio::test]
async fn tool_registry_call_unknown_tool_returns_error() {
    let registry = obsidian_mcp::tools::ToolRegistry::new();

    let result = registry.call("nonexistent_tool", json!({})).await;
    assert!(result.is_err());

    let err = result.unwrap_err();
    // The error should be a JSON-RPC MethodNotFound (-32601)
    assert_eq!(err.code, -32601);
}

/// Test: tool registry sandboxing — cannot call unregistered tools
#[tokio::test]
async fn tool_registry_sandboxing() {
    let mut registry = obsidian_mcp::tools::ToolRegistry::new();

    let handler: obsidian_mcp::tools::ToolHandler = Arc::new(move |_params: Value| {
        Box::pin(async { Ok(json!("should not be called")) })
    });

    registry.register_with_handler(
        obsidian_mcp::tools::ToolDescriptor {
            name: "only_this_tool".to_string(),
            description: "The only registered tool".to_string(),
            input_schema: json!({"type": "object"}),
        },
        handler,
    );

    // Try calling something that looks like a function but isn't registered
    let result = registry.call("std::process::exit", json!({})).await;
    assert!(result.is_err(), "arbitrary string must not be callable");

    let result2 = registry.call("../../etc/passwd", json!({})).await;
    assert!(result2.is_err(), "path-like string must not be callable");
}
