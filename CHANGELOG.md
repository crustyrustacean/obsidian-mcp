# Changelog

All notable changes to this project will be documented in this file.

## [1.0.0] - 2026-04-30

### 🚀 Features

- Phase 1: foundation & tooling — config, Obsidian API client, SSL, 11 tests

- Config module: loads OBSIDIAN_API_KEY and OBSIDIAN_API_URL from .env
  file (parsed directly, no process-env pollution) with defaults
- ObsidianClient: reqwest-based HTTP client with self-signed SSL cert
  acceptance, Bearer token auth, per-request Accept headers
- ObsidianError: typed error enum (API, connection, config,
  write-verification) — no panics in tool handlers
- CLI binary: --test-connection and --env-file flags, tracing setup
- AGENTS.md: project context, architecture decisions, tool inventory
- .plans/: 5-phase plan with TDD emphasis and security review sections
- .env.example: template for required config vars

Tests (all passing):
  phase1_config: env loading, default URL, missing key error
  phase1_client: Accept header, Bearer auth, markdown content,
    404/401/unreachable error handling
  phase1_ssl: self-signed cert acceptance (full TLS server via rcgen)

Security review (all clear):
  - API key never logged/printed
  - .env in .gitignore
  - TLS risk documented inline
  - No CLI flag secrets
  - cargo audit: zero vulnerabilities

Release binary: 3.8 MB (under 5 MB target)
- Phase 2: MCP protocol layer — JSON-RPC 2.0, SSE transport, tool registry, 17 tests

- protocol.rs: JSON-RPC 2.0 message types (Request, Response, Error,
  RequestId) with custom deserializer enforcing jsonrpc: "2.0" at
  parse time. Standard error codes. Response builder helpers.
- server.rs: Axum-based MCP server with two endpoints:
  - POST /mcp — JSON-RPC request/response (initialize, tools/list,
    tools/call, unknown method returns -32601)
  - GET /sse — Server-Sent Events stream with connection limiting
    (max 10 concurrent via tokio::sync::Semaphore)
  - Max body size: 1 MB (413 on oversize)
  - Graceful shutdown on SIGINT/SIGTERM
- tools.rs: Tool registry with register, list, and async dispatch.
  Only explicitly registered handlers are callable (sandboxing).
  ToolDescriptor with name, description, inputSchema.
  tool_result_content() and tool_error_content() helpers for MCP format.
- main.rs: Server startup with --host and --port flags (defaults
  127.0.0.1:3000). Removed dotenvy dependency (direct .env parsing
  from Phase 1).
- AGENTS.md: Updated with module layout, CLI flags, architecture

Tests (all passing):
  phase2_jsonrpc: 8 tests — valid/invalid request parsing,
    response serialization, string IDs, version validation,
    standard error codes
  phase2_server: 9 tests — SSE endpoint, tools/list, tools/call
    dispatch, unknown tool/method errors, parse error, oversized
    payload, Obsidian error surfacing with isError, concurrent requests
  phase2_tools: 4 tests — registry list, handler dispatch,
    unknown tool error, sandboxing (arbitrary strings rejected)

Security review:
  - Input validation: jsonrpc version enforced at deserialization,
    max body 1MB, unknown methods rejected
  - SSE connection limit: 10 concurrent (Semaphore), 429 on exceed
  - Localhost-only by default (--host flag available)
  - No info leakage in error responses
  - Tool sandboxing: only registered handlers callable
  - Rate limiting: deferred to Phase 5
  - cargo audit: zero vulnerabilities
  - cargo clippy: zero warnings
- Phase 3: implement all 16 MCP tools with handlers, security validation, 35 tests

- Read/Write group (6): read_note, read_note_metadata, write_note,
  append_note, patch_note, delete_note
- Search group (3): search, dataview_query, jsonlogic_query
- Batch/Convenience (3): batch_read, periodic_note, recent_changes
- Navigate group (4): list_directory, get_tags, server_status, open_note

Security features:
- Path validation: reject traversal (..), absolute paths, null bytes, >512 chars
- Write verification: read-back after every write, check byte count > 2
- Batch limit: max 20 paths per batch_read call
- Delete confirmation: requires confirm=true flag
- Error sanitization: truncate API error responses to 200 chars

Tool registration via tools_impl::register_all_tools() wires all 16
tools with handlers to the ToolRegistry. All per-tool Accept/Content-Type
headers set explicitly — never shared across endpoints.

Fix: rcgen 0.14 CertifiedKey API change (key_pair → signing_key) in
phase1_ssl test.

74 tests total passing (8 unit + 6+3+1 phase1 + 8+9+4 phase2 + 35 phase3).
- Phase 4: vault structure, frontmatter schema, session protocols, SKILL.md; bump v0.4.0

New modules:
- src/frontmatter.rs: Typed frontmatter schema with serde_yaml parsing,
  validation (required tags, type/ tag, date range), markdown roundtrip
  serialization, enum-enforced note types (session-log, idea, research,
  troubleshooting, manifest, index)
- src/vault.rs: Vault directory initializer (idempotent), structure
  validation, session protocols (TaskType routing table), session log
  generation, manifest validation

Vault files:
- vault/context-manifest.md: AI entry point with frontmatter
- vault/_index.md: Dataview queries for navigation
- vault/content/ideas-bank.md: Ideas capture
- vault/content/growth-experiments.md: Experiment tracking
- vault/research/, sessions/, troubleshooting/: Empty dirs with .gitkeep

SKILL.md: AI client instructions documenting session start/end protocols,
routing table (coding → troubleshooting, planning → content, general →
manifest), max 5 MCP calls rule, frontmatter schema, write safety,
rate limits, and vault structure.

Removed audience-insights.md from vault structure per user request.

