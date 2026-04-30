# Frontmatter Schema

Every note in the vault must have YAML frontmatter at the top:

```yaml
---
tags:
  - project/my-rust-project
  - type/session-log
  - topic/mcp-server
created: 2025-06-15
updated: 2025-06-15
type: session-log
status: active
confidence: high
---
```

## Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `tags` | string[] | yes | Must include at least one `type/` tag |
| `created` | date | yes | Date the note was created (YYYY-MM-DD) |
| `updated` | date | yes | Date the note was last modified (auto-set on write) |
| `type` | enum | yes | Note type (see below) |
| `status` | enum | yes | `active`, `completed`, or `abandoned` |
| `confidence` | enum | yes | `high`, `medium`, or `low` |

## Note Types

| Type | Purpose |
|------|---------|
| `session-log` | Session records in `sessions/` |
| `idea` | Ideas captured in `content/ideas-bank.md` |
| `research` | Research notes in `research/` |
| `troubleshooting` | Debug notes in `troubleshooting/` |
| `manifest` | The `context-manifest.md` entry point |
| `index` | The `_index.md` navigation page |

The `type` field is an enum — arbitrary values are rejected at deserialization time. This prevents typo-driven categorization drift.

## Validation Rules

1. `tags` must not be empty and must include at least one `type/` tag
2. `created` must be ≤ `updated` (no future-dated creates)
3. `type` must be one of the six known values
4. All dates use `YYYY-MM-DD` format

## Parsing

Frontmatter is parsed between `---` delimiters using `serde_yaml`. This is safe for typed deserialization — `serde_yaml` does not support dangerous YAML features like custom tags or arbitrary code execution.

The `Frontmatter::from_markdown()` method extracts and validates frontmatter from a markdown string. The `Frontmatter::to_markdown_prefix()` method serializes it back with delimiters.
