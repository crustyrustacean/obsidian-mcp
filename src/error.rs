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

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("Batch read limit exceeded: requested {requested}, maximum is {max}")]
    BatchLimitExceeded { requested: usize, max: usize },

    #[error("Rate limit exceeded for tool '{tool}': {message}")]
    RateLimitExceeded { tool: String, message: String },
}

impl ObsidianError {
    /// Create an API error from a reqwest Response.
    pub async fn from_response(path: &str, resp: reqwest::Response) -> Self {
        let status = resp.status().as_u16();
        let body = resp.text().await.unwrap_or_default();
        // Sanitize error message — don't leak full filesystem paths from Obsidian API errors
        let message = if body.is_empty() {
            format!("request to '{path}' failed")
        } else {
            // Truncate long error bodies and strip potential path leaks
            let sanitized = body
                .replace('\\', "/")
                .chars()
                .take(200)
                .collect::<String>();
            format!("request to '{path}' failed: {sanitized}")
        };
        ObsidianError::ApiError { status, message }
    }
}

/// Validate a vault path to prevent directory traversal attacks.
///
/// Rejects paths containing:
/// - `..` (directory traversal)
/// - Absolute paths (starting with `/` or a drive letter)
/// - Null bytes
/// - Paths longer than 512 characters
pub fn validate_path(path: &str) -> Result<&str, ObsidianError> {
    if path.is_empty() {
        return Err(ObsidianError::InvalidPath("path cannot be empty".to_string()));
    }

    if path.len() > 512 {
        return Err(ObsidianError::InvalidPath(
            "path exceeds maximum length of 512 characters".to_string(),
        ));
    }

    if path.contains('\0') {
        return Err(ObsidianError::InvalidPath(
            "path contains null bytes".to_string(),
        ));
    }

    if path.contains("..") {
        return Err(ObsidianError::InvalidPath(
            "path contains directory traversal (..)".to_string(),
        ));
    }

    // Reject absolute paths (Unix or Windows)
    if path.starts_with('/') || path.chars().nth(1) == Some(':') {
        return Err(ObsidianError::InvalidPath(
            "absolute paths are not allowed".to_string(),
        ));
    }

    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_path_rejects_empty() {
        assert!(validate_path("").is_err());
    }

    #[test]
    fn validate_path_rejects_traversal() {
        assert!(validate_path("../../etc/passwd").is_err());
        assert!(validate_path("notes/../secrets").is_err());
        assert!(validate_path("..").is_err());
    }

    #[test]
    fn validate_path_rejects_absolute_unix() {
        assert!(validate_path("/etc/passwd").is_err());
    }

    #[test]
    fn validate_path_rejects_absolute_windows() {
        assert!(validate_path("C:\\Windows\\System32").is_err());
    }

    #[test]
    fn validate_path_rejects_null_bytes() {
        assert!(validate_path("file\0.txt").is_err());
    }

    #[test]
    fn validate_path_rejects_very_long_paths() {
        let long_path = "a".repeat(513);
        assert!(validate_path(&long_path).is_err());
    }

    #[test]
    fn validate_path_accepts_valid_paths() {
        assert!(validate_path("notes/my-note.md").is_ok());
        assert!(validate_path("ideas-bank.md").is_ok());
        assert!(validate_path("research/deep/topic.md").is_ok());
    }
}
