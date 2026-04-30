# Module Layout

| Module | Responsibility |
|--------|---------------|
| `src/stdio.rs` | Stdio transport: reads NDJSON from stdin, dispatches, writes to stdout |
| `src/server.rs` | Axum app (SSE), shared `dispatch_request()`, `AppState`, connection limiting |
| `src/protocol.rs` | JSON-RPC 2.0 types (`Request`, `Response`, `Error`, `RequestId`), validation |
| `src/tools.rs` | Tool registry (`register`, `list`, `dispatch`), `ToolDescriptor`, `ToolHandler` |
| `src/tools_impl.rs` | 16 tool descriptors + handler registration |
| `src/client.rs` | Obsidian Local REST API client (all 16 tool methods) |
| `src/config.rs` | `Config` struct, `.env` file parsing, defaults |
| `src/error.rs` | `ObsidianError` enum, path/query validation functions |
| `src/rate_limiter.rs` | Per-tool GCRA rate limiting via `flux-limiter` |
| `src/frontmatter.rs` | YAML frontmatter schema, parsing, validation |
| `src/vault.rs` | Vault init, directory layout, session protocols, routing table |
| `src/audit.rs` | Session-end audit: session log presence, manifest freshness |

### Supporting Files

| Path | Purpose |
|------|---------|
| `SKILL.md` | AI client instructions for session protocols |
| `.pi/extensions/obsidian/` | TypeScript pi extension — spawns binary in stdio mode |
| `config/` | MCP config snippets for each client |
| `vault/` | Seed vault structure (directories + template files) |
