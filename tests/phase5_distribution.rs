//! Phase 5 tests — Distribution & Hardening
//!
//! Tests for graceful degradation, session audit, config validation,
//! binary size, and reliability features.

use obsidian_mcp::audit::audit_session_end;
use obsidian_mcp::client::ObsidianClient;
use obsidian_mcp::config::Config;
use obsidian_mcp::error::ObsidianError;
use obsidian_mcp::tools::ToolRegistry;
use obsidian_mcp::tools_impl;
use serde_json::json;
use std::sync::Arc;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn mock_config(server: &MockServer) -> Config {
    Config {
        api_key: "test-api-key".to_string(),
        api_url: server.uri(),
    }
}

fn mock_client(server: &MockServer) -> Arc<ObsidianClient> {
    Arc::new(ObsidianClient::new(mock_config(server)))
}

// ════════════════════════════════════════════════════════════════
// Graceful Degradation
// ════════════════════════════════════════════════════════════════

#[tokio::test]
async fn obsidian_unreachable_returns_structured_error() {
    // Point to a port with nothing listening
    let config = Config {
        api_key: "test-key".to_string(),
        api_url: "http://127.0.0.1:1".to_string(), // port 1 — nothing there
    };
    let client = ObsidianClient::new(config);

    let result = client.read_note("test.md").await;
    assert!(result.is_err(), "should fail when Obsidian is unreachable");

    // Verify it's a ConnectionError, not a panic
    match result.unwrap_err() {
        ObsidianError::ConnectionError { .. } => {}
        e => panic!("expected ConnectionError, got: {e}"),
    }
}

#[tokio::test]
async fn obsidian_timeout_returns_structured_error() {
    let server = MockServer::start().await;

    // Simulate a slow response (10 seconds) — client timeout is 30s so we use
    // a direct delay approach by having the mock respond very slowly
    Mock::given(method("GET"))
        .and(path("/vault/slow.md"))
        .respond_with(ResponseTemplate::new(200).set_delay(std::time::Duration::from_secs(60)))
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let result = client.read_note("slow.md").await;
    assert!(result.is_err(), "should timeout on slow response");
}

#[tokio::test]
async fn recovery_after_transient_failure() {
    let server = MockServer::start().await;

    // First call fails
    Mock::given(method("GET"))
        .and(path("/vault/flaky.md"))
        .and(header("Authorization", "Bearer test-api-key"))
        .respond_with(ResponseTemplate::new(500))
        .up_to_n_times(1)
        .mount(&server)
        .await;

    // Second call succeeds
    Mock::given(method("GET"))
        .and(path("/vault/flaky.md"))
        .and(header("Authorization", "Bearer test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_string("Recovered"))
        .mount(&server)
        .await;

    let client = mock_client(&server);

    // First call fails
    let first = client.read_note("flaky.md").await;
    assert!(first.is_err());

    // Second call succeeds — server is still operational
    let second = client.read_note("flaky.md").await;
    assert!(second.is_ok());
    assert_eq!(second.unwrap(), "Recovered");
}

#[tokio::test]
async fn mcp_tool_returns_is_error_on_api_failure() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/vault/broken.md"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let mut registry = ToolRegistry::new();
    tools_impl::register_all_tools(&mut registry, client);

    let result = registry
        .call("obsidian_read_note", json!({"path": "broken.md"}))
        .await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.data.as_ref().and_then(|d| d.get("isError")), Some(&json!(true)));
}

// ════════════════════════════════════════════════════════════════
// Session-End Audit
// ════════════════════════════════════════════════════════════════

#[test]
fn audit_missing_session_log() {
    let dir = tempfile::tempdir().unwrap();
    let vault_path = dir.path();

    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let manifest = format!(
        "---\ntags:\n  - type/manifest\ncreated: {today}\nupdated: {today}\ntype: manifest\nstatus: active\nconfidence: high\n---\n# Manifest"
    );

    let result = audit_session_end(vault_path, "sessions/missing.md", &manifest);
    assert!(!result.session_log_present);
    assert!(!result.all_passed());
}

#[test]
fn audit_stale_manifest() {
    let dir = tempfile::tempdir().unwrap();
    let vault_path = dir.path();

    let manifest = "---\ntags:\n  - type/manifest\ncreated: 2020-01-01\nupdated: 2020-01-01\ntype: manifest\nstatus: active\nconfidence: high\n---\n# Manifest";

    let result = audit_session_end(vault_path, "sessions/test.md", manifest);
    assert!(!result.manifest_updated);
}

