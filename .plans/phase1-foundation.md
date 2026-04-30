# Phase 1 — Foundation & Tooling

**Objective:** Stand up the project structure and confirm connectivity to Obsidian's Local REST API.

**Duration:** Week 1

**TDD Discipline:** Write failing tests before any implementation. Every task below follows the red → green → refactor cycle. No production code without a failing test that demands it.

## Tests (write first)

- [x] **Test: HTTP client sends correct `Accept: text/markdown` header** — mock the Obsidian API endpoint; assert the outbound request carries the header before any client code exists.
- [x] **Test: SSL self-signed cert is accepted** — spin up a local HTTPS server with a self-signed cert; assert the client connects without certificate errors.
- [x] **Test: `.env` loading populates config struct** — write a temp `.env` file; assert `OBSIDIAN_API_KEY` and `OBSIDIAN_API_URL` are parsed into a config struct with correct defaults.
- [x] **Test: API key is included as `Authorization: Bearer` header** — mock endpoint; assert the key appears in outbound requests.
- [x] **Test: smoke-test read returns markdown content** — integration test against a real or mock vault; assert the response body is valid markdown.

## Implementation (make tests pass)

- [x] Initialize workspace: `cargo new obsidian-mcp --bin` ✅ (done)
- [x] Add dependencies to `Cargo.toml`: `reqwest` (with `tokio` async runtime), `serde` + `serde_json`, `tokio`, `dotenvy`, `thiserror`, `anyhow`
- [x] Add `wiremock` as a `[dev-dependency]` for HTTP mocking
- [ ] Install Obsidian plugins: Local REST API (required) and Dataview (required); optionally Periodic Notes *(manual step — user must install in Obsidian)*
- [x] Implement the Obsidian API client module: config loading, client builder with SSL + auth headers, `read_note()` method
- [x] Write the smoke-test binary entry point that reads a known vault note
- [x] Handle self-signed SSL cert: use `reqwest::ClientBuilder::danger_accept_invalid_certs(true)` or bundle the cert
- [x] Create `.env.example` with `OBSIDIAN_API_KEY` and `OBSIDIAN_API_URL` placeholders

## Security Review

- [x] **API key exposure:** Confirmed `OBSIDIAN_API_KEY` is never logged, printed, or included in error messages. Audited all `tracing`/`println` calls. Only appears in `Authorization: Bearer` header construction.
- [x] **`.env` in `.gitignore`:** Verified `.env` is in `.gitignore` and will never be committed.
- [x] **TLS verification scope:** `danger_accept_invalid_certs(true)` disables cert verification globally for this client. Documented the risk in `src/client.rs` inline comment. TODO: Consider scoped approach (cert fingerprint pinning or loading from plugin data directory) in future phase.
- [x] **Secrets in process args:** No API key CLI flag exists. Only `--test-connection` and `--env-file` flags. API key loaded from `.env` file or env vars only.
- [x] **Dependency audit:** `cargo audit` reports zero vulnerabilities across 261 dependencies.

## Deliverable

A Rust binary that can read a markdown note from the live vault, backed by a full suite of unit + integration tests.

## Notes

- Obsidian's Local REST API requires per-endpoint content-type headers (e.g., `Accept: text/markdown`) that auto-generation cannot set per-request — this was a key pain point in the Python version.
- The API uses a self-signed SSL cert; handle this early.
- **TDD reminder:** If a test is hard to write for a piece of functionality, that's a design signal — refactor the interface until it's testable.
