# AGENTS.md — obsidian-mcp

## Project Overview

A Rust-native MCP (Model Context Protocol) server that bridges an Obsidian vault to any MCP-compatible AI client. Replaces the original Python/FastMCP implementation with a single statically-compiled binary using SSE transport.

## Architecture

- **Transport:** MCP over HTTP SSE (not stdio)
- **HTTP Client:** `reqwest` with `tokio` async runtime
- **JSON:** `serde` + `serde_json`
- **Error Handling:** `thiserror` for typed errors, `anyhow` for top-level propagation
- **Config:** `dotenvy` for `.env` loading (`OBSIDIAN_API_KEY`)
- **SSL:** Self-signed cert handling via `reqwest::ClientBuilder::danger_accept_invalid_certs(true)` or bundled cert

## Key Design Decisions

1. **Roll minimal MCP protocol** — Rust MCP ecosystem is too young; implementing JSON-RPC 2.0 + SSE transport manually (~200 lines)
2. **SSE transport** — not stdio, for broader client compatibility
3. **Per-tool headers** — each tool sets its own `Accept`/`Content-Type` explicitly; never share a single header set across endpoints
4. **Write verification** — after every write, immediately read back and verify byte count > 2 (guards against silent empty-write failures)
5. **Max 5 MCP calls at session start** — keep context budgets lean
6. **Dedicated vault** — new, standalone Obsidian vault (not a subfolder)
7. **Never panic in tool handlers** — surface Obsidian API errors as MCP tool errors

## Vault Structure

```
vault/
  context-manifest.md       # entry point; AI reads this first
  _index.md                 # Dataview live queries by type/status/recency
  content/
    ideas-bank.md
    audience-insights.md
    growth-experiments.md
  research/
  sessions/                 # one note per session, auto-written at session end
  troubleshooting/
```

## Frontmatter Schema (every note)

```yaml
tags:
  - project/my-rust-project
  - type/session-log
  - topic/mcp-server
created: YYYY-MM-DD
updated: YYYY-MM-DD
type: session-log
status: active
confidence: high
```

## Tool Inventory (16 tools)

| Group | Tools |
|-------|-------|
| Read/Write (6) | `obsidian_read_note`, `obsidian_read_note_metadata`, `obsidian_write_note`, `obsidian_append_note`, `obsidian_patch_note`, `obsidian_delete_note` |
| Search (3) | `obsidian_search`, `obsidian_dataview_query`, `obsidian_jsonlogic_query` |
| Batch/Convenience (3) | `obsidian_batch_read`, `obsidian_periodic_note`, `obsidian_recent_changes` |
| Navigate (4) | `obsidian_list_directory`, `obsidian_get_tags`, `obsidian_server_status`, `obsidian_open_note` |

## Build & Run

```bash
cargo build --release
cargo run -- --test-connection
```

## Constraints

- Release binary size target: < 5MB (`opt-level = "z"`, `lto = true`)
- Cross-compile targets: x86_64-unknown-linux-gnu, aarch64-apple-darwin, x86_64-pc-windows-msvc
