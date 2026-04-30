use crate::client::ObsidianClient;
use crate::error::ObsidianError;
use crate::protocol::ErrorCode;
use crate::tools::{tool_result_content, ToolDescriptor, ToolRegistry};
use serde_json::{json, Value};
use std::sync::Arc;

/// Tool descriptor definitions for all 16 Obsidian MCP tools.
/// Each tool has its own explicit Accept/Content-Type headers —
/// never share a single header set across endpoints.

pub fn read_note_descriptor() -> ToolDescriptor {
    ToolDescriptor {
        name: "obsidian_read_note".to_string(),
        description: "Read a note from the Obsidian vault as markdown.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the note within the vault (e.g., 'notes/my-note.md')"
                }
            },
            "required": ["path"]
        }),
    }
}

pub fn read_note_metadata_descriptor() -> ToolDescriptor {
    ToolDescriptor {
        name: "obsidian_read_note_metadata".to_string(),
        description: "Read a note's metadata (frontmatter) as JSON.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the note within the vault"
                }
            },
            "required": ["path"]
        }),
    }
}

pub fn write_note_descriptor() -> ToolDescriptor {
    ToolDescriptor {
        name: "obsidian_write_note".to_string(),
        description: "Write (create or replace) a note in the Obsidian vault. The write is verified by reading the note back.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the note within the vault"
                },
                "content": {
                    "type": "string",
                    "description": "Markdown content to write"
                }
            },
            "required": ["path", "content"]
        }),
    }
}

pub fn append_note_descriptor() -> ToolDescriptor {
    ToolDescriptor {
        name: "obsidian_append_note".to_string(),
        description: "Append content to an existing note in the Obsidian vault.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the note within the vault"
                },
                "content": {
                    "type": "string",
                    "description": "Markdown content to append"
                }
            },
            "required": ["path", "content"]
        }),
    }
}

pub fn patch_note_descriptor() -> ToolDescriptor {
    ToolDescriptor {
        name: "obsidian_patch_note".to_string(),
        description: "Patch (update) a specific heading within a note.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the note within the vault"
                },
                "heading": {
                    "type": "string",
                    "description": "Heading to target (e.g., '## Section')"
                },
                "content": {
                    "type": "string",
                    "description": "Content to place under the heading"
                }
            },
            "required": ["path", "heading", "content"]
        }),
    }
}

pub fn delete_note_descriptor() -> ToolDescriptor {
    ToolDescriptor {
        name: "obsidian_delete_note".to_string(),
        description: "Delete a note from the Obsidian vault. Requires confirm=true to prevent accidental deletion.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the note within the vault"
                },
                "confirm": {
                    "type": "boolean",
                    "description": "Must be true to confirm deletion"
                }
            },
            "required": ["path", "confirm"]
        }),
    }
}

pub fn search_descriptor() -> ToolDescriptor {
    ToolDescriptor {
        name: "obsidian_search".to_string(),
        description: "Full-text search across the Obsidian vault.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Search query"
                }
            },
            "required": ["query"]
        }),
    }
}

pub fn dataview_query_descriptor() -> ToolDescriptor {
    ToolDescriptor {
        name: "obsidian_dataview_query".to_string(),
        description: "Execute a Dataview DQL query. SECURITY: The caller is responsible for query safety — Dataview queries can be resource-intensive.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "dql": {
                    "type": "string",
                    "description": "Dataview DQL query string"
                }
            },
            "required": ["dql"]
        }),
    }
}

pub fn jsonlogic_query_descriptor() -> ToolDescriptor {
    ToolDescriptor {
        name: "obsidian_jsonlogic_query".to_string(),
        description: "Execute a JSONLogic search query against the Obsidian vault.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "logic": {
                    "type": "object",
                    "description": "JSONLogic query object"
                }
            },
            "required": ["logic"]
        }),
    }
}

pub fn batch_read_descriptor() -> ToolDescriptor {
    ToolDescriptor {
        name: "obsidian_batch_read".to_string(),
        description: "Read multiple notes in parallel. Maximum 20 paths per call. Partial failures are reported per-path.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "paths": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Array of note paths to read (max 20)"
                }
            },
            "required": ["paths"]
        }),
    }
}

pub fn periodic_note_descriptor() -> ToolDescriptor {
    ToolDescriptor {
        name: "obsidian_periodic_note".to_string(),
        description: "Get a periodic note (daily, weekly, or monthly) from the Obsidian Periodic Notes plugin.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "period": {
                    "type": "string",
                    "enum": ["daily", "weekly", "monthly"],
                    "description": "Period type"
                }
            },
            "required": ["period"]
        }),
    }
}

