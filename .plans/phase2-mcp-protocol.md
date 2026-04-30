# Phase 2 — MCP Protocol Layer

**Objective:** Implement the MCP server transport so the binary is recognizable to MCP clients.

**Duration:** Week 2

**TDD Discipline:** Write failing tests before any implementation. Every task below follows the red → green → refactor cycle. No production code without a failing test that demands it.

## Tests (write first)

- [x] **Test: JSON-RPC 2.0 parsing — valid request** — assert a well-formed JSON-RPC request string deserializes into the `JsonRpcRequest` struct with correct `id`, `method`, and `params`.
- [x] **Test: JSON-RPC 2.0 parsing — invalid request** — assert malformed JSON or missing `jsonrpc: "2.0"` field returns a structured `InvalidRequest` error.
- [x] **Test: JSON-RPC 2.0 response serialization** — assert `JsonRpcResponse` and `JsonRpcError` structs serialize to spec-compliant JSON.
- [x] **Test: SSE endpoint accepts connection** — send an HTTP GET to the SSE path; assert `Content-Type: text/event-stream` and the connection stays open.
- [x] **Test: `tools/list` returns valid tool descriptors** — call `tools/list` via JSON-RPC; assert response is an array of objects each containing `name`, `description`, and `inputSchema`.
- [x] **Test: `tools/call` dispatches to registered tool** — register a stub tool; call `tools/call` with its name; assert the stub is invoked with the correct arguments.
- [x] **Test: `tools/call` with unknown tool name** — call `tools/call` with a nonexistent tool; assert a `MethodNotFound` error is returned (not a panic).
- [x] **Test: Obsidian API error surfaces as MCP tool error** — simulate a 500 from Obsidian; assert the MCP response contains an `isError: true` content block with the error message.
- [x] **Test: concurrent MCP requests** — send multiple `tools/list` calls in parallel; assert all return correct results without data races.

## Implementation (make tests pass)

- [x] Implement JSON-RPC 2.0 message types (Request, Response, Error, Notification)
- [x] Implement SSE transport layer (HTTP endpoint for MCP clients to connect)
- [x] Define tool registry: tool descriptors with `name`, `description`, and `input_schema` as JSON Schema
- [x] Implement `tools/list` handler — returns all registered tool descriptors
- [x] Implement `tools/call` handler — dispatches to the appropriate tool function
- [x] Wire in error handling: surface Obsidian API errors as MCP tool errors, never panics
- [x] Validate with MCP Inspector: `npx @modelcontextprotocol/inspector` *(manual — server verified via curl, inspector test deferred to Phase 5)*

## Security Review

- [x] **Input validation:** All JSON-RPC inputs are validated. Oversized payloads (>1 MB) rejected with 413. Unknown methods return `-32601` error — never fall through. `jsonrpc` version field validated at deserialization time.
- [x] **SSE connection limits:** Max 10 concurrent SSE connections enforced via `tokio::sync::Semaphore`. Returns 429 when exceeded.
- [x] **Request authentication:** SSE endpoint binds to `127.0.0.1` by default (localhost only). Trust boundary documented: if exposed beyond localhost, a bearer token must be added. `--host` flag available but not default.
- [x] **Error information leakage:** Error responses contain only error codes, messages, and optional `isError` flag. No stack traces, file paths, API keys, or internal state.
- [ ] **Rate limiting:** Not yet implemented. Deferred to Phase 5 (hardening). A misbehaving client could flood `tools/call` — noted as TODO.
- [x] **Tool registration sandboxing:** Only explicitly registered tools in the `HashMap` can be called. Arbitrary strings like `std::process::exit` or path-like inputs are rejected with `-32601`. Test confirms this.

## Deliverable

A binary that MCP Inspector can connect to and list tools against, backed by a full suite of protocol-level tests.

## Notes

- Decision: **Roll minimal MCP protocol** rather than adopt an immature Rust MCP library.
- SSE transport chosen over stdio for broader client compatibility (pi, Claude Desktop, Cursor, etc.).
- The MCP spec is straightforward; JSON-RPC 2.0 + SSE is estimated at ~200 lines.
- **TDD reminder:** Start each task by writing the test that defines "done." If the protocol spec is ambiguous, the test documents the intended behavior.
