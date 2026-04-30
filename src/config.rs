use crate::error::ObsidianError;

const DEFAULT_API_URL: &str = "https://localhost:27124";

/// Configuration for connecting to the Obsidian Local REST API.
#[derive(Debug, Clone)]
pub struct Config {
    /// API key for the Obsidian Local REST API plugin.
    /// Loaded from OBSIDIAN_API_KEY environment variable.
    /// **Never log or print this value.**
    pub api_key: String,

    /// Base URL for the Obsidian Local REST API.
    /// Defaults to https://localhost:27124 if OBSIDIAN_API_URL is not set.
    pub api_url: String,
}

impl Config {
    /// Load config from a specific .env file path.
    ///
    /// Reads the file directly (does not pollute the process environment),
    /// then falls back to current env vars for anything not in the file.
    pub fn load_from_path(env_path: std::path::PathBuf) -> Result<Self, ObsidianError> {
        let mut api_key: Option<String> = None;
        let mut api_url: Option<String> = None;

        // Read the .env file directly instead of using dotenvy (which sets process env vars
        // and causes test interference). This is also more secure — no global state pollution.
        if env_path.exists() {
            let contents = std::fs::read_to_string(&env_path).map_err(|e| {
                ObsidianError::ConfigError(format!("failed to read {}: {e}", env_path.display()))
            })?;

            for line in contents.lines() {
                let line = line.trim();
                // Skip empty lines and comments
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                if let Some((key, value)) = line.split_once('=') {
                    let key = key.trim();
                    let value = value.trim();
                    // Strip surrounding quotes if present
                    let value = value
                        .strip_prefix('"')
                        .and_then(|v| v.strip_suffix('"'))
                        .or_else(|| value.strip_prefix('\'').and_then(|v| v.strip_suffix('\'')))
                        .unwrap_or(value);

                    match key {
                        "OBSIDIAN_API_KEY" => api_key = Some(value.to_string()),
                        "OBSIDIAN_API_URL" => api_url = Some(value.to_string()),
                        _ => {} // ignore unknown keys
                    }
                }
            }
        }

        // Fall back to process env vars for anything not in the file
        let api_key = api_key
            .or_else(|| std::env::var("OBSIDIAN_API_KEY").ok())
            .ok_or_else(|| {
                ObsidianError::ConfigError(
                    "OBSIDIAN_API_KEY is required but not set".to_string(),
                )
            })?;

        let api_url = api_url
            .or_else(|| std::env::var("OBSIDIAN_API_URL").ok())
            .unwrap_or_else(|| DEFAULT_API_URL.to_string());

        Ok(Config { api_key, api_url })
    }

    /// Load config from the current process environment.
    ///
    /// First attempts to load `.env` in the current directory,
    /// then falls back to raw environment variables.
    pub fn load() -> Result<Self, ObsidianError> {
        let env_path = std::path::PathBuf::from(".env");
        Self::load_from_path(env_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_api_url_is_localhost() {
        assert_eq!(DEFAULT_API_URL, "https://localhost:27124");
    }
}