pub fn recent_changes_descriptor() -> ToolDescriptor {
    ToolDescriptor {
        name: "obsidian_recent_changes".to_string(),
        description: "Get recently changed notes from the vault, sorted by modification time. Returns full paths (not relative).".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "limit": {
                    "type": "integer",
                    "description": "Maximum number of results (1-100, default 10)",
                    "default": 10
                }
            }
        }),
    }
}

pub fn list_directory_descriptor() -> ToolDescriptor {
    ToolDescriptor {
        name: "obsidian_list_directory".to_string(),
        description: "List the contents of a directory in the Obsidian vault.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Directory path within the vault (empty string for root)"
                }
            }
        }),
    }
}

pub fn get_tags_descriptor() -> ToolDescriptor {
    ToolDescriptor {
        name: "obsidian_get_tags".to_string(),
        description: "Get the tag hierarchy with counts from the Obsidian vault.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {}
        }),
    }
}

pub fn server_status_descriptor() -> ToolDescriptor {
    ToolDescriptor {
        name: "obsidian_server_status".to_string(),
        description: "Health check — test connectivity to the Obsidian Local REST API.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {}
        }),
    }
}

pub fn open_note_descriptor() -> ToolDescriptor {
    ToolDescriptor {
        name: "obsidian_open_note".to_string(),
        description: "Trigger the Obsidian UI to open a specific note.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the note to open"
                }
            },
            "required": ["path"]
        }),
    }
}

/// Convert an ObsidianError into a JSON-RPC error result value.
fn obsidian_error_to_jsonrpc(err: ObsidianError) -> crate::protocol::JsonRpcError {
    let code = match &err {
        ObsidianError::InvalidPath(_) => ErrorCode::InvalidParams as i32,
        ObsidianError::BatchLimitExceeded { .. } => ErrorCode::InvalidParams as i32,
        ObsidianError::RateLimitExceeded { .. } => ErrorCode::ServerError as i32,
        _ => ErrorCode::ServerError as i32,
    };

    crate::protocol::JsonRpcError {
        code,
        message: err.to_string(),
        data: Some(json!({"isError": true})),
    }
}

