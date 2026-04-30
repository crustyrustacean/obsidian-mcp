//! Phase 3 tests — Core Tools (16 tools)
//!
//! Each test uses wiremock to mock the Obsidian Local REST API and verify:
//! - Correct HTTP method (GET/PUT/POST/PATCH/DELETE)
//! - Correct per-tool Accept/Content-Type headers
//! - Correct request/response shape
//! - Security validations (path traversal, batch limits, delete confirm)
//! - Write verification (read-back after write)

use obsidian_mcp::client::ObsidianClient;
use obsidian_mcp::config::Config;
use obsidian_mcp::error::ObsidianError;
use obsidian_mcp::tools::ToolRegistry;
use obsidian_mcp::tools_impl;
use serde_json::json;
use std::sync::Arc;
use wiremock::matchers::{
    body_json, body_string, header, method, path,
};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Helper: create a Config pointing at the mock server.
fn mock_config(server: &MockServer) -> Config {
    Config {
        api_key: "test-api-key".to_string(),
        api_url: server.uri(),
    }
}

/// Helper: create a client wired to the mock server.
fn mock_client(server: &MockServer) -> Arc<ObsidianClient> {
    Arc::new(ObsidianClient::new(mock_config(server)))
}

// ════════════════════════════════════════════════════════════════
// Read/Write Group (6 tools)
// ════════════════════════════════════════════════════════════════

#[tokio::test]
async fn read_note_sends_accept_text_markdown() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/vault/notes%2Ftest.md"))
        .and(header("Accept", "text/markdown"))
        .and(header("Authorization", "Bearer test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_string("# Hello"))
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let result = client.read_note("notes/test.md").await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "# Hello");
}

#[tokio::test]
async fn read_note_metadata_sends_accept_application_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/vault/notes%2Ftest.md"))
        .and(header("Accept", "application/json"))
        .and(header("Authorization", "Bearer test-api-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({
                "frontmatter": { "tags": ["test"] },
                "content": "# Hello"
            })),
        )
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let result = client.read_note_metadata("notes/test.md").await;
    assert!(result.is_ok());
    let metadata = result.unwrap();
    assert!(metadata.is_object());
}

