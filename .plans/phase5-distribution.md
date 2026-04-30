# Phase 5 — Distribution & Hardening

**Objective:** Make the binary easy to deploy and reliable in practice.

**Duration:** Week 6

**TDD Discipline:** Write failing tests before any implementation. Hardening and reliability features are just as testable as core logic — simulate failures and assert graceful behavior before implementing the fix.

## Tests (write first)

### Graceful Degradation

- [x] **Test: Obsidian unreachable returns structured MCP error** — mock a connection refusal; assert the MCP tool returns `isError: true` with a descriptive message, not a panic or process exit.
- [x] **Test: Obsidian timeout returns structured MCP error** — mock a delayed response (> timeout threshold); assert a timeout error is returned gracefully.
- [x] **Test: recovery after transient failure** — mock two sequential calls: first fails, second succeeds; assert the server is still operational and the second call works.

### `--test-connection` Flag

- [x] **Test: `--test-connection` succeeds on healthy API** — covered by existing phase1_client tests.
- [x] **Test: `--test-connection` fails on unreachable API** — covered by existing phase1_client tests.
- [ ] **Test: `--test-connection` reports SSL issues** — mock an SSL handshake failure; assert the diagnostic message identifies the cert issue.

### Session-End Audit

- [x] **Test: audit detects missing session log** — simulate a session end that skips log writing; assert the audit flags the omission.
- [x] **Test: audit detects stale manifest** — simulate a session end without updating `context-manifest.md`; assert the audit flags the stale `updated` timestamp.
- [x] **Test: audit passes on correct session close** — simulate a well-formed session end; assert the audit reports all checks passing.

### Cross-Compilation & Binary Size

- [x] **Test: release binary size is under 5MB** — build in release mode; assert the binary file size < 5 MB. (3.96 MB with strip + panic=abort + codegen-units=1)
- [x] **Test: cross-compiled binaries run on target platforms** — at minimum, verify the current host's target binary starts and responds to `--help`.

### Config Snippets

- [x] **Test: each config snippet is valid JSON** — parse each MCP config snippet file; assert it is valid JSON with required fields.
- [x] **Test: config snippets reference correct binary path and args** — validate that the `command` and `args` fields in each snippet match the actual CLI interface.
- [x] **Test: config snippets have no hardcoded secrets** — scan all config JSON files for API key values.

## Implementation (make tests pass)

### Cross-Compilation

- [x] Configure `Cargo.toml` for minimal binary size: `opt-level = "z"`, `lto = true`, `strip = true`, `panic = "abort"`, `codegen-units = 1` (result: 3.96 MB)
- [ ] Set up cross-compile targets (requires cross-compilation toolchain, deferred to CI)

### MCP Config Snippets

- [x] Write config snippets for:
  - Claude Desktop (`config/claude-desktop.json`)
  - Claude Code CLI (`config/claude-code.json`)
  - Cursor / Windsurf (`config/cursor.json`)
  - Example .env (`config/.env.example`)

### Reliability

- [x] Graceful degradation: if Obsidian is unreachable, return a structured MCP error (not a crash) so the AI falls back to project state files
- [x] Add request timeout (30s) and connect timeout (10s) to HTTP client
- [x] `--test-connection` flag already implemented (Phase 1)
- [ ] Document Obsidian indexer lag: full-text search may not reflect programmatic writes immediately; prefer Dataview queries which are immediate (documented in README.md troubleshooting section)

### Session-End Audit

- [x] Implement session-end audit checklist: `src/audit.rs`
  - Checks session log existence
  - Validates manifest frontmatter
  - Detects stale manifest (updated date != today)

### Documentation

- [x] Write deployment README with setup instructions, config examples, and troubleshooting (`README.md`)

## Security Review

- [ ] **Release binary integrity:** Publish SHA-256 checksums alongside each release binary. Document how users can verify the binary matches the checksum.
- [x] **Stripped binaries:** `strip = true` in `[profile.release]` — 3.96 MB release binary
- [ ] **Dependency audit (final):** Run `cargo audit` and `cargo deny` on the final dependency tree. Record all findings. Zero critical/high advisories before release.
- [x] **Config snippet secrets:** MCP config snippets use environment variable references only (`${OBSIDIAN_API_KEY}`). Test verifies no hardcoded secrets.
- [ ] **Supply chain:** Pin exact dependency versions in `Cargo.lock`. Consider `cargo vet` or a reproducible build verification step.
- [x] **Network binding:** Default host is `127.0.0.1` — verified by `--help` output test and default in CLI args.
- [x] **SIGTERM/SIGINT handling:** Graceful shutdown already implemented in main.rs (Phase 2).
- [x] **Logging PII:** API key is never logged. Vault content is not logged. Only API URL is logged at startup.

## Deliverable

Single-binary release, MCP config examples, deployment README — all backed by passing reliability, audit, and hardening tests.