/// Register all 16 Obsidian tools with their handlers into the registry.
pub fn register_all_tools(registry: &mut ToolRegistry, client: Arc<ObsidianClient>) {
    // ── Read/Write Group (6) ──

    registry.register_with_handler(read_note_descriptor(), {
        let client = client.clone();
        Arc::new(move |args: Value| {
            let client = client.clone();
            Box::pin(async move {
                let path = args["path"]
                    .as_str()
                    .ok_or_else(|| crate::protocol::JsonRpcError {
                        code: ErrorCode::InvalidParams as i32,
                        message: "missing 'path' parameter".to_string(),
                        data: None,
                    })?;

                match client.read_note(path).await {
                    Ok(content) => Ok(tool_result_content(&content)),
                    Err(e) => Err(obsidian_error_to_jsonrpc(e)),
                }
            })
        })
    });

    registry.register_with_handler(read_note_metadata_descriptor(), {
        let client = client.clone();
        Arc::new(move |args: Value| {
            let client = client.clone();
            Box::pin(async move {
                let path = args["path"]
                    .as_str()
                    .ok_or_else(|| crate::protocol::JsonRpcError {
                        code: ErrorCode::InvalidParams as i32,
                        message: "missing 'path' parameter".to_string(),
                        data: None,
                    })?;

                match client.read_note_metadata(path).await {
                    Ok(metadata) => Ok(tool_result_content(&metadata.to_string())),
                    Err(e) => Err(obsidian_error_to_jsonrpc(e)),
                }
            })
        })
    });

    registry.register_with_handler(write_note_descriptor(), {
        let client = client.clone();
        Arc::new(move |args: Value| {
            let client = client.clone();
            Box::pin(async move {
                let path = args["path"]
                    .as_str()
                    .ok_or_else(|| crate::protocol::JsonRpcError {
                        code: ErrorCode::InvalidParams as i32,
                        message: "missing 'path' parameter".to_string(),
                        data: None,
                    })?;
                let content = args["content"]
                    .as_str()
                    .ok_or_else(|| crate::protocol::JsonRpcError {
                        code: ErrorCode::InvalidParams as i32,
                        message: "missing 'content' parameter".to_string(),
                        data: None,
                    })?;

                match client.write_note(path, content).await {
                    Ok(()) => Ok(tool_result_content(&format!(
                        "Note written successfully: {path}"
                    ))),
                    Err(e) => Err(obsidian_error_to_jsonrpc(e)),
                }
            })
        })
    });

    registry.register_with_handler(append_note_descriptor(), {
        let client = client.clone();
        Arc::new(move |args: Value| {
            let client = client.clone();
            Box::pin(async move {
                let path = args["path"]
                    .as_str()
                    .ok_or_else(|| crate::protocol::JsonRpcError {
                        code: ErrorCode::InvalidParams as i32,
                        message: "missing 'path' parameter".to_string(),
                        data: None,
                    })?;
                let content = args["content"]
                    .as_str()
                    .ok_or_else(|| crate::protocol::JsonRpcError {
                        code: ErrorCode::InvalidParams as i32,
                        message: "missing 'content' parameter".to_string(),
                        data: None,
                    })?;

                match client.append_note(path, content).await {
                    Ok(()) => Ok(tool_result_content(&format!(
                        "Content appended successfully: {path}"
                    ))),
                    Err(e) => Err(obsidian_error_to_jsonrpc(e)),
                }
            })
        })
    });

    registry.register_with_handler(patch_note_descriptor(), {
        let client = client.clone();
        Arc::new(move |args: Value| {
            let client = client.clone();
            Box::pin(async move {
                let path = args["path"]
                    .as_str()
                    .ok_or_else(|| crate::protocol::JsonRpcError {
                        code: ErrorCode::InvalidParams as i32,
                        message: "missing 'path' parameter".to_string(),
                        data: None,
                    })?;
                let heading = args["heading"]
                    .as_str()
                    .ok_or_else(|| crate::protocol::JsonRpcError {
                        code: ErrorCode::InvalidParams as i32,
                        message: "missing 'heading' parameter".to_string(),
                        data: None,
                    })?;
                let content = args["content"]
                    .as_str()
                    .ok_or_else(|| crate::protocol::JsonRpcError {
                        code: ErrorCode::InvalidParams as i32,
                        message: "missing 'content' parameter".to_string(),
                        data: None,
                    })?;

                match client.patch_note(path, heading, content).await {
                    Ok(()) => Ok(tool_result_content(&format!(
                        "Note patched successfully: {path} → {heading}"
                    ))),
                    Err(e) => Err(obsidian_error_to_jsonrpc(e)),
                }
            })
        })
    });

    registry.register_with_handler(delete_note_descriptor(), {
        let client = client.clone();
        Arc::new(move |args: Value| {
            let client = client.clone();
            Box::pin(async move {
                let path = args["path"]
                    .as_str()
                    .ok_or_else(|| crate::protocol::JsonRpcError {
                        code: ErrorCode::InvalidParams as i32,
                        message: "missing 'path' parameter".to_string(),
                        data: None,
                    })?;
                let confirm = args["confirm"].as_bool().unwrap_or(false);

                match client.delete_note(path, confirm).await {
                    Ok(()) => Ok(tool_result_content(&format!(
                        "Note deleted: {path}"
                    ))),
                    Err(e) => Err(obsidian_error_to_jsonrpc(e)),
                }
            })
        })
    });

    // ── Search Group (3) ──

    registry.register_with_handler(search_descriptor(), {
        let client = client.clone();
        Arc::new(move |args: Value| {
            let client = client.clone();
            Box::pin(async move {
                let query = args["query"]
                    .as_str()
                    .ok_or_else(|| crate::protocol::JsonRpcError {
                        code: ErrorCode::InvalidParams as i32,
                        message: "missing 'query' parameter".to_string(),
                        data: None,
                    })?;

                match client.search(query).await {
                    Ok(results) => Ok(tool_result_content(&serde_json::to_string_pretty(&results).unwrap_or_default())),
                    Err(e) => Err(obsidian_error_to_jsonrpc(e)),
                }
            })
        })
    });

    registry.register_with_handler(dataview_query_descriptor(), {
        let client = client.clone();
        Arc::new(move |args: Value| {
            let client = client.clone();
            Box::pin(async move {
                let dql = args["dql"]
                    .as_str()
                    .ok_or_else(|| crate::protocol::JsonRpcError {
                        code: ErrorCode::InvalidParams as i32,
                        message: "missing 'dql' parameter".to_string(),
                        data: None,
                    })?;

                match client.dataview_query(dql).await {
                    Ok(results) => Ok(tool_result_content(&serde_json::to_string_pretty(&results).unwrap_or_default())),
                    Err(e) => Err(obsidian_error_to_jsonrpc(e)),
                }
            })
        })
    });

    registry.register_with_handler(jsonlogic_query_descriptor(), {
        let client = client.clone();
        Arc::new(move |args: Value| {
            let client = client.clone();
            Box::pin(async move {
                let logic = args.get("logic").cloned().ok_or_else(|| {
                    crate::protocol::JsonRpcError {
                        code: ErrorCode::InvalidParams as i32,
                        message: "missing 'logic' parameter".to_string(),
                        data: None,
                    }
                })?;

                match client.jsonlogic_query(&logic).await {
                    Ok(results) => Ok(tool_result_content(&serde_json::to_string_pretty(&results).unwrap_or_default())),
                    Err(e) => Err(obsidian_error_to_jsonrpc(e)),
                }
            })
        })
    });

    // ── Batch/Convenience Group (3) ──

    registry.register_with_handler(batch_read_descriptor(), {
        let client = client.clone();
        Arc::new(move |args: Value| {
            let client = client.clone();
            Box::pin(async move {
                let paths_array = args["paths"]
                    .as_array()
                    .ok_or_else(|| crate::protocol::JsonRpcError {
                        code: ErrorCode::InvalidParams as i32,
                        message: "missing or invalid 'paths' array parameter".to_string(),
                        data: None,
                    })?;

                let paths: Vec<String> = paths_array
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();

                if paths.is_empty() {
                    return Err(crate::protocol::JsonRpcError {
                        code: ErrorCode::InvalidParams as i32,
                        message: "'paths' array must not be empty".to_string(),
                        data: None,
                    });
                }

                match client.batch_read(&paths).await {
                    Ok(map) => {
                        let result: Value = map.into_iter().map(|(k, v)| (k, json!(v))).collect();
                        Ok(tool_result_content(&serde_json::to_string_pretty(&result).unwrap_or_default()))
                    }
                    Err(e) => Err(obsidian_error_to_jsonrpc(e)),
                }
            })
        })
    });

    registry.register_with_handler(periodic_note_descriptor(), {
        let client = client.clone();
        Arc::new(move |args: Value| {
            let client = client.clone();
            Box::pin(async move {
                let period = args["period"]
                    .as_str()
                    .ok_or_else(|| crate::protocol::JsonRpcError {
                        code: ErrorCode::InvalidParams as i32,
                        message: "missing 'period' parameter".to_string(),
                        data: None,
                    })?;

                match client.periodic_note(period).await {
                    Ok(content) => Ok(tool_result_content(&content)),
                    Err(e) => Err(obsidian_error_to_jsonrpc(e)),
                }
            })
        })
    });

    registry.register_with_handler(recent_changes_descriptor(), {
        let client = client.clone();
        Arc::new(move |args: Value| {
            let client = client.clone();
            Box::pin(async move {
                let limit = args["limit"].as_u64().unwrap_or(10) as usize;

                match client.recent_changes(limit).await {
                    Ok(results) => Ok(tool_result_content(&serde_json::to_string_pretty(&results).unwrap_or_default())),
                    Err(e) => Err(obsidian_error_to_jsonrpc(e)),
                }
            })
        })
    });

    // ── Navigate Group (4) ──

    registry.register_with_handler(list_directory_descriptor(), {
        let client = client.clone();
        Arc::new(move |args: Value| {
            let client = client.clone();
            Box::pin(async move {
                let path = args["path"].as_str().unwrap_or("");

                match client.list_directory(path).await {
                    Ok(listing) => Ok(tool_result_content(&serde_json::to_string_pretty(&listing).unwrap_or_default())),
                    Err(e) => Err(obsidian_error_to_jsonrpc(e)),
                }
            })
        })
    });

    registry.register_with_handler(get_tags_descriptor(), {
        let client = client.clone();
        Arc::new(move |args: Value| {
            let client = client.clone();
            let _ = args; // no parameters
            Box::pin(async move {
                match client.get_tags().await {
                    Ok(tags) => Ok(tool_result_content(&serde_json::to_string_pretty(&tags).unwrap_or_default())),
                    Err(e) => Err(obsidian_error_to_jsonrpc(e)),
                }
            })
        })
    });

    registry.register_with_handler(server_status_descriptor(), {
        let client = client.clone();
        Arc::new(move |args: Value| {
            let client = client.clone();
            let _ = args;
            Box::pin(async move {
                match client.server_status().await {
                    Ok(()) => Ok(tool_result_content("Obsidian API is reachable and healthy")),
                    Err(e) => Err(obsidian_error_to_jsonrpc(e)),
                }
            })
        })
    });

    registry.register_with_handler(open_note_descriptor(), {
        let client = client.clone();
        Arc::new(move |args: Value| {
            let client = client.clone();
            Box::pin(async move {
                let path = args["path"]
                    .as_str()
                    .ok_or_else(|| crate::protocol::JsonRpcError {
                        code: ErrorCode::InvalidParams as i32,
                        message: "missing 'path' parameter".to_string(),
                        data: None,
                    })?;

                match client.open_note(path).await {
                    Ok(()) => Ok(tool_result_content(&format!("Opened note: {path}"))),
                    Err(e) => Err(obsidian_error_to_jsonrpc(e)),
                }
            })
        })
    });
}
