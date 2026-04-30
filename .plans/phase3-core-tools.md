# Phase 3 — Core Tools (16 Tools)

**Objective:** Implement all 16 tools. Each tool must set its own headers explicitly — do not share a single header set across all endpoints.

**Duration:** Weeks 3–4

**TDD Discipline:** Write failing tests before any implementation. Every tool follows the red → green → refactor cycle. For each tool: (1) write a test defining the expected request/response shape against a mock Obsidian API, (2) write the minimum code to pass, (3) refactor. Integration tests against a real vault come after all unit tests are green.

## Read/Write Group (6 tools)

### Tests (write first)

- [ ] **Test: `obsidian_read_note` sends `Accept: text/markdown`** — mock `GET /vault/{path}`; assert the `Accept` header is `text/markdown` and the response body is returned as a string.
- [ ] **Test: `obsidian_read_note_metadata` sends `Accept: application/json`** — mock `GET /vault/{path}`; assert the `Accept` header is `application/json` and the response is parsed as JSON frontmatter.
- [ ] **Test: `obsidian_write_note` sends `Content-Type: text/markdown`** — mock `PUT /vault/{path}`; assert the content-type header and that the body matches the input content.
- [ ] **Test: `obsidian_write_note` verifies write (read-back)** — after a mock PUT, assert a follow-up GET is issued and the byte count > 2 is checked. Simulate an empty write and assert an error is returned.
- [ ] **Test: `obsidian_append_note` sends `POST`** — mock `POST /vault/{path}`; assert the content is appended, not replaced.
- [ ] **Test: `obsidian_patch_note` targets a heading** — mock `PATCH /vault/{path}`; assert the heading and content are sent correctly.
- [ ] **Test: `obsidian_delete_note` sends `DELETE`** — mock `DELETE /vault/{path}`; assert a 204 or 200 response is handled.

### Implementation (make tests pass)

- [ ] `obsidian_read_note(path)` — `GET /vault/{path}`, `Accept: text/markdown`
- [ ] `obsidian_read_note_metadata(path)` — `GET /vault/{path}`, `Accept: application/json`
- [ ] `obsidian_write_note(path, content)` — `PUT /vault/{path}`, `Content-Type: text/markdown`
- [ ] `obsidian_append_note(path, content)` — `POST /vault/{path}`
- [ ] `obsidian_patch_note(path, heading, content)` — heading-level update via `PATCH`
- [ ] `obsidian_delete_note(path)` — `DELETE /vault/{path}`

## Search Group (3 tools)

### Tests (write first)

- [ ] **Test: `obsidian_search` encodes query** — mock `GET` search endpoint; assert the query string is URL-encoded and results are returned as an array.
- [ ] **Test: `obsidian_dataview_query` sends correct content-type** — mock `POST /search/`; assert `Content-Type: application/vnd.olrapi.dataview.dql+txt` and that the DQL body is sent as plain text.
- [ ] **Test: `obsidian_jsonlogic_query` sends JSON body** — mock `POST /search/`; assert the JSONLogic payload is serialized correctly and the content-type is set appropriately.

### Implementation (make tests pass)

- [ ] `obsidian_search(query)` — full-text search
- [ ] `obsidian_dataview_query(dql)` — `POST /search/`, `Content-Type: application/vnd.olrapi.dataview.dql+txt`
- [ ] `obsidian_jsonlogic_query(logic)` — JSONLogic search variant

## Batch/Convenience Group (3 tools)

### Tests (write first)

- [ ] **Test: `obsidian_batch_read` parallel reads** — mock multiple `GET /vault/{path}` endpoints; assert all are called concurrently and results are collected into a `HashMap<String, String>`.
- [ ] **Test: `obsidian_batch_read` partial failure** — mock one path returning 404 and another succeeding; assert the success result is returned and the failure is reported without panicking.
- [ ] **Test: `obsidian_periodic_note` routes by period** — mock daily/weekly/monthly endpoints; assert the correct one is called based on the `period` argument.
- [ ] **Test: `obsidian_recent_changes` returns full paths** — set up a mock directory tree with known mtimes; assert results use full (not relative) paths and are sorted by most recent first.

### Implementation (make tests pass)

- [ ] `obsidian_batch_read(paths[])` — parallel reads with `futures::future::join_all`, return map of path → content
- [ ] `obsidian_periodic_note(period)` — daily/weekly/monthly via Periodic Notes plugin endpoint
- [ ] `obsidian_recent_changes(limit)` — walk vault directory tree, sort by mtime, return top N (use full paths, not relative — silent bug in original)

## Navigate Group (4 tools)

### Tests (write first)

- [ ] **Test: `obsidian_list_directory` returns file/folder list** — mock `GET /vault/{path}/`; assert the response is parsed into a structured list of entries with names and types.
- [ ] **Test: `obsidian_get_tags` returns hierarchy** — mock the tags endpoint; assert tags with counts are returned.
- [ ] **Test: `obsidian_server_status` health check** — mock the status endpoint; assert a healthy response returns `Ok(())` and an unhealthy one returns a structured error.
- [ ] **Test: `obsidian_open_note` triggers UI open** — mock the open endpoint; assert the correct path is sent and the response is acknowledged.

### Implementation (make tests pass)

- [ ] `obsidian_list_directory(path)` — `GET /vault/{path}/`
- [ ] `obsidian_get_tags()` — retrieve tag hierarchy with counts
- [ ] `obsidian_server_status()` — health check
- [ ] `obsidian_open_note(path)` — trigger Obsidian UI to open a note

## Critical Implementation Note

After each write, immediately read the note back and verify byte count > 2. The original article lost an entire session of work to silent empty-write failures.

## Integration Tests (after all unit tests pass)

- [ ] All 16 tools pass against MCP Inspector with a real vault
- [ ] Write verification (read-back check) works end-to-end with a real vault
- [ ] `obsidian_recent_changes` returns full paths (not relative) with a real vault

## Security Review

- [ ] **Path traversal:** All `path` parameters must be validated to prevent directory traversal (e.g., `../../etc/passwd`). Reject paths containing `..`, absolute paths, or null bytes. Canonicalize and verify the resolved path stays within the vault root.
- [ ] **Write/append/patch content sanitization:** Ensure content written to the vault cannot inject Obsidian directives that execute code (e.g., `dataviewjs` blocks) unless explicitly intended. Document the risk.
- [ ] **Batch read amplification:** `obsidian_batch_read` must enforce a maximum number of paths (e.g., 20) to prevent a single call from spawning hundreds of parallel requests.
- [ ] **Dataview query injection:** `obsidian_dataview_query` accepts raw DQL. Document that the caller is responsible for query safety — Obsidian's Dataview plugin has its own sandbox, but queries can still be resource-intensive.
- [ ] **Delete confirmation:** Consider requiring an explicit `confirm: true` flag in `obsidian_delete_note` to prevent accidental deletion via a misformatted MCP call.
- [ ] **Rate limiting per tool:** Ensure no single tool can be called in a tight loop that would DOS the Obsidian REST API. Implement per-tool rate limits or a global cooldown.
- [ ] **Error response sanitization:** Ensure Obsidian API error responses (which may contain file system paths or internal details) are sanitized before being returned as MCP tool errors.

## Deliverable

All 16 tools passing unit tests (against mock API) and integration tests (against a real vault via MCP Inspector).
