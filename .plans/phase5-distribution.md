# Phase 5 — Distribution & Hardening

**Objective:** Make the binary easy to deploy and reliable in practice.

**Duration:** Week 6

**TDD Discipline:** Write failing tests before any implementation. Hardening and reliability features are just as testable as core logic — simulate failures and assert graceful behavior before implementing the fix.

## Tests (write first)

### Graceful Degradation

- [ ] **Test: Obsidian unreachable returns structured MCP error** — mock a connection refusal; assert the MCP tool returns `isError: true` with a descriptive message, not a panic or process exit.
- [ ] **Test: Obsidian timeout returns structured MCP error** — mock a delayed response (> timeout threshold); assert a timeout error is returned gracefully.
- [ ] **Test: recovery after transient failure** — mock two sequential calls: first fails, second succeeds; assert the server is still operational and the second call works.

### `--test-connection` Flag

- [ ] **Test: `--test-connection` succeeds on healthy API** — mock a healthy Obsidian API; assert exit code 0 and a success message.
- [ ] **Test: `--test-connection` fails on unreachable API** — mock a connection refusal; assert non-zero exit code and a diagnostic message (no panic).
- [ ] **Test: `--test-connection` reports SSL issues** — mock an SSL handshake failure; assert the diagnostic message identifies the cert issue.

### Session-End Audit

- [ ] **Test: audit detects missing session log** — simulate a session end that skips log writing; assert the audit flags the omission.
- [ ] **Test: audit detects stale manifest** — simulate a session end without updating `context-manifest.md`; assert the audit flags the stale `updated` timestamp.
- [ ] **Test: audit passes on correct session close** — simulate a well-formed session end; assert the audit reports all checks passing.

### Cross-Compilation & Binary Size

- [ ] **Test: release binary size is under 5MB** — build in release mode; assert the binary file size < 5 MB.
- [ ] **Test: cross-compiled binaries run on target platforms** — at minimum, verify the current host's target binary starts and responds to `--help`.

### Config Snippets

- [ ] **Test: each config snippet is valid JSON** — parse each MCP config snippet file; assert it is valid JSON with required fields.
- [ ] **Test: config snippets reference correct binary path and args** — validate that the `command` and `args` fields in each snippet match the actual CLI interface.

## Implementation (make tests pass)

### Cross-Compilation

- [ ] Set up cross-compile targets:
  - `x86_64-unknown-linux-gnu`
  - `aarch64-apple-darwin`
  - `x86_64-pc-windows-msvc`
- [ ] Configure `Cargo.toml` for minimal binary size: `opt-level = "z"`, `lto = true` (target: < 5MB)

### MCP Config Snippets

- [ ] Write config snippets for:
  - Claude Desktop
  - Claude Code CLI
  - Cursor / Windsurf
  - pi (if applicable via extension)

### Reliability

- [ ] Graceful degradation: if Obsidian is unreachable, return a structured MCP error (not a crash) so the AI falls back to project state files
- [ ] Add `--test-connection` flag for quick connectivity diagnostics
- [ ] Document Obsidian indexer lag: full-text search may not reflect programmatic writes immediately; prefer Dataview queries which are immediate

### Session-End Audit

- [ ] Implement session-end audit checklist: grep for tool counts, version strings, and self-describing facts across all files before closing

### Documentation

- [ ] Write deployment README with setup instructions, config examples, and troubleshooting

## Security Review

- [ ] **Release binary integrity:** Publish SHA-256 checksums alongside each release binary. Document how users can verify the binary matches the checksum.
- [ ] **Stripped binaries:** Ensure release builds strip debug symbols (`strip = true` in `Cargo.toml` `[profile.release]`) to reduce attack surface and binary size.
- [ ] **Dependency audit (final):** Run `cargo audit` and `cargo deny` on the final dependency tree. Record all findings. Zero critical/high advisories before release.
- [ ] **Config snippet secrets:** MCP config snippets must not contain hardcoded API keys. Use environment variable references only. Audit all snippet files.
- [ ] **Supply chain:** Pin exact dependency versions in `Cargo.lock`. Consider `cargo vet` or a reproducible build verification step.
- [ ] **Network binding:** Verify the SSE server binds only to `127.0.0.1` (localhost) by default — never `0.0.0.0`. Document how to change the bind address if needed, with a clear security warning.
- [ ] **SIGTERM/SIGINT handling:** Ensure the server shuts down gracefully on signal, closing connections and flushing any in-flight responses — no orphaned connections or partial writes.
- [ ] **Logging PII:** Ensure log output (at all levels) does not include vault note content, API keys, or file paths beyond what is necessary for debugging. Redact sensitive fields in structured logs.

## Deliverable

Single-binary release, MCP config examples, deployment README — all backed by passing reliability, audit, and hardening tests.
