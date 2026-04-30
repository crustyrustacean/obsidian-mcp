// Binary entry point — crate modules are declared in lib.rs

use clap::Parser;
use std::sync::Arc;

/// Obsidian Second Brain MCP Server
///
/// Bridges an Obsidian vault to MCP-compatible AI clients via the
/// Local REST API plugin.
#[derive(Parser, Debug)]
#[command(name = "obsidian-mcp", version, about)]
struct Args {
    /// Test connectivity to the Obsidian Local REST API and exit
    #[arg(long)]
    test_connection: bool,

    /// Path to a .env file (default: .env in current directory)
    #[arg(long)]
    env_file: Option<std::path::PathBuf>,

    /// Host to bind the MCP server to (default: 127.0.0.1)
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// Port for the MCP server to listen on (default: 3000)
    #[arg(long, default_value_t = 3000)]
    port: u16,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing subscriber (respects RUST_LOG env var)
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let args = Args::parse();

    // Load config — API key is never logged (see security review in AGENTS.md)
    let config = if let Some(env_path) = args.env_file {
        obsidian_mcp::config::Config::load_from_path(env_path)?
    } else {
        obsidian_mcp::config::Config::load()?
    };

    tracing::info!(
        api_url = %config.api_url,
        "connecting to Obsidian Local REST API"
        // NOTE: api_key is intentionally NOT logged here
    );

    let obsidian_client = Arc::new(obsidian_mcp::client::ObsidianClient::new(config));

    if args.test_connection {
        match obsidian_client.test_connection().await {
            Ok(()) => {
                println!("✓ Connected to Obsidian Local REST API successfully");
                return Ok(());
            }
            Err(e) => {
                eprintln!("✗ Connection failed: {e}");
                std::process::exit(1);
            }
        }
    }

    // Build the tool registry with all 16 tools
    let mut registry = obsidian_mcp::tools::ToolRegistry::new();
    obsidian_mcp::tools_impl::register_all_tools(&mut registry, obsidian_client);

    let tool_count = registry.list().len();
    tracing::info!(tools = tool_count, "registered MCP tools");

    let app = obsidian_mcp::server::app(registry);

    let addr = format!("{}:{}", args.host, args.port);
    tracing::info!("starting MCP server on {addr}");
    tracing::info!("SSE endpoint: http://{addr}/sse");
    tracing::info!("MCP endpoint: http://{addr}/mcp");

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

/// Graceful shutdown on SIGINT (Ctrl+C) or SIGTERM.
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to listen for ctrl+c");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to listen for SIGTERM")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => tracing::info!("received Ctrl+C, shutting down"),
        _ = terminate => tracing::info!("received SIGTERM, shutting down"),
    }
}
