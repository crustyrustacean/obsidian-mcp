use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Test: HTTP client sends correct Accept: text/markdown header on read_note
#[tokio::test]
async fn read_note_sends_accept_text_markdown_header() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/vault/test-note.md"))
        .and(header("Accept", "text/markdown"))
        .respond_with(ResponseTemplate::new(200).set_body_string("# Hello World\n"))
        .expect(1)
        .mount(&mock_server)
        .await;

    let config = obsidian_mcp::config::Config {
        api_key: "test-key".to_string(),
        api_url: mock_server.uri(),
    };

    let client = obsidian_mcp::client::ObsidianClient::new(config);
    let result = client.read_note("test-note.md").await;

    assert!(result.is_ok(), "read_note should succeed");
    assert_eq!(result.unwrap(), "# Hello World\n");
}

/// Test: API key is included as Authorization: Bearer header
#[tokio::test]
async fn client_sends_authorization_bearer_header() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/vault/test-note.md"))
        .and(header("Authorization", "Bearer my-secret-key"))
        .respond_with(ResponseTemplate::new(200).set_body_string("content"))
        .expect(1)
        .mount(&mock_server)
        .await;

    let config = obsidian_mcp::config::Config {
        api_key: "my-secret-key".to_string(),
        api_url: mock_server.uri(),
    };

    let client = obsidian_mcp::client::ObsidianClient::new(config);
    let result = client.read_note("test-note.md").await;

    assert!(result.is_ok(), "should succeed when Bearer token is correct");
}

/// Test: smoke-test read returns markdown content
#[tokio::test]
async fn read_note_returns_markdown_content() {
    let mock_server = MockServer::start().await;

    let markdown_body = "# My Note\n\nSome **bold** text and a [link](https://example.com).\n";

    Mock::given(method("GET"))
        .and(path("/vault/my-note.md"))
        .and(header("Accept", "text/markdown"))
        .respond_with(ResponseTemplate::new(200).set_body_string(markdown_body))
        .mount(&mock_server)
        .await;

    let config = obsidian_mcp::config::Config {
        api_key: "test-key".to_string(),
        api_url: mock_server.uri(),
    };

    let client = obsidian_mcp::client::ObsidianClient::new(config);
    let result = client.read_note("my-note.md").await;

    assert!(result.is_ok());
    let content = result.unwrap();
    assert!(content.starts_with("# "), "markdown should start with a heading");
    assert!(content.contains("**bold**"), "markdown should contain bold text");
}

/// Test: read_note returns error on 404
#[tokio::test]
async fn read_note_returns_error_on_not_found() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/vault/nonexistent.md"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&mock_server)
        .await;

    let config = obsidian_mcp::config::Config {
        api_key: "test-key".to_string(),
        api_url: mock_server.uri(),
    };

    let client = obsidian_mcp::client::ObsidianClient::new(config);
    let result = client.read_note("nonexistent.md").await;

    assert!(result.is_err(), "should return error on 404");
}

/// Test: read_note returns error on 401 (bad API key)
#[tokio::test]
async fn read_note_returns_error_on_unauthorized() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/vault/test.md"))
        .respond_with(ResponseTemplate::new(401))
        .mount(&mock_server)
        .await;

    let config = obsidian_mcp::config::Config {
        api_key: "bad-key".to_string(),
        api_url: mock_server.uri(),
    };

    let client = obsidian_mcp::client::ObsidianClient::new(config);
    let result = client.read_note("test.md").await;

    assert!(result.is_err(), "should return error on 401");
}

/// Test: read_note returns error when Obsidian is unreachable
#[tokio::test]
async fn read_note_returns_error_when_unreachable() {
    let config = obsidian_mcp::config::Config {
        api_key: "test-key".to_string(),
        api_url: "https://127.0.0.1:1".to_string(), // port 1 should be unreachable
    };

    let client = obsidian_mcp::client::ObsidianClient::new(config);
    let result = client.read_note("test.md").await;

    assert!(result.is_err(), "should return error when server is unreachable");
}