New dependencies: chrono (serde feature), serde_yaml
113 tests total, zero clippy warnings.
- Phase 5: distribution, hardening, audit module, config snippets, README; bump v0.5.0

Hardening:
- Add strip=true, panic=abort, codegen-units=1 to [profile.release]
- Release binary: 3.96 MB (under 5MB target)
- Add request timeout (30s) and connect timeout (10s) to HTTP client
- Graceful degradation tests: unreachable API, timeout, transient failure recovery

Session-end audit:
- New src/audit.rs module: checks session log existence, manifest
  validity, manifest freshness (updated date == today)
- AuditResult struct with warnings list and all_passed() check

MCP config snippets:
- config/claude-desktop.json (stdio transport)
- config/claude-code.json (stdio transport)
- config/cursor.json (SSE transport)
- config/.env.example (template, no secrets)
- Test: all snippets are valid JSON with required fields
- Test: no hardcoded API keys in any config file

Documentation:
- README.md: full deployment guide with quick start, client config
  examples (Claude Desktop, Cursor), CLI flags, tool inventory,
  security notes, vault structure, cross-compilation, troubleshooting

Tests: 130 total (44 unit + 6+3+1+8+9+4 phase1/2 + 42 phase3 + 13 phase5)
Zero clippy warnings.
- Add stdio transport — the primary MCP transport mode

The binary was SSE-only, but stdio is the dominant MCP transport
(Claude Desktop, Claude Code, mcp CLI, MCP Inspector all use stdio).

Changes:
- New src/stdio.rs: reads NDJSON from stdin, dispatches JSON-RPC,
  writes responses to stdout. Notifications (no id) get no response.
- Extract dispatch_request() from server.rs mcp_handler — shared by
  both SSE and stdio transports. Handlers now take &ToolRegistry
  instead of &AppState.
- New --transport flag: 'stdio' (default) or 'sse'
- Tracing writer explicitly set to stderr (was default but implicit —
  stdout must be reserved for MCP JSON-RPC in stdio mode)
- Config snippets fixed: Claude Desktop/Code use command+args (stdio),
  Cursor uses SSE url
- README updated with dual transport docs

Tests: 140 total (5 new stdio unit + 4 subprocess integration + 1
transport default check). Zero clippy warnings. Binary: 3.8 MB.
- Add pi extension: TypeScript bridge for obsidian-mcp stdio transport

pi doesn't speak MCP natively — it uses TypeScript extensions with
pi.registerTool(). This extension spawns the obsidian-mcp binary in
stdio mode, performs the MCP initialize handshake, and registers all
16 Obsidian tools as pi custom tools.

Features:
- Auto-spawns on session_start, cleans up on session_shutdown
- MCP initialize handshake + notifications/initialized
- Per-tool JSON-RPC forwarding with 30s timeout
- Stderr from binary monitored for WARN/ERROR
- Binary path configurable via OBSIDIAN_MCP_BINARY env var
- Add mdbook documentation (22 chapters)

docs/ structure:
- Introduction, Quick Start, Architecture
- Module Layout, Dual Transport, Protocol Layer
- 4 tool chapters (read/write, search, batch, navigate)
- 4 client chapters (pi, Claude Desktop, Cursor, MCP Inspector)
- 3 vault chapters (layout, frontmatter schema, session protocols)
- Security, Configuration, CLI Reference
- Building & Deploying, Troubleshooting

Build with: cd docs && mdbook build
Read with: cd docs && mdbook serve
- Add MIT License.txt; update README with pi client section, --port flag, docs link

### 🐛 Bug Fixes

- Fix reqwest rustls-tls feature flag (renamed to rustls in 0.13); bump v0.2.1

### 🏗️ Refactor

- Update AGENTS.md: reflect stdio primary transport, new modules, pi extension

### 🔒 Security

- Code review fixes: security hardening, rate limiting, dead deps removed

Security fixes:
- S1 (High): URL-encode all vault paths and open_note paths via
  urlencoding::encode() to prevent path injection via percent-encoding
- S2: Add query length limits (1000 chars) for search/dataview queries
  with new ContentTooLong error variant
- S6: Dedicated DeleteConfirmationRequired error variant instead of
  misusing InvalidPath for delete confirm=false
- S7: Document dataviewjs injection risk in write/append/patch tool
  descriptors and client method doc comments

Rate limiting:
- Wire up flux-limiter (GCRA) for per-tool rate limiting
- New rate_limiter module: 2 req/s sustained, burst of 5 per tool
- Each tool handler checks rate limit before dispatching
- RateLimitExceeded error now includes retry_after_seconds

Dependency cleanup:
- Remove async-trait (unused)
- Remove axum-server (unused)
- Move tempfile from [dependencies] to [dev-dependencies]

Functional fixes:
- recent_changes: implement recursive directory walking with mtime-based
  sorting (was a stub that just returned first N entries)
- batch_read: pre-validate all paths before making HTTP calls
- Use clamp() instead of min().max() per clippy

Tests: 89 total (16 unit + 6+3+1+8+9+4 phase1/2 + 42 phase3)
- New: URL-encoding path tests, delete confirmation error test,
  query length limit tests, rate limiter unit tests
- Zero clippy warnings

### ⚙️ Miscellaneous

- Bump version to 0.2.0
- Bump version to 0.3.0
- Bump version to 0.3.1
- Bump version to 0.4.1
- Upgrade to Rust edition 2024; collapse nested if-let chain

Edition 2024 is clean — 113 tests pass, zero clippy warnings.
Only code change: collapsed 3-level nested if-let in
collect_files_recursive into a single if-let chain using
edition 2024's let-chains in if expressions.
- Add git-cliff and cargo-release configuration

<!-- generated by git-cliff -->
