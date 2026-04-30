use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;

/// Test: SSL self-signed cert is accepted by the Obsidian client
///
/// Spins up a local HTTPS server with a self-signed certificate,
/// then confirms the ObsidianClient can connect to it without
/// certificate verification errors.
#[tokio::test]
async fn client_accepts_self_signed_cert() {
    // Install a crypto provider for rustls (needed once per process)
    let _ = rustls::crypto::ring::default_provider().install_default();

    // Generate a self-signed cert using rcgen 0.13 API
    let rcgen::CertifiedKey { cert, key_pair } =
        rcgen::generate_simple_self_signed(vec!["localhost".to_string(), "127.0.0.1".to_string()])
            .unwrap();

    let cert_der = cert.der().clone();
    let key_der = key_pair.serialize_der();

    // Build a TLS acceptor for the server side
    let key = rustls::pki_types::PrivateKeyDer::try_from(key_der.clone()).unwrap();
    let server_config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(vec![cert_der], key)
        .unwrap();
    let acceptor = TlsAcceptor::from(Arc::new(server_config));

    // Start a bare TCP listener
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let port = addr.port();

    // Spawn a minimal TLS server that responds with a proper HTTP response
    let server_handle = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let mut tls_stream = acceptor.accept(stream).await.unwrap();
        use tokio::io::AsyncReadExt;
        use tokio::io::AsyncWriteExt;

        // Read the request (we don't care about its content, just drain it)
        let mut buf = vec![0u8; 8192];
        let _ = tls_stream.read(&mut buf).await;

        let body = "# Hello World\n";
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/markdown\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        tls_stream.write_all(response.as_bytes()).await.unwrap();
        tls_stream.shutdown().await.unwrap();
    });

    // Build an ObsidianClient configured for this local server
    let config = obsidian_mcp::config::Config {
        api_key: "test-key".to_string(),
        api_url: format!("https://127.0.0.1:{port}"),
    };

    let client = obsidian_mcp::client::ObsidianClient::new(config);
    let result = client.read_note("test.md").await;

    // The key assertion: we connected to a self-signed cert server without error
    assert!(
        result.is_ok(),
        "client should accept self-signed cert, got error: {:?}",
        result.err()
    );
    let content = result.unwrap();
    assert!(
        content.contains("# Hello World"),
        "response should contain the markdown content, got: {content}"
    );

    // Clean up the server task
    let _ = server_handle.await;
}
