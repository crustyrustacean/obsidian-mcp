use crate::error::ObsidianError;
use crate::frontmatter::{Frontmatter, NoteType};
use chrono::Local;
use std::path::Path;

/// Required directories in the vault.
const VAULT_DIRS: &[&str] = &["content", "research", "sessions", "troubleshooting"];

/// Seed files to create in the vault root and content directory.
struct SeedFile {
    path: &'static str,
    content: String,
}

/// Get the list of seed files for vault initialization.
fn seed_files() -> Vec<SeedFile> {
    let today = Local::now().format("%Y-%m-%d").to_string();
    vec![
        SeedFile {
            path: "context-manifest.md",
            content: format!(
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

This is the AI's entry point into the vault. Read this file first.

## Active Sessions

<!-- Sessions are appended here automatically -->

## Current Focus

<!-- Updated at session end -->

## Key Decisions

<!-- Accumulated over time -->
"#,
            ),
        },
        SeedFile {
            path: "_index.md",
            content: format!(
r#"---
tags:
  - type/index
created: {today}
updated: {today}
type: index
status: active
confidence: high
---

# Vault Index

Dataview live queries for quick navigation.

## Recent Session Logs

```dataview
LIST FROM "sessions"
SORT file.mtime DESC
LIMIT 10
```

## Active Ideas

```dataview
TABLE status, confidence
FROM "content"
WHERE type = "idea"
SORT updated DESC
```

## Open Troubleshooting

```dataview
TABLE status, confidence
FROM "troubleshooting"
WHERE status = "active"
SORT updated DESC
```
"#,
            ),
        },
        SeedFile {
            path: "content/ideas-bank.md",
            content: format!(
r#"---
tags:
  - type/idea
  - topic/general
created: {today}
updated: {today}
type: idea
status: active
confidence: medium
---

# Ideas Bank

Capture ideas here. Each idea should have a heading with a short description.

<!-- Add ideas below -->
"#,
            ),
        },
        SeedFile {
            path: "content/growth-experiments.md",
            content: format!(
r#"---
tags:
  - type/research
  - topic/growth
created: {today}
updated: {today}
type: research
status: active
confidence: medium
---

# Growth Experiments

Track experiments and their outcomes.

| Experiment | Hypothesis | Result | Date |
|------------|-----------|--------|------|
<!-- Add experiments below -->
"#,
            ),
        },
    ]
}

/// Initialize a vault directory with the required structure and seed files.
///
/// Creates directories and seed files if they don't exist.
/// Idempotent — running twice on the same directory does not overwrite existing files.
///
/// # Errors
///
/// Returns an error if the directory cannot be created or if a seed file
/// cannot be written (but only for new files — existing files are left alone).
pub fn init_vault(vault_path: &Path) -> Result<(), ObsidianError> {
    // Create the root directory if it doesn't exist
    std::fs::create_dir_all(vault_path).map_err(|e| {
        ObsidianError::ConfigError(format!(
            "failed to create vault directory {}: {e}",
            vault_path.display()
        ))
    })?;

    // Create required subdirectories
    for dir in VAULT_DIRS {
        let dir_path = vault_path.join(dir);
        std::fs::create_dir_all(&dir_path).map_err(|e| {
            ObsidianError::ConfigError(format!(
                "failed to create vault subdirectory {}: {e}",
                dir_path.display()
            ))
        })?;
    }

    // Create seed files (only if they don't already exist — idempotent)
    for seed in seed_files() {
        let file_path = vault_path.join(seed.path);
        if !file_path.exists() {
            // Ensure parent directory exists
            if let Some(parent) = file_path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    ObsidianError::ConfigError(format!(
                        "failed to create parent directory {}: {e}",
                        parent.display()
                    ))
                })?;
            }
            std::fs::write(&file_path, seed.content).map_err(|e| {
                ObsidianError::ConfigError(format!(
                    "failed to write seed file {}: {e}",
                    file_path.display()
                ))
            })?;
        }
    }

    Ok(())
}

/// Validate that a vault directory has the expected structure.
///
/// Checks that all required directories and root files exist.
pub fn validate_vault(vault_path: &Path) -> Result<VaultValidation, ObsidianError> {
    let mut missing_dirs = Vec::new();
    let mut missing_files = Vec::new();

    for dir in VAULT_DIRS {
        if !vault_path.join(dir).is_dir() {
            missing_dirs.push(dir.to_string());
        }
    }

    let required_files = ["context-manifest.md", "_index.md"];
    for file in required_files {
        if !vault_path.join(file).is_file() {
            missing_files.push(file.to_string());
        }
    }

    Ok(VaultValidation {
        valid: missing_dirs.is_empty() && missing_files.is_empty(),
        missing_dirs,
        missing_files,
    })
}

/// Result of vault structure validation.
#[derive(Debug)]
pub struct VaultValidation {
    pub valid: bool,
    pub missing_dirs: Vec<String>,
    pub missing_files: Vec<String>,
}

// ── Session Protocols ────────────────────────────────────────────

