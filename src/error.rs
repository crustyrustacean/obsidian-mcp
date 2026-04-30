use thiserror::Error;

/// Typed errors for the Obsidian MCP server.
///
/// These are never panicked — all errors flow through Result types
/// and are surfaced as MCP tool errors to the client.
#[derive(Error, Debug)]
pub enum ObsidianError {
    #[error("Obsidian API returned HTTP {status}: {message}")]
    ApiError { status: u16, message: String },

    #[error("Failed to connect to Obsidian API at {url}: {source}")]
    ConnectionError {
        url: String,
        #[source]
        source: reqwest::Error,
    },

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Write verification failed: note at '{path}' appears empty after write")]
    WriteVerificationFailed { path: String },
}

impl ObsidianError {
    /// Create an API error from a reqwest Response.
    pub async fn from_response(path: &str, resp: reqwest::Response) -> Self {
        let status = resp.status().as_u16();
        let message = format!("request to '{path}' failed");
        ObsidianError::ApiError { status, message }
    }
}
