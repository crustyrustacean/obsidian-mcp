use crate::config::Config;
use crate::error::{validate_path, validate_query, ObsidianError};
use serde_json::Value;
use std::collections::HashMap;

/// Maximum number of paths allowed in a batch read operation.
const BATCH_READ_MAX_PATHS: usize = 20;

/// HTTP client for the Obsidian Local REST API.
///
/// Handles:
/// - Self-signed SSL certificate acceptance (Obsidian uses HTTPS with a self-signed cert)
/// - Bearer token authentication via `Authorization` header
/// - Per-request `Accept`/`Content-Type` headers (each tool sets its own explicitly)
///
/// # Security Notes
///
/// - All vault paths are URL-encoded before being interpolated into request URLs
///   to prevent path injection via percent-encoding (e.g., `%2F..%2F`).
/// - All paths are validated against directory traversal before any HTTP call.
/// - `danger_accept_invalid_certs(true)` is enabled because Obsidian's Local REST API
///   uses a self-signed certificate. See AGENTS.md for discussion of alternatives
///   (pinning the cert fingerprint, loading from plugin data directory).
/// - Write operations (write/append/patch) verify the write by reading the note back
///   and checking that byte count > 2, guarding against silent empty-write failures.
/// - Content written to the vault can contain Obsidian directives like `dataviewjs`
///   blocks. These execute JavaScript within Obsidian. The caller (AI client) is
///   responsible for ensuring written content does not inject unintended executable
///   directives. This is a known risk documented in the tool descriptions.
pub struct ObsidianClient {
    http: reqwest::Client,
    config: Config,
}

impl ObsidianClient {
    /// Create a new ObsidianClient with the given config.
    ///
    /// Installs the `ring` crypto provider for rustls if not already installed,
    /// then builds an HTTP client that accepts self-signed SSL certificates.
    pub fn new(config: Config) -> Self {
        let http = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .expect("failed to build HTTP client");

        Self { http, config }
    }

    /// Helper: build the full URL for a vault path, URL-encoding the path segment.
    ///
    /// This prevents path injection via percent-encoding (e.g., a path like
    /// `foo%2F..%2F..%2Fetc` would be double-encoded, preventing traversal).
    fn vault_url(&self, path: &str) -> String {
        format!(
            "{}/vault/{}",
            self.config.api_url,
            urlencoding::encode(path)
        )
    }

    /// Helper: build the Authorization header value.
    fn auth_header(&self) -> String {
        format!("Bearer {}", self.config.api_key)
    }

    // ── Read/Write Group (6 tools) ────────────────────────────────

    /// Read a note from the vault as markdown.
    ///
    /// Sends `GET /vault/{path}` with `Accept: text/markdown`.
    pub async fn read_note(&self, path: &str) -> Result<String, ObsidianError> {
        validate_path(path)?;
        let url = self.vault_url(path);

        let resp = self
            .http
            .get(&url)
            .header("Authorization", self.auth_header())
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

        let body = resp.text().await.map_err(|e| ObsidianError::ConnectionError {
            url,
            source: e,
        })?;

        Ok(body)
    }

    /// Read a note's metadata (frontmatter) as JSON.
    ///
    /// Sends `GET /vault/{path}` with `Accept: application/json`.
    pub async fn read_note_metadata(&self, path: &str) -> Result<Value, ObsidianError> {
        validate_path(path)?;
        let url = self.vault_url(path);

        let resp = self
            .http
            .get(&url)
            .header("Authorization", self.auth_header())
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

        let body = resp.json().await.map_err(|e| ObsidianError::ConnectionError {
            url,
            source: e,
        })?;

        Ok(body)
    }

    /// Write (create or replace) a note in the vault.
    ///
    /// Sends `PUT /vault/{path}` with `Content-Type: text/markdown`.
    /// After writing, reads the note back to verify the write succeeded
    /// (guards against silent empty-write failures).
    ///
    /// # Security Note
    /// Content written to the vault can contain Obsidian directives like
    /// `dataviewjs` blocks which execute JavaScript. The caller is responsible
    /// for ensuring written content is safe.
    pub async fn write_note(&self, path: &str, content: &str) -> Result<(), ObsidianError> {
        validate_path(path)?;
        let url = self.vault_url(path);

        let resp = self
            .http
            .put(&url)
            .header("Authorization", self.auth_header())
            .header("Content-Type", "text/markdown")
            .body(content.to_string())
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

        // Write verification: read back and check byte count > 2
        self.verify_write(path).await
    }

