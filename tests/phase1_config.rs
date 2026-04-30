use std::io::Write;

/// Test: .env loading populates config struct
///
/// Writes a temp .env file with OBSIDIAN_API_KEY and OBSIDIAN_API_URL,
/// loads it, and asserts the config struct has the correct values.
#[tokio::test]
async fn config_loads_from_env_file() {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    let env_path = dir.path().join(".env");

    let mut f = std::fs::File::create(&env_path).expect("failed to create .env");
    writeln!(f, "OBSIDIAN_API_KEY=test-key-12345").unwrap();
    writeln!(f, "OBSIDIAN_API_URL=https://localhost:27124").unwrap();

    let config = obsidian_mcp::config::Config::load_from_path(dir.path().join(".env"))
        .expect("config should load");

    assert_eq!(config.api_key, "test-key-12345");
    assert_eq!(config.api_url, "https://localhost:27124");
}

/// Test: config has sensible defaults when URL is missing
#[tokio::test]
async fn config_defaults_url_when_missing() {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    let env_path = dir.path().join(".env");

    let mut f = std::fs::File::create(&env_path).expect("failed to create .env");
    writeln!(f, "OBSIDIAN_API_KEY=some-key").unwrap();
    // Deliberately omit OBSIDIAN_API_URL

    let config = obsidian_mcp::config::Config::load_from_path(dir.path().join(".env"))
        .expect("config should load with default URL");

    assert_eq!(config.api_key, "some-key");
    assert_eq!(
        config.api_url, "https://localhost:27124",
        "should default to the standard Obsidian Local REST API URL"
    );
}

/// Test: config returns error when API key is missing
#[tokio::test]
async fn config_errors_without_api_key() {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    let env_path = dir.path().join(".env");

    let mut f = std::fs::File::create(&env_path).expect("failed to create .env");
    writeln!(f, "OBSIDIAN_API_URL=https://localhost:27124").unwrap();
    // Deliberately omit OBSIDIAN_API_KEY

    let result = obsidian_mcp::config::Config::load_from_path(dir.path().join(".env"));

    assert!(result.is_err(), "should error when API key is missing");
}