#[test]
fn audit_correct_session_close() {
    let dir = tempfile::tempdir().unwrap();
    let vault_path = dir.path();

    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let manifest = format!(
        "---\ntags:\n  - type/manifest\ncreated: {today}\nupdated: {today}\ntype: manifest\nstatus: active\nconfidence: high\n---\n# Manifest"
    );

    let sessions_dir = vault_path.join("sessions");
    std::fs::create_dir_all(&sessions_dir).unwrap();
    std::fs::write(sessions_dir.join("test.md"), "# Session").unwrap();

    let result = audit_session_end(vault_path, "sessions/test.md", &manifest);
    assert!(result.all_passed());
    assert!(result.warnings.is_empty());
}

// ════════════════════════════════════════════════════════════════
// Stdio Transport
// ════════════════════════════════════════════════════════════════

#[test]
fn stdio_initialize_returns_server_info() {
    let binary = env!("CARGO_BIN_EXE_obsidian-mcp");
    let mut child = std::process::Command::new(binary)
        .args(["--transport", "stdio"])
        .env("OBSIDIAN_API_KEY", "test-key")
        .env("OBSIDIAN_API_URL", "https://localhost:1")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("failed to spawn binary");

    use std::io::{BufRead, Write};
    let stdin = child.stdin.as_mut().expect("stdin");
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": null
    });
    writeln!(stdin, "{request}").unwrap();
    stdin.flush().unwrap();

    let stdout = child.stdout.as_mut().expect("stdout");
    let reader = std::io::BufReader::new(stdout);
    let line = reader.lines().next().unwrap().unwrap();
    let resp: serde_json::Value = serde_json::from_str(&line).unwrap();

    assert_eq!(resp["jsonrpc"], "2.0");
    assert_eq!(resp["id"], 1);
    assert_eq!(resp["result"]["serverInfo"]["name"], "obsidian-mcp");
    assert_eq!(resp["result"]["protocolVersion"], "2024-11-05");

    let _ = child.kill();
    let _ = child.wait();
}

#[test]
fn stdio_tools_list_returns_16_tools() {
    let binary = env!("CARGO_BIN_EXE_obsidian-mcp");
    let mut child = std::process::Command::new(binary)
        .args(["--transport", "stdio"])
        .env("OBSIDIAN_API_KEY", "test-key")
        .env("OBSIDIAN_API_URL", "https://localhost:1")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("failed to spawn binary");

    use std::io::{BufRead, Write};
    let stdin = child.stdin.as_mut().expect("stdin");
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list",
        "params": null
    });
    writeln!(stdin, "{request}").unwrap();
    stdin.flush().unwrap();

    let stdout = child.stdout.as_mut().expect("stdout");
    let reader = std::io::BufReader::new(stdout);
    let line = reader.lines().next().unwrap().unwrap();
    let resp: serde_json::Value = serde_json::from_str(&line).unwrap();

    assert_eq!(resp["jsonrpc"], "2.0");
    assert_eq!(resp["id"], 2);
    let tools = resp["result"]["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 16);

    let _ = child.kill();
    let _ = child.wait();
}

#[test]
fn stdio_invalid_json_returns_parse_error() {
    let binary = env!("CARGO_BIN_EXE_obsidian-mcp");
    let mut child = std::process::Command::new(binary)
        .args(["--transport", "stdio"])
        .env("OBSIDIAN_API_KEY", "test-key")
        .env("OBSIDIAN_API_URL", "https://localhost:1")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("failed to spawn binary");

    use std::io::{BufRead, Write};
    let stdin = child.stdin.as_mut().expect("stdin");
    writeln!(stdin, "not valid json").unwrap();
    stdin.flush().unwrap();

    let stdout = child.stdout.as_mut().expect("stdout");
    let reader = std::io::BufReader::new(stdout);
    let line = reader.lines().next().unwrap().unwrap();
    let resp: serde_json::Value = serde_json::from_str(&line).unwrap();

    assert_eq!(resp["error"]["code"], -32700); // ParseError

    let _ = child.kill();
    let _ = child.wait();
}