/// Maximum number of MCP calls allowed at session start.
pub const MAX_SESSION_START_CALLS: usize = 5;

/// Task type for routing decisions during session start.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskType {
    /// Coding or debugging — load troubleshooting files
    Coding,
    /// Planning — load content files
    Planning,
    /// General — manifest only
    General,
}

impl TaskType {
    /// Determine the task type from a description string.
    ///
    /// Simple keyword matching for now. Could be extended with
    /// more sophisticated classification later.
    pub fn from_description(desc: &str) -> Self {
        let lower = desc.to_lowercase();
        if lower.contains("debug")
            || lower.contains("error")
            || lower.contains("bug")
            || lower.contains("fix")
            || lower.contains("compile")
            || lower.contains("build fail")
        {
            TaskType::Coding
        } else if lower.contains("plan")
            || lower.contains("design")
            || lower.contains("architect")
            || lower.contains("roadmap")
            || lower.contains("strategy")
        {
            TaskType::Planning
        } else {
            TaskType::General
        }
    }

    /// Get the files to load for this task type during session start.
    ///
    /// Returns a list of vault-relative paths. The session start protocol
    /// reads `context-manifest.md` (1 call) plus these additional files.
    /// Total calls must not exceed MAX_SESSION_START_CALLS.
    pub fn files_to_load(&self) -> Vec<&'static str> {
        match self {
            TaskType::Coding => vec![
                "troubleshooting/",       // list directory
                "context-manifest.md",    // entry point
            ],
            TaskType::Planning => vec![
                "content/ideas-bank.md",
                "content/growth-experiments.md",
                "context-manifest.md",
            ],
            TaskType::General => vec![
                "context-manifest.md",
            ],
        }
    }

    /// Count the total MCP calls needed for session start with this task type.
    pub fn session_start_call_count(&self) -> usize {
        self.files_to_load().len()
    }
}

/// Generate a session log filename based on the current timestamp.
pub fn session_log_filename() -> String {
    let now = Local::now();
    format!("sessions/{}.md", now.format("%Y-%m-%d-%H%M"))
}

/// Generate the content for a new session log.
pub fn session_log_content(
    project_name: &str,
    task_type: &TaskType,
    summary: &str,
) -> String {
    let today = Local::now().format("%Y-%m-%d").to_string();
    let type_tag = match task_type {
        TaskType::Coding => "type/session-log",
        TaskType::Planning => "type/session-log",
        TaskType::General => "type/session-log",
    };
    let topic_tag = match task_type {
        TaskType::Coding => "topic/coding",
        TaskType::Planning => "topic/planning",
        TaskType::General => "topic/general",
    };

    format!(
r#"---
tags:
  - project/{project_name}
  - {type_tag}
  - {topic_tag}
created: {today}
updated: {today}
type: session-log
status: active
confidence: medium
---

# Session Log — {today}

**Task type:** {task_type}
**Project:** {project_name}

## Summary

{summary}

## Notes

<!-- Session notes go here -->

## Outcomes

<!-- To be filled at session end -->
"#,
        task_type = match task_type {
            TaskType::Coding => "Coding / Debugging",
            TaskType::Planning => "Planning",
            TaskType::General => "General",
        },
    )
}

