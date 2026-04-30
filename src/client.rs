use crate::config::Config;
use crate::error::ObsidianError;

/// HTTP client for the Obsidian Local REST API.
///
/// Handles:
/// - Self-signed SSL certificate acceptance (Obsidian uses HTTPS with a self-signed cert)
/// - Bearer token authentication via `Authorization` header
/// - Per-request `Accept` headers (text/markdown vs application/json)
pub struct ObsidianClient {
    http: reqwest::Client,
    config: Config,
}

impl ObsidianClient {
    /// Create a new ObsidianClient with the given config.
    ///
    /// Builds an HTTP client that:
    /// - Accepts self-signed SSL certificates
    /// - Attaches the Bearer token to every request
    pub fn new(config: Config) -> Self {
        let http = reqwest::Client::builder()
            // Obsidian Local REST API uses a self-signed cert.
            // SECURITY: This disables TLS certificate verification globally for this client.
            // See AGENTS.md and Phase 1 security review for discussion of alternatives
            // (pinning the cert fingerprint, loading from plugin data directory).
            .danger_accept_invalid_certs(true)
            .build()
            .expect("failed to build HTTP client");

        Self { http, config }
    }

    /// Read a note from the vault as markdown.
    ///
    /// Sends `GET /vault/{path}` with `Accept: text/markdown`.
    pub async fn read_note(&self, path: &str) -> Result<String, ObsidianError> {
        let url = format!("{}/vault/{}", self.config.api_url, path);

        let resp = self
            .http
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Accept", "text/markdown")
            .send()
            .await
            .map_err(|e| ObsidianError::ConnectionError {
                url: url.clone(),
                source: e,
            })?;

        let status = resp.status();
        if !status.is_success() {
            return Err(ObsidianError::from_response(path, resp).await);
        }

        let body = resp
            .text()
            .await
            .map_err(|e| ObsidianError::ConnectionError {
                url,
                source: e,
            })?;

        Ok(body)
    }

    /// Read a note's metadata (frontmatter) as JSON.
    ///
    /// Sends `GET /vault/{path}` with `Accept: application/json`.
    pub async fn read_note_metadata(&self, path: &str) -> Result<serde_json::Value, ObsidianError> {
        let url = format!("{}/vault/{}", self.config.api_url, path);

        let resp = self
            .http
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| ObsidianError::ConnectionError {
                url: url.clone(),
                source: e,
            })?;

        let status = resp.status();
        if !status.is_success() {
            return Err(ObsidianError::from_response(path, resp).await);
        }

        let body = resp
            .json()
            .await
            .map_err(|e| ObsidianError::ConnectionError {
                url,
                source: e,
            })?;

        Ok(body)
    }

    /// Test connectivity to the Obsidian API.
    ///
    /// Returns Ok(()) if the API is reachable and the API key is valid.
    pub async fn test_connection(&self) -> Result<(), ObsidianError> {
        // The Local REST API doesn't have a dedicated health endpoint,
        // so we try to list the root vault directory.
        let url = format!("{}/", self.config.api_url);

        let resp = self
            .http
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .send()
            .await
            .map_err(|e| ObsidianError::ConnectionError {
                url: url.clone(),
                source: e,
            })?;

        if resp.status().is_success() {
            Ok(())
        } else {
            Err(ObsidianError::ApiError {
                status: resp.status().as_u16(),
                message: "connection test failed — check API key and Obsidian plugin status"
                    .to_string(),
            })
        }
    }
}
