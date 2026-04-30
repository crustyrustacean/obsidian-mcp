// Binary entry point — crate modules are declared in lib.rs

use clap::Parser;

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

    let client = obsidian_mcp::client::ObsidianClient::new(config);

    if args.test_connection {
        match client.test_connection().await {
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

    // TODO: Phase 2 — start the MCP SSE server here
    tracing::info!("obsidian-mcp server not yet implemented (Phase 2)");
    Ok(())
}