/// Validate a context manifest file.
///
/// Checks that the manifest has valid frontmatter of type "manifest"
/// and that it can be parsed without errors.
pub fn validate_manifest(content: &str) -> Result<Frontmatter, ObsidianError> {
    let fm = Frontmatter::from_markdown(content)
        .map_err(|e| ObsidianError::ConfigError(format!("invalid manifest: {e}")))?
        .ok_or_else(|| {
            ObsidianError::ConfigError("manifest has no frontmatter".to_string())
        })?;

    if fm.note_type != NoteType::Manifest {
        return Err(ObsidianError::ConfigError(format!(
            "manifest has wrong type: expected 'manifest', got '{}'",
            fm.note_type
        )));
    }

    fm.validate()
        .map_err(|e| ObsidianError::ConfigError(format!("manifest validation failed: {e}")))?;

    Ok(fm)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn vault_init_creates_directories() {
        let dir = tempfile::tempdir().unwrap();
        let vault_path = dir.path();

        init_vault(vault_path).unwrap();

        for dir_name in VAULT_DIRS {
            assert!(vault_path.join(dir_name).is_dir(), "missing directory: {dir_name}");
        }
    }

    #[test]
    fn vault_init_creates_seed_files() {
        let dir = tempfile::tempdir().unwrap();
        let vault_path = dir.path();

        init_vault(vault_path).unwrap();

        assert!(vault_path.join("context-manifest.md").is_file());
        assert!(vault_path.join("_index.md").is_file());
        assert!(vault_path.join("content/ideas-bank.md").is_file());
        assert!(vault_path.join("content/growth-experiments.md").is_file());
    }

    #[test]
    fn vault_init_is_idempotent() {
        let dir = tempfile::tempdir().unwrap();
        let vault_path = dir.path();

        init_vault(vault_path).unwrap();

        // Modify a seed file
        let manifest_path = vault_path.join("context-manifest.md");
        let original = fs::read_to_string(&manifest_path).unwrap();
        fs::write(&manifest_path, "MODIFIED").unwrap();

        // Run init again
        init_vault(vault_path).unwrap();

        // The modified file should NOT be overwritten
        assert_eq!(fs::read_to_string(&manifest_path).unwrap(), "MODIFIED");
        let _ = original; // suppress unused warning
    }

    #[test]
    fn vault_validate_detects_missing_dirs() {
        let dir = tempfile::tempdir().unwrap();
        let vault_path = dir.path();

        // Create only some directories
        fs::create_dir_all(vault_path.join("content")).unwrap();
        fs::write(vault_path.join("context-manifest.md"), "").unwrap();
        fs::write(vault_path.join("_index.md"), "").unwrap();

        let result = validate_vault(vault_path).unwrap();
        assert!(!result.valid);
        assert!(result.missing_dirs.contains(&"research".to_string()));
        assert!(result.missing_dirs.contains(&"sessions".to_string()));
        assert!(result.missing_dirs.contains(&"troubleshooting".to_string()));
    }

    #[test]
    fn vault_validate_passes_for_valid_vault() {
        let dir = tempfile::tempdir().unwrap();
        let vault_path = dir.path();

        init_vault(vault_path).unwrap();

        let result = validate_vault(vault_path).unwrap();
        assert!(result.valid);
        assert!(result.missing_dirs.is_empty());
        assert!(result.missing_files.is_empty());
    }

    #[test]
    fn task_type_coding_from_description() {
        assert_eq!(TaskType::from_description("debug the build error"), TaskType::Coding);
        assert_eq!(TaskType::from_description("fix the bug in parser"), TaskType::Coding);
        assert_eq!(TaskType::from_description("compile fails on windows"), TaskType::Coding);
    }

    #[test]
    fn task_type_planning_from_description() {
        assert_eq!(TaskType::from_description("plan the next sprint"), TaskType::Planning);
        assert_eq!(TaskType::from_description("design the architecture"), TaskType::Planning);
    }

    #[test]
    fn task_type_general_from_description() {
        assert_eq!(TaskType::from_description("what's in the vault?"), TaskType::General);
        assert_eq!(TaskType::from_description("read the notes"), TaskType::General);
    }

    #[test]
    fn session_start_calls_within_limit() {
        // Every task type's session start must not exceed the max call limit
        for task in &[TaskType::Coding, TaskType::Planning, TaskType::General] {
            let count = task.session_start_call_count();
            assert!(
                count <= MAX_SESSION_START_CALLS,
                "task {task:?} requires {count} calls, exceeding limit of {MAX_SESSION_START_CALLS}"
            );
        }
    }

    #[test]
    fn session_log_filename_has_expected_format() {
        let filename = session_log_filename();
        assert!(filename.starts_with("sessions/"));
        assert!(filename.ends_with(".md"));
    }

    #[test]
    fn session_log_content_has_frontmatter() {
        let content = session_log_content("test-project", &TaskType::Coding, "Fixing a bug");
        let fm = Frontmatter::from_markdown(&content).unwrap().unwrap();
        assert_eq!(fm.note_type, NoteType::SessionLog);
        assert!(fm.tags.iter().any(|t| t.contains("project/test-project")));
        assert!(fm.tags.iter().any(|t| t == "topic/coding"));
    }

    #[test]
    fn validate_manifest_accepts_valid() {
        let today = Local::now().format("%Y-%m-%d").to_string();
        let content = format!(
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
        assert!(validate_manifest(&content).is_ok());
    }

    #[test]
    fn validate_manifest_rejects_wrong_type() {
        let today = Local::now().format("%Y-%m-%d").to_string();
        let content = format!(
r#"---
tags:
  - type/idea
created: {today}
updated: {today}
type: idea
status: active
confidence: medium
---

# Not a manifest
"#
        );
        assert!(validate_manifest(&content).is_err());
    }

    #[test]
    fn validate_manifest_rejects_no_frontmatter() {
        let content = "# Just markdown\nNo frontmatter";
        assert!(validate_manifest(content).is_err());
    }

    #[test]
    fn vault_seed_files_have_valid_frontmatter() {
        let dir = tempfile::tempdir().unwrap();
        let vault_path = dir.path();
        init_vault(vault_path).unwrap();

        // Check each seed file has parseable frontmatter
        for seed in seed_files() {
            let file_path = vault_path.join(seed.path);
            let content = fs::read_to_string(&file_path).unwrap();
            let fm = Frontmatter::from_markdown(&content)
                .unwrap_or_else(|e| panic!("{}: frontmatter parse error: {e}", seed.path))
                .unwrap_or_else(|| panic!("{}: no frontmatter found", seed.path));
            assert!(fm.validate().is_ok(), "{}: frontmatter validation failed", seed.path);
        }
    }
}