    /// Append content to an existing note.
    ///
    /// Sends `POST /vault/{path}` with `Content-Type: text/markdown`.
    /// The content is appended, not replaced.
    ///
    /// # Security Note
    /// Same dataviewjs risk as `write_note`.
    pub async fn append_note(&self, path: &str, content: &str) -> Result<(), ObsidianError> {
        validate_path(path)?;
        let url = self.vault_url(path);

        let resp = self
            .http
            .post(&url)
            .header("Authorization", self.auth_header())
            .header("Content-Type", "text/markdown")
            .body(content.to_string())
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

        self.verify_write(path).await
    }

    /// Patch (update) a specific heading within a note.
    ///
    /// Sends `PATCH /vault/{path}` with the heading and content.
    ///
    /// # Security Note
    /// Same dataviewjs risk as `write_note`.
    pub async fn patch_note(
        &self,
        path: &str,
        heading: &str,
        content: &str,
    ) -> Result<(), ObsidianError> {
        validate_path(path)?;
        let url = self.vault_url(path);

        let resp = self
            .http
            .patch(&url)
            .header("Authorization", self.auth_header())
            .header("Content-Type", "text/markdown")
            .header("Heading", heading)
            .body(content.to_string())
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

        self.verify_write(path).await
    }

    /// Delete a note from the vault.
    ///
    /// Sends `DELETE /vault/{path}`. Requires `confirm: true` to prevent
    /// accidental deletion via a misformatted MCP call.
    pub async fn delete_note(&self, path: &str, confirm: bool) -> Result<(), ObsidianError> {
        validate_path(path)?;

        if !confirm {
            return Err(ObsidianError::DeleteConfirmationRequired);
        }

        let url = self.vault_url(path);

        let resp = self
            .http
            .delete(&url)
            .header("Authorization", self.auth_header())
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

        Ok(())
    }

    /// Verify a write by reading the note back and checking byte count > 2.
    async fn verify_write(&self, path: &str) -> Result<(), ObsidianError> {
        match self.read_note(path).await {
            Ok(content) if content.len() > 2 => Ok(()),
            Ok(_) => Err(ObsidianError::WriteVerificationFailed {
                path: path.to_string(),
            }),
            Err(e) => Err(e),
        }
    }

    // ── Search Group (3 tools) ────────────────────────────────────

    /// Full-text search across the vault.
    ///
    /// Sends `GET /search/{query}` with URL-encoded query.
    pub async fn search(&self, query: &str) -> Result<Vec<Value>, ObsidianError> {
        validate_query(query, "query")?;
        let url = format!(
            "{}/search/{}",
            self.config.api_url,
            urlencoding::encode(query)
        );

        let resp = self
            .http
            .get(&url)
            .header("Authorization", self.auth_header())
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| ObsidianError::ConnectionError {
                url: url.clone(),
                source: e,
            })?;

        let status = resp.status();
        if !status.is_success() {
            return Err(ObsidianError::from_response("search", resp).await);
        }

        let results: Vec<Value> = resp.json().await.map_err(|e| ObsidianError::ConnectionError {
            url,
            source: e,
        })?;