#[tokio::test]
async fn write_note_sends_content_type_text_markdown() {
    let server = MockServer::start().await;
    // Mock the PUT request
    Mock::given(method("PUT"))
        .and(path("/vault/notes%2Fnew.md"))
        .and(header("Content-Type", "text/markdown"))
        .and(header("Authorization", "Bearer test-api-key"))
        .and(body_string("# New Note"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    // Mock the read-back verification (read_note)
    Mock::given(method("GET"))
        .and(path("/vault/notes%2Fnew.md"))
        .and(header("Accept", "text/markdown"))
        .respond_with(ResponseTemplate::new(200).set_body_string("# New Note"))
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let result = client.write_note("notes/new.md", "# New Note").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn write_note_verifies_write_readback() {
    let server = MockServer::start().await;

    // Mock the PUT (succeeds)
    Mock::given(method("PUT"))
        .and(path("/vault/notes%2Fempty.md"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    // Mock the read-back — returns empty content (triggers verification failure)
    Mock::given(method("GET"))
        .and(path("/vault/notes%2Fempty.md"))
        .respond_with(ResponseTemplate::new(200).set_body_string(""))
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let result = client.write_note("notes/empty.md", "some content").await;
    assert!(result.is_err());
    match result.unwrap_err() {
        ObsidianError::WriteVerificationFailed { path } => {
            assert_eq!(path, "notes/empty.md");
        }
        e => panic!("expected WriteVerificationFailed, got: {e}"),
    }
}

#[tokio::test]
async fn append_note_sends_post() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/vault/notes%2Fexisting.md"))
        .and(header("Content-Type", "text/markdown"))
        .and(body_string("\nAppended content"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    // Mock the read-back verification
    Mock::given(method("GET"))
        .and(path("/vault/notes%2Fexisting.md"))
        .respond_with(ResponseTemplate::new(200).set_body_string("Original\nAppended content"))
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let result = client.append_note("notes/existing.md", "\nAppended content").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn patch_note_targets_heading() {
    let server = MockServer::start().await;
    Mock::given(method("PATCH"))
        .and(path("/vault/notes%2Fnote.md"))
        .and(header("Content-Type", "text/markdown"))
        .and(header("Heading", "## Section"))
        .and(body_string("Updated section content"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    // Mock the read-back verification
    Mock::given(method("GET"))
        .and(path("/vault/notes%2Fnote.md"))
        .respond_with(ResponseTemplate::new(200).set_body_string("# Note\n## Section\nUpdated section content"))
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let result = client.patch_note("notes/note.md", "## Section", "Updated section content").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn delete_note_sends_delete() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/vault/notes%2Fold.md"))
        .and(header("Authorization", "Bearer test-api-key"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let result = client.delete_note("notes/old.md", true).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn delete_note_requires_confirm() {
    let server = MockServer::start().await;
    let client = mock_client(&server);

    // No mock needed — should fail before making any request
    let result = client.delete_note("notes/old.md", false).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        ObsidianError::DeleteConfirmationRequired => {}
        e => panic!("expected DeleteConfirmationRequired, got: {e}"),
    }
}

// ════════════════════════════════════════════════════════════════
// Search Group (3 tools)
// ════════════════════════════════════════════════════════════════

#[tokio::test]
async fn search_encodes_query() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/search/hello%20world"))
        .and(header("Accept", "application/json"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!([
                { "path": "notes/hello.md", "matches": [{ "match": "hello world" }] }
            ])),
        )
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let result = client.search("hello world").await;
    assert!(result.is_ok());
    let results = result.unwrap();
    assert_eq!(results.len(), 1);
}

#[tokio::test]
async fn dataview_query_sends_correct_content_type() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/search/"))
        .and(header("Content-Type", "application/vnd.olrapi.dataview.dql+txt"))
        .and(body_string("TABLE file.name FROM \"notes\""))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!([
                { "file": { "name": "note1" } }
            ])),
        )
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let result = client.dataview_query("TABLE file.name FROM \"notes\"").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn jsonlogic_query_sends_json_body() {
    let server = MockServer::start().await;
    let logic = json!({
        "and": [
            { "in": ["tag1", { "var": "file.tags" }] }
        ]
    });

    Mock::given(method("POST"))
        .and(path("/search/"))
        .and(header("Content-Type", "application/vnd.olrapi.jsonlogic+json"))
        .and(body_json(&logic))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!([
                { "path": "notes/tagged.md" }
            ])),
        )
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let result = client.jsonlogic_query(&logic).await;
    assert!(result.is_ok());
}

// ════════════════════════════════════════════════════════════════
// Batch/Convenience Group (3 tools)
// ════════════════════════════════════════════════════════════════

#[tokio::test]
async fn batch_read_parallel_reads() {
    let server = MockServer::start().await;

    // Mock multiple note reads
    Mock::given(method("GET"))
        .and(path("/vault/a.md"))
        .respond_with(ResponseTemplate::new(200).set_body_string("Content A"))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/vault/b.md"))
        .respond_with(ResponseTemplate::new(200).set_body_string("Content B"))
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let result = client
        .batch_read(&["a.md".to_string(), "b.md".to_string()])
        .await;
    assert!(result.is_ok());

    let map = result.unwrap();
    assert_eq!(map.get("a.md").unwrap(), "Content A");
    assert_eq!(map.get("b.md").unwrap(), "Content B");
}

#[tokio::test]
async fn batch_read_partial_failure() {
    let server = MockServer::start().await;

    // One path succeeds
    Mock::given(method("GET"))
        .and(path("/vault/exists.md"))
        .respond_with(ResponseTemplate::new(200).set_body_string("I exist"))
        .mount(&server)
        .await;

    // Another path fails
    Mock::given(method("GET"))
        .and(path("/vault/missing.md"))
        .respond_with(ResponseTemplate::new(404).set_body_string("Not found"))
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let result = client
        .batch_read(&["exists.md".to_string(), "missing.md".to_string()])
        .await;
    assert!(result.is_ok());

    let map = result.unwrap();
    assert_eq!(map.get("exists.md").unwrap(), "I exist");
    // Partial failure should be reported with [ERROR] prefix
    let missing_result = map.get("missing.md").unwrap();
    assert!(missing_result.starts_with("[ERROR]"));
}

#[tokio::test]
async fn batch_read_limit_exceeded() {
    let server = MockServer::start().await;
    let client = mock_client(&server);

    let paths: Vec<String> = (0..25).map(|i| format!("note{i}.md")).collect();
    let result = client.batch_read(&paths).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        ObsidianError::BatchLimitExceeded { requested, max } => {
            assert_eq!(requested, 25);
            assert_eq!(max, 20);
        }
        e => panic!("expected BatchLimitExceeded, got: {e}"),
    }
}

#[tokio::test]
async fn periodic_note_routes_by_period() {
    let server = MockServer::start().await;

    // Mock daily
    Mock::given(method("GET"))
        .and(path("/periodic/daily"))
        .and(header("Accept", "text/markdown"))
        .respond_with(ResponseTemplate::new(200).set_body_string("# Daily Note"))
        .mount(&server)
        .await;

    // Mock weekly
    Mock::given(method("GET"))
        .and(path("/periodic/weekly"))
        .and(header("Accept", "text/markdown"))
        .respond_with(ResponseTemplate::new(200).set_body_string("# Weekly Note"))
        .mount(&server)
        .await;

    // Mock monthly
    Mock::given(method("GET"))
        .and(path("/periodic/monthly"))
        .and(header("Accept", "text/markdown"))
        .respond_with(ResponseTemplate::new(200).set_body_string("# Monthly Note"))
        .mount(&server)
        .await;

    let client = mock_client(&server);

    let daily = client.periodic_note("daily").await.unwrap();
    assert!(daily.contains("Daily Note"));

    let weekly = client.periodic_note("weekly").await.unwrap();
    assert!(weekly.contains("Weekly Note"));

    let monthly = client.periodic_note("monthly").await.unwrap();
    assert!(monthly.contains("Monthly Note"));
}

#[tokio::test]
async fn periodic_note_rejects_invalid_period() {
    let server = MockServer::start().await;
    let client = mock_client(&server);

    let result = client.periodic_note("yearly").await;
    assert!(result.is_err());
    match result.unwrap_err() {
        ObsidianError::InvalidPath(msg) => {
            assert!(msg.contains("yearly"));
        }
        e => panic!("expected InvalidPath, got: {e}"),
    }
}

#[tokio::test]
async fn recent_changes_returns_results() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/vault/"))
        .and(header("Accept", "application/json"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({
                "files": [
                    { "path": "notes/recent.md", "type": "file" },
                    { "path": "notes/old.md", "type": "file" }
                ]
            })),
        )
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let result = client.recent_changes(10).await;
    assert!(result.is_ok());
    let changes = result.unwrap();
    assert!(!changes.is_empty());
}

// ════════════════════════════════════════════════════════════════
// Navigate Group (4 tools)
// ════════════════════════════════════════════════════════════════

#[tokio::test]
async fn list_directory_returns_file_folder_list() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/vault/research/"))
        .and(header("Accept", "application/json"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({
                "files": [
                    { "name": "topic1.md", "type": "file" },
                    { "name": "subfolder", "type": "folder" }
                ]
            })),
        )
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let result = client.list_directory("research").await;
    assert!(result.is_ok());

    let listing = result.unwrap();
    let files = listing.get("files").unwrap().as_array().unwrap();
    assert_eq!(files.len(), 2);
}

#[tokio::test]
async fn list_directory_root() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/vault/"))
        .and(header("Accept", "application/json"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({
                "files": [{ "name": "readme.md", "type": "file" }]
            })),
        )
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let result = client.list_directory("").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn get_tags_returns_hierarchy() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/vault-tags/"))
        .and(header("Accept", "application/json"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({
                "tags": [
                    { "name": "project/my-rust-project", "count": 5 },
                    { "name": "type/session-log", "count": 12 }
                ]
            })),
        )
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let result = client.get_tags().await;
    assert!(result.is_ok());

    let tags = result.unwrap();
    let tag_list = tags.get("tags").unwrap().as_array().unwrap();
    assert_eq!(tag_list.len(), 2);
}

#[tokio::test]
async fn server_status_health_check() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/"))
        .and(header("Authorization", "Bearer test-api-key"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let result = client.server_status().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn server_status_unhealthy_returns_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(401).set_body_string("Unauthorized"))
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let result = client.server_status().await;
    assert!(result.is_err());
    match result.unwrap_err() {
        ObsidianError::ApiError { status, .. } => {
            assert_eq!(status, 401);
        }
        e => panic!("expected ApiError, got: {e}"),
    }
}

#[tokio::test]
async fn open_note_triggers_ui_open() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/open/notes%2Fmy-note.md"))
        .and(header("Authorization", "Bearer test-api-key"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let result = client.open_note("notes/my-note.md").await;
    assert!(result.is_ok());
}

// ════════════════════════════════════════════════════════════════
// Security: Path validation
// ════════════════════════════════════════════════════════════════

#[tokio::test]
async fn read_note_rejects_path_traversal() {
    let server = MockServer::start().await;
    let client = mock_client(&server);

    let result = client.read_note("../../etc/passwd").await;
    assert!(result.is_err());
    match result.unwrap_err() {
        ObsidianError::InvalidPath(msg) => {
            assert!(msg.contains("traversal"));
        }
        e => panic!("expected InvalidPath, got: {e}"),
    }
}

#[tokio::test]
async fn write_note_rejects_absolute_path() {
    let server = MockServer::start().await;
    let client = mock_client(&server);

    let result = client.write_note("/etc/passwd", "bad").await;
    assert!(result.is_err());
    match result.unwrap_err() {
        ObsidianError::InvalidPath(msg) => {
            assert!(msg.contains("absolute"));
        }
        e => panic!("expected InvalidPath, got: {e}"),
    }
}

#[tokio::test]
async fn open_note_rejects_null_bytes() {
    let server = MockServer::start().await;
    let client = mock_client(&server);

    let result = client.open_note("file\0.md").await;
    assert!(result.is_err());
    match result.unwrap_err() {
        ObsidianError::InvalidPath(msg) => {
            assert!(msg.contains("null"));
        }
        e => panic!("expected InvalidPath, got: {e}"),
    }
}

#[tokio::test]
async fn delete_note_rejects_path_traversal() {
    let server = MockServer::start().await;
    let client = mock_client(&server);

    let result = client.delete_note("../../secret", true).await;
    assert!(result.is_err());
}

// ════════════════════════════════════════════════════════════════
// Tool Registry: all 16 tools registered
// ════════════════════════════════════════════════════════════════

#[tokio::test]
async fn all_16_tools_registered() {
    let server = MockServer::start().await;
    let client = mock_client(&server);

    let mut registry = ToolRegistry::new();
    tools_impl::register_all_tools(&mut registry, client);

    let tools = registry.list();
    assert_eq!(tools.len(), 16, "expected 16 tools, got {}", tools.len());

    let expected_names = [
        "obsidian_read_note",
        "obsidian_read_note_metadata",
        "obsidian_write_note",
        "obsidian_append_note",
        "obsidian_patch_note",
        "obsidian_delete_note",
        "obsidian_search",
        "obsidian_dataview_query",
        "obsidian_jsonlogic_query",
        "obsidian_batch_read",
        "obsidian_periodic_note",
        "obsidian_recent_changes",
        "obsidian_list_directory",
        "obsidian_get_tags",
        "obsidian_server_status",
        "obsidian_open_note",
    ];

    let registered_names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
    for name in &expected_names {
        assert!(
            registered_names.contains(name),
            "tool '{name}' not found in registry"
        );
    }
}

#[tokio::test]
async fn tool_descriptors_have_required_fields() {
    let server = MockServer::start().await;
    let client = mock_client(&server);

    let mut registry = ToolRegistry::new();
    tools_impl::register_all_tools(&mut registry, client);

    for tool in registry.list() {
        assert!(!tool.name.is_empty(), "tool name must not be empty");
        assert!(!tool.description.is_empty(), "tool description must not be empty");
        assert!(
            tool.input_schema.is_object(),
            "tool {} input_schema must be an object",
            tool.name
        );
        // Every tool must have a type: object in its schema
        assert_eq!(
            tool.input_schema.get("type").and_then(|t| t.as_str()),
            Some("object"),
            "tool {} inputSchema must have type: object",
            tool.name
        );
    }
}

// ════════════════════════════════════════════════════════════════
// Tool call dispatch via registry
// ════════════════════════════════════════════════════════════════

#[tokio::test]
async fn tool_call_read_note_dispatches_correctly() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/vault/test.md"))
        .and(header("Accept", "text/markdown"))
        .respond_with(ResponseTemplate::new(200).set_body_string("Hello from vault"))
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let mut registry = ToolRegistry::new();
    tools_impl::register_all_tools(&mut registry, client);

    let result = registry
        .call("obsidian_read_note", json!({"path": "test.md"}))
        .await;
    assert!(result.is_ok());

    let response = result.unwrap();
    let content = response["content"][0]["text"].as_str().unwrap();
    assert_eq!(content, "Hello from vault");
}

#[tokio::test]
async fn tool_call_server_status_returns_healthy() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let mut registry = ToolRegistry::new();
    tools_impl::register_all_tools(&mut registry, client);

    let result = registry
        .call("obsidian_server_status", json!({}))
        .await;
    assert!(result.is_ok());

    let response = result.unwrap();
    let text = response["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("healthy") || text.contains("reachable"));
}

#[tokio::test]
async fn tool_call_delete_without_confirm_returns_error() {
    let server = MockServer::start().await;

    let client = mock_client(&server);
    let mut registry = ToolRegistry::new();
    tools_impl::register_all_tools(&mut registry, client);

    let result = registry
        .call("obsidian_delete_note", json!({"path": "test.md", "confirm": false}))
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn tool_call_missing_required_param_returns_error() {
    let server = MockServer::start().await;

    let client = mock_client(&server);
    let mut registry = ToolRegistry::new();
    tools_impl::register_all_tools(&mut registry, client);

    let result = registry
        .call("obsidian_read_note", json!({}))
        .await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code, -32602); // InvalidParams
}

// ════════════════════════════════════════════════════════════════
// Error handling: API errors are surfaced, not panicked
// ════════════════════════════════════════════════════════════════

#[tokio::test]
async fn read_note_404_returns_api_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/vault/nonexistent.md"))
        .respond_with(ResponseTemplate::new(404).set_body_string("Not found"))
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let result = client.read_note("nonexistent.md").await;
    assert!(result.is_err());
    match result.unwrap_err() {
        ObsidianError::ApiError { status, .. } => {
            assert_eq!(status, 404);
        }
        e => panic!("expected ApiError, got: {e}"),
    }
}

#[tokio::test]
async fn write_note_500_returns_api_error() {
    let server = MockServer::start().await;

    Mock::given(method("PUT"))
        .and(path("/vault/fail.md"))
        .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let result = client.write_note("fail.md", "content").await;
    assert!(result.is_err());
    match result.unwrap_err() {
        ObsidianError::ApiError { status, .. } => {
            assert_eq!(status, 500);
        }
        e => panic!("expected ApiError, got: {e}"),
    }
}

// ════════════════════════════════════════════════════════════════
// URL encoding: paths are URL-encoded in request URLs
// ════════════════════════════════════════════════════════════════

#[tokio::test]
async fn paths_with_slashes_are_url_encoded() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/vault/notes%2Fdeep%2Ftopic.md"))
        .and(header("Accept", "text/markdown"))
        .respond_with(ResponseTemplate::new(200).set_body_string("Deep content"))
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let result = client.read_note("notes/deep/topic.md").await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Deep content");
}

