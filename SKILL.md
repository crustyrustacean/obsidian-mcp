# Obsidian MCP Server — AI Skill File

This document instructs an AI client on how to use the Obsidian MCP server
effectively. It defines session protocols and hard rules for context management.

## Session Start Protocol

**Hard rule: Maximum 5 MCP calls at session start.** This keeps context budgets lean.

1. **Always** read `context-manifest.md` first (1 call). This is your entry point.
2. Based on the task type, load additional files:

| Task Type | Keywords | Additional Files | Total Calls |
|-----------|----------|-----------------|-------------|
| Coding / Debugging | debug, error, bug, fix, compile, build fail | `troubleshooting/` listing | 2 |
| Planning | plan, design, architect, roadmap, strategy | `content/ideas-bank.md`, `content/growth-experiments.md` | 3 |
| General | (everything else) | manifest only | 1 |

### Do NOT

- Do NOT read the entire vault at session start.
- Do NOT make more than 5 MCP calls during initialization.
- Do NOT modify the skill file (`SKILL.md`) programmatically.

## Session End Protocol

When a session concludes:

1. Write a session log to `sessions/YYYY-MM-DD-HHMM.md` using the session-log frontmatter schema.
2. Update `context-manifest.md` with:
   - A reference to the new session log
   - An updated `updated:` date in frontmatter
3. Grep known facts across files for drift (optional, for long sessions).

## Frontmatter Schema

Every note in the vault must have this frontmatter:

```yaml
tags:
  - project/<name>
  - type/<type>
  - topic/<topic>
created: YYYY-MM-DD
updated: YYYY-MM-DD
type: session-log | idea | research | troubleshooting | manifest | index
status: active | completed | abandoned
confidence: high | medium | low
```

- `updated` is auto-set on write operations.
- `tags` must include at least one `type/` tag.
- `type` is an enum — arbitrary values are rejected.

## Write Safety

- All write operations (write/append/patch) verify the write by reading the note back.
- **Dataviewjs risk**: Content written to the vault can contain Obsidian directives
  like `dataviewjs` blocks which execute JavaScript. Only write content you intend
  to be executable. If uncertain, avoid `dataviewjs` blocks.
- Delete operations require `confirm: true`.

## Rate Limiting

Each tool has a rate limit of 2 requests/second sustained with a burst of 5.
If you hit the limit, wait the indicated `retry_after` duration before retrying.

## Vault Structure

```
vault/
  context-manifest.md       # Entry point — read this first
  _index.md                 # Dataview queries for navigation
  content/
    ideas-bank.md           # Capture ideas
    growth-experiments.md   # Track experiments
  research/                 # Research notes
  sessions/                 # One note per session
  troubleshooting/          # Debugging notes
```

## Security

- Path traversal is blocked (`..`, absolute paths, null bytes rejected).
- Batch reads are limited to 20 paths per call.
- Search queries are limited to 1000 characters.
- The vault is a dedicated, standalone directory — no tool can access files outside it.
