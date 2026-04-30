use crate::vault::validate_manifest;
use std::path::Path;

/// Result of a session-end audit.
#[derive(Debug)]
pub struct AuditResult {
    pub session_log_present: bool,
    pub manifest_updated: bool,
    pub manifest_valid: bool,
    pub warnings: Vec<String>,
}

impl AuditResult {
    pub fn all_passed(&self) -> bool {
        self.session_log_present && self.manifest_updated && self.manifest_valid
    }
}

/// Run a session-end audit on the vault.
///
/// Checks:
/// 1. A session log exists for the current session
/// 2. The context-manifest.md has been updated (updated date matches today)
/// 3. The context-manifest.md has valid frontmatter of type "manifest"
///
/// This is a local filesystem audit — it reads files directly from the vault
/// directory, not through the Obsidian API.
pub fn audit_session_end(
    vault_path: &Path,
    session_log_path: &str,
    manifest_content: &str,
) -> AuditResult {
    let mut warnings = Vec::new();

    // Check 1: Session log file exists
    let session_log_present = vault_path.join(session_log_path).is_file();
    if !session_log_present {
        warnings.push(format!(
            "Session log missing: {session_log_path} was not written"
        ));
    }

    // Check 2 & 3: Manifest is valid and updated
    let manifest_result = validate_manifest(manifest_content);
    let manifest_valid = manifest_result.is_ok();

    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let manifest_updated = manifest_result.as_ref().is_ok_and(|fm| {
        fm.updated.to_string() == today
    });

    if !manifest_valid {
        warnings.push("Manifest is invalid or missing proper frontmatter".to_string());
    } else if !manifest_updated {
        warnings.push(format!(
            "Manifest is stale — 'updated' date is not today ({today})"
        ));
    }

    AuditResult {
        session_log_present,
        manifest_updated,
        manifest_valid,
        warnings,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn audit_detects_missing_session_log() {
        let dir = tempfile::tempdir().unwrap();
        let vault_path = dir.path();

        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let manifest = format!(
r#"---
tags:
  - type/manifest
created: {today}
updated: {today}
type: manifest
status: active
confidence: high
---

# Context Manifest
"#
        );

        // Write manifest but not session log
        fs::write(vault_path.join("context-manifest.md"), &manifest).unwrap();

        let result = audit_session_end(vault_path, "sessions/2026-04-30-1200.md", &manifest);
        assert!(!result.session_log_present);
        assert!(result.warnings.iter().any(|w| w.contains("missing")));
        assert!(!result.all_passed());
    }

    #[test]
    fn audit_detects_stale_manifest() {
        let dir = tempfile::tempdir().unwrap();
        let vault_path = dir.path();

        let manifest = r#"---
tags:
  - type/manifest
created: 2025-01-01
updated: 2025-01-01
type: manifest
status: active
confidence: high
---

# Context Manifest
"#;

        let result = audit_session_end(vault_path, "sessions/test.md", manifest);
        assert!(!result.manifest_updated);
        assert!(result.warnings.iter().any(|w| w.contains("stale")));
        assert!(!result.all_passed());
    }

    #[test]
    fn audit_passes_on_correct_session_close() {
        let dir = tempfile::tempdir().unwrap();
        let vault_path = dir.path();

        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let manifest = format!(
r#"---
tags:
  - type/manifest
created: {today}
updated: {today}
type: manifest
status: active
confidence: high
---

# Context Manifest
"#
        );

        // Write session log
        let sessions_dir = vault_path.join("sessions");
        fs::create_dir_all(&sessions_dir).unwrap();
        fs::write(sessions_dir.join("2026-04-30-1200.md"), "# Session Log").unwrap();

        let result = audit_session_end(vault_path, "sessions/2026-04-30-1200.md", &manifest);
        assert!(result.session_log_present);
        assert!(result.manifest_updated);
        assert!(result.manifest_valid);
        assert!(result.warnings.is_empty());
        assert!(result.all_passed());
    }

    #[test]
    fn audit_detects_invalid_manifest() {
        let dir = tempfile::tempdir().unwrap();
        let vault_path = dir.path();

        let sessions_dir = vault_path.join("sessions");
        fs::create_dir_all(&sessions_dir).unwrap();
        fs::write(sessions_dir.join("test.md"), "# Session").unwrap();

        let result = audit_session_end(vault_path, "sessions/test.md", "# No frontmatter\nJust text");
        assert!(!result.manifest_valid);
        assert!(!result.all_passed());
    }
}