#[tokio::test]
async fn open_note_url_encodes_path() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/open/notes%2Fmy-note.md"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let result = client.open_note("notes/my-note.md").await;
    assert!(result.is_ok());
}

// ════════════════════════════════════════════════════════════════
// Delete confirmation uses dedicated error variant
// ════════════════════════════════════════════════════════════════

#[tokio::test]
async fn delete_without_confirm_returns_dedicated_error() {
    let server = MockServer::start().await;
    let client = mock_client(&server);

    let result = client.delete_note("test.md", false).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        ObsidianError::DeleteConfirmationRequired => {}
        e => panic!("expected DeleteConfirmationRequired, got: {e}"),
    }
}

// ════════════════════════════════════════════════════════════════
// Query length limits
// ════════════════════════════════════════════════════════════════

#[tokio::test]
async fn search_query_too_long_returns_error() {
    let server = MockServer::start().await;
    let client = mock_client(&server);

    let long_query = "a".repeat(1001);
    let result = client.search(&long_query).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        ObsidianError::ContentTooLong(param, max) => {
            assert_eq!(param, "query");
            assert_eq!(max, 1000);
        }
        e => panic!("expected ContentTooLong, got: {e}"),
    }
}

#[tokio::test]
async fn dataview_query_too_long_returns_error() {
    let server = MockServer::start().await;
    let client = mock_client(&server);

    let long_dql = "a".repeat(1001);
    let result = client.dataview_query(&long_dql).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        ObsidianError::ContentTooLong(param, _) => {
            assert_eq!(param, "dql");
        }
        e => panic!("expected ContentTooLong, got: {e}"),
    }
}

// ════════════════════════════════════════════════════════════════
// Rate limiting
// ════════════════════════════════════════════════════════════════

#[test]
fn rate_limiter_allows_initial_requests() {
    let limiter = obsidian_mcp::rate_limiter::ToolRateLimiter::new();
    assert!(limiter.check("test_tool").is_ok());
}

#[test]
fn rate_limiter_per_tool_independence() {
    let limiter = obsidian_mcp::rate_limiter::ToolRateLimiter::new();
    // Exhaust tool_a's burst
    for _ in 0..10 {
        let _ = limiter.check("tool_a");
    }
    // tool_b should still be allowed
    assert!(limiter.check("tool_b").is_ok());
}