#[test]
fn stdio_notification_gets_no_response() {
    let binary = env!("CARGO_BIN_EXE_obsidian-mcp");
    let mut child = std::process::Command::new(binary)
        .args(["--transport", "stdio"])
        .env("OBSIDIAN_API_KEY", "test-key")
        .env("OBSIDIAN_API_URL", "https://localhost:1")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("failed to spawn binary");

    use std::io::{BufRead, Write};
    let stdin = child.stdin.as_mut().expect("stdin");

    // Send a notification (no id) — should NOT produce a response
    let notification = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized"
    });
    writeln!(stdin, "{notification}").unwrap();
    stdin.flush().unwrap();

    // Now send a real request — should get a response
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 42,
        "method": "tools/list"
    });
    writeln!(stdin, "{request}").unwrap();
    stdin.flush().unwrap();

    let stdout = child.stdout.as_mut().expect("stdout");
    let reader = std::io::BufReader::new(stdout);
    let line = reader.lines().next().unwrap().unwrap();
    let resp: serde_json::Value = serde_json::from_str(&line).unwrap();

    // The response should be for our tools/list request (id: 42),
    // not for the notification
    assert_eq!(resp["id"], 42);
    assert!(resp["result"]["tools"].is_array());

    let _ = child.kill();
    let _ = child.wait();
}

// ════════════════════════════════════════════════════════════════
// Config Snippets Validation
// ════════════════════════════════════════════════════════════════

#[test]
fn claude_desktop_config_is_valid_json() {
    let content = std::fs::read_to_string("config/claude-desktop.json")
        .expect("config/claude-desktop.json should exist");
    let parsed: serde_json::Value = serde_json::from_str(&content)
        .expect("config should be valid JSON");
    assert!(parsed.get("mcpServers").is_some(), "should have mcpServers key");
    let obsidian = parsed["mcpServers"]["obsidian"].as_object()
        .expect("should have obsidian server config");
    // Claude Desktop uses stdio transport (command + args)
    assert!(obsidian.get("command").is_some(), "should have command field");
}

#[test]
fn claude_code_config_is_valid_json() {
    let content = std::fs::read_to_string("config/claude-code.json")
        .expect("config/claude-code.json should exist");
    let parsed: serde_json::Value = serde_json::from_str(&content)
        .expect("config should be valid JSON");
    assert!(parsed.get("mcpServers").is_some());
    let obsidian = parsed["mcpServers"]["obsidian"].as_object()
        .expect("should have obsidian server config");
    assert!(obsidian.get("command").is_some(), "should have command field");
}

#[test]
fn cursor_config_is_valid_json() {
    let content = std::fs::read_to_string("config/cursor.json")
        .expect("config/cursor.json should exist");
    let parsed: serde_json::Value = serde_json::from_str(&content)
        .expect("config should be valid JSON");
    assert!(parsed.get("mcp").is_some());
}

#[test]
fn config_snippets_have_no_hardcoded_secrets() {
    for entry in std::fs::read_dir("config").unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "json") {
            let content = std::fs::read_to_string(&path).unwrap();
            // Must not contain actual API keys — only env var references
            assert!(
                !content.contains("OBSIDIAN_API_KEY=") || content.contains("${"),
                "config file {:?} must not contain hardcoded API keys",
                path.file_name().unwrap()
            );
        }
    }
}

// ════════════════════════════════════════════════════════════════
// Binary Size Check
// ════════════════════════════════════════════════════════════════

#[test]
fn release_binary_size_under_10mb() {
    let binary_path = if cfg!(windows) {
        "target/release/obsidian-mcp.exe"
    } else {
        "target/release/obsidian-mcp"
    };

    if !std::path::Path::new(binary_path).exists() {
        eprintln!("Skipping binary size test — release binary not built");
        return;
    }

    let size = std::fs::metadata(binary_path)
        .expect("binary should exist")
        .len();

    let max_bytes = 10 * 1024 * 1024; // 10 MB — generous for cross-platform
    assert!(
        size <= max_bytes,
        "release binary is {} bytes ({:.1} MB), exceeding 10 MB limit",
        size,
        size as f64 / (1024.0 * 1024.0)
    );
}

// ════════════════════════════════════════════════════════════════
// Network Binding Default
// ════════════════════════════════════════════════════════════════

#[test]
fn default_transport_is_stdio() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_obsidian-mcp"))
        .arg("--help")
        .output()
        .expect("failed to run binary");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("stdio"),
        "default transport should be stdio, got: {stdout}"
    );
}

#[test]
fn sse_host_default_is_localhost() {
    // Verify the SSE bind address defaults to 127.0.0.1, not 0.0.0.0
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_obsidian-mcp"))
        .arg("--help")
        .output()
        .expect("failed to run binary");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("127.0.0.1"),
        "default host should be 127.0.0.1, got: {stdout}"
    );
}