        Ok(results)
    }

    /// Execute a Dataview DQL query.
    ///
    /// Sends `POST /search/` with `Content-Type: application/vnd.olrapi.dataview.dql+txt`.
    /// SECURITY: The caller is responsible for query safety. Obsidian's Dataview plugin
    /// has its own sandbox, but queries can still be resource-intensive.
    pub async fn dataview_query(&self, dql: &str) -> Result<Vec<Value>, ObsidianError> {
        validate_query(dql, "dql")?;
        let url = format!("{}/search/", self.config.api_url);

        let resp = self
            .http
            .post(&url)
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/vnd.olrapi.dataview.dql+txt")
            .body(dql.to_string())
            .send()
            .await
            .map_err(|e| ObsidianError::ConnectionError {
                url: url.clone(),
                source: e,
            })?;

        let status = resp.status();
        if !status.is_success() {
            return Err(ObsidianError::from_response("dataview_query", resp).await);
        }

        let results: Vec<Value> = resp.json().await.map_err(|e| ObsidianError::ConnectionError {
            url,
            source: e,
        })?;

        Ok(results)
    }

    /// Execute a JSONLogic search query.
    ///
    /// Sends `POST /search/` with `Content-Type: application/vnd.olrapi.jsonlogic+json`.
    pub async fn jsonlogic_query(&self, logic: &Value) -> Result<Vec<Value>, ObsidianError> {
        let url = format!("{}/search/", self.config.api_url);

        let resp = self
            .http
            .post(&url)
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/vnd.olrapi.jsonlogic+json")
            .json(logic)
            .send()
            .await
            .map_err(|e| ObsidianError::ConnectionError {
                url: url.clone(),
                source: e,
            })?;

        let status = resp.status();
        if !status.is_success() {
            return Err(ObsidianError::from_response("jsonlogic_query", resp).await);
        }

        let results: Vec<Value> = resp.json().await.map_err(|e| ObsidianError::ConnectionError {
            url,
            source: e,
        })?;

        Ok(results)
    }

    // ── Batch/Convenience Group (3 tools) ──────────────────────────

    /// Read multiple notes in parallel.
    ///
    /// Returns a map of path → content. Failed reads are reported in the
    /// returned map with an error message rather than causing the entire
    /// operation to fail. Enforces a maximum of 20 paths.
    pub async fn batch_read(
        &self,
        paths: &[String],
    ) -> Result<HashMap<String, String>, ObsidianError> {
        if paths.len() > BATCH_READ_MAX_PATHS {
            return Err(ObsidianError::BatchLimitExceeded {
                requested: paths.len(),
                max: BATCH_READ_MAX_PATHS,
            });
        }

        // Pre-validate all paths before making any HTTP calls
        for path in paths {
            validate_path(path)?;
        }

        let futures: Vec<_> = paths
            .iter()
            .map(|path| {
                let path = path.clone();
                async move {
                    let result = self.read_note(&path).await;
                    (path, result)
                }
            })
            .collect();

        let results = futures::future::join_all(futures).await;

        let mut map = HashMap::new();
        for (path, result) in results {
            match result {
                Ok(content) => {
                    map.insert(path, content);
                }
                Err(e) => {
                    // Report partial failures without panicking
                    map.insert(path, format!("[ERROR] {e}"));
                }
            }
        }

        Ok(map)
    }

    /// Get a periodic note (daily, weekly, or monthly).
    ///
    /// Routes to the correct Periodic Notes plugin endpoint based on the period type.
    pub async fn periodic_note(&self, period: &str) -> Result<String, ObsidianError> {
        let valid_periods = ["daily", "weekly", "monthly"];
        let period = period.to_lowercase();
        if !valid_periods.contains(&period.as_str()) {
            return Err(ObsidianError::InvalidPath(format!(
                "invalid period '{period}': must be one of daily, weekly, monthly"
            )));
        }

        let url = format!("{}/periodic/{period}", self.config.api_url);

        let resp = self
            .http
            .get(&url)
            .header("Authorization", self.auth_header())
            .header("Accept", "text/markdown")
            .send()
            .await
            .map_err(|e| ObsidianError::ConnectionError {
                url: url.clone(),
                source: e,
            })?;

        let status = resp.status();
        if !status.is_success() {
            return Err(ObsidianError::from_response(&period, resp).await);
        }

        let body = resp.text().await.map_err(|e| ObsidianError::ConnectionError {
            url,
            source: e,
        })?;

        Ok(body)
    }

    /// Get recently changed notes sorted by modification time.
    ///
    /// Recursively walks the vault directory tree via the Obsidian API,
    /// collects file entries with their metadata (including mtime),
    /// sorts by most recent modification time first, and returns
    /// the top N entries using full paths (not relative — fixes a silent
    /// bug in the original Python implementation).
    pub async fn recent_changes(&self, limit: usize) -> Result<Vec<Value>, ObsidianError> {
        let limit = limit.clamp(1, 100);

        // Fetch root directory listing with metadata
        let url = format!("{}/vault/", self.config.api_url);

        let resp = self
            .http
            .get(&url)
            .header("Authorization", self.auth_header())
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| ObsidianError::ConnectionError {
                url: url.clone(),
                source: e,
            })?;

        let status = resp.status();
        if !status.is_success() {
            return Err(ObsidianError::from_response("recent_changes", resp).await);
        }

        let body: Value = resp.json().await.map_err(|e| ObsidianError::ConnectionError {
            url,
            source: e,
        })?;

        // The Obsidian Local REST API returns a "files" array from directory listings.
        // Each file entry may include metadata like mtime.
        let files = body
            .get("files")
            .and_then(|f| f.as_array())
            .cloned()
            .unwrap_or_default();

        // Recursively collect all markdown files with their metadata
        let mut all_files = Vec::new();
        self.collect_files_recursive(&files, &mut all_files).await;

        // Sort by mtime descending (most recent first).
        // mtime is stored as seconds since epoch in the Obsidian API metadata.
        all_files.sort_by(|a, b| {
            let mtime_a = a.get("mtime").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let mtime_b = b.get("mtime").and_then(|v| v.as_f64()).unwrap_or(0.0);
            mtime_b.partial_cmp(&mtime_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        let results: Vec<Value> = all_files.into_iter().take(limit).collect();

        Ok(results)
    }

    /// Recursively collect file entries from directory listings,
    /// fetching metadata for subdirectories.
    fn collect_files_recursive<'a>(
        &'a self,
        entries: &'a [Value],
        results: &'a mut Vec<Value>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            for entry in entries {
                let is_folder = entry
                    .get("type")
                    .and_then(|v| v.as_str())
                    .map(|t| t == "folder")
                    .unwrap_or(false);

                if is_folder {
                    // Recursively fetch subdirectory contents
                    if let Some(path) = entry.get("path").and_then(|v| v.as_str()) {
                        if let Ok(listing) = self.list_directory(path).await {
                            if let Some(children) = listing.get("files").and_then(|f| f.as_array()) {
                                self.collect_files_recursive(children, results).await;
                            }
                        }
                    }
                } else {
                    // It's a file — include it
                    results.push(entry.clone());
                }
            }
        })
    }

    // ── Navigate Group (4 tools) ──────────────────────────────────

    /// List the contents of a directory in the vault.
    ///
    /// Sends `GET /vault/{path}/` with `Accept: application/json`.
    /// Returns a structured list of entries with names and types (file/folder).
    pub async fn list_directory(&self, path: &str) -> Result<Value, ObsidianError> {
        // Allow empty path (root listing) but validate non-empty paths
        if !path.is_empty() {
            validate_path(path)?;
        }

        let url = if path.is_empty() {
            format!("{}/vault/", self.config.api_url)
        } else {
            format!(
                "{}/vault/{}/",
                self.config.api_url,
                urlencoding::encode(path)
            )
        };

        let resp = self
            .http
            .get(&url)
            .header("Authorization", self.auth_header())
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

        let body: Value = resp.json().await.map_err(|e| ObsidianError::ConnectionError {
            url,
            source: e,
        })?;

        Ok(body)
    }

    /// Get the tag hierarchy with counts.
    ///
    /// Sends `GET /vault-tags/` or equivalent endpoint.
    pub async fn get_tags(&self) -> Result<Value, ObsidianError> {
        let url = format!("{}/vault-tags/", self.config.api_url);

        let resp = self
            .http
            .get(&url)
            .header("Authorization", self.auth_header())
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| ObsidianError::ConnectionError {
                url: url.clone(),
                source: e,
            })?;

        let status = resp.status();
        if !status.is_success() {
            return Err(ObsidianError::from_response("get_tags", resp).await);
        }

        let body: Value = resp.json().await.map_err(|e| ObsidianError::ConnectionError {
            url,
            source: e,
        })?;

        Ok(body)
    }

    /// Health check — test connectivity to the Obsidian API.
    ///
    /// Returns Ok(()) if the API is reachable and the API key is valid.
    pub async fn server_status(&self) -> Result<(), ObsidianError> {
        let url = format!("{}/", self.config.api_url);

        let resp = self
            .http
            .get(&url)
            .header("Authorization", self.auth_header())
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

    /// Trigger Obsidian UI to open a note.
    ///
    /// Sends `POST /open/{path}` to tell the Obsidian app to navigate to the note.
    pub async fn open_note(&self, path: &str) -> Result<(), ObsidianError> {
        validate_path(path)?;
        let url = format!(
            "{}/open/{}",
            self.config.api_url,
            urlencoding::encode(path)
        );

        let resp = self
            .http
            .post(&url)
            .header("Authorization", self.auth_header())
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

        Ok(())
    }

    // ── Legacy alias ─────────────────────────────────────────────

    /// Test connectivity to the Obsidian API.
    /// Alias for `server_status()`.
    pub async fn test_connection(&self) -> Result<(), ObsidianError> {
        self.server_status().await
    }
}
