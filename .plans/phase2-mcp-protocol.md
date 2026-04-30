# Phase 2 — MCP Protocol Layer

**Objective:** Implement the MCP server transport so the binary is recognizable to MCP clients.

**Duration:** Week 2

**TDD Discipline:** Write failing tests before any implementation. Every task below follows the red → green → refactor cycle. No production code without a failing test that demands it.

## Tests (write first)

- [ ] **Test: JSON-RPC 2.0 parsing — valid request** — assert a well-formed JSON-RPC request string deserializes into the `JsonRpcRequest` struct with correct `id`, `method`, and `params`.
- [ ] **Test: JSON-RPC 2.0 parsing — invalid request** — assert malformed JSON or missing `jsonrpc: "2.0"` field returns a structured `InvalidRequest` error.
- [ ] **Test: JSON-RPC 2.0 response serialization** — assert `JsonRpcResponse` and `JsonRpcError` structs serialize to spec-compliant JSON.
- [ ] **Test: SSE endpoint accepts connection** — send an HTTP GET to the SSE path; assert `Content-Type: text/event-stream` and the connection stays open.
- [ ] **Test: `tools/list` returns valid tool descriptors** — call `tools/list` via JSON-RPC; assert response is an array of objects each containing `name`, `description`, and `inputSchema`.
- [ ] **Test: `tools/call` dispatches to registered tool** — register a stub tool; call `tools/call` with its name; assert the stub is invoked with the correct arguments.
- [ ] **Test: `tools/call` with unknown tool name** — call `tools/call` with a nonexistent tool; assert a `MethodNotFound` error is returned (not a panic).
- [ ] **Test: Obsidian API error surfaces as MCP tool error** — simulate a 500 from Obsidian; assert the MCP response contains an `isError: true` content block with the error message.
- [ ] **Test: concurrent MCP requests** — send multiple `tools/list` calls in parallel; assert all return correct results without data races.

## Implementation (make tests pass)

- [ ] Implement JSON-RPC 2.0 message types (Request, Response, Error, Notification)
- [ ] Implement SSE transport layer (HTTP endpoint for MCP clients to connect)
- [ ] Define tool registry: tool descriptors with `name`, `description`, and `input_schema` as JSON Schema
- [ ] Implement `tools/list` handler — returns all registered tool descriptors
- [ ] Implement `tools/call` handler — dispatches to the appropriate tool function
- [ ] Wire in error handling: surface Obsidian API errors as MCP tool errors, never panics
- [ ] Validate with MCP Inspector: `npx @modelcontextprotocol/inspector`

## Security Review

- [ ] **Input validation:** All JSON-RPC inputs must be validated before processing. Reject oversized payloads (set a max body size, e.g., 1 MB). Reject unknown methods with standard errors — never fall through to default handling.
- [ ] **SSE connection limits:** Enforce a maximum number of concurrent SSE connections to prevent resource exhaustion. Consider a configurable limit (default: 10).
- [ ] **Request authentication:** Determine whether the SSE endpoint should require an auth token. If exposed on `localhost` only, document the trust boundary. If exposed beyond localhost, require a bearer token.
- [ ] **Error information leakage:** Ensure error responses do not include stack traces, file paths, internal state, or API keys. Errors should be human-readable but not system-revealing.
- [ ] **Rate limiting:** Implement basic rate limiting on `tools/call` to prevent a misbehaving MCP client from flooding the Obsidian API.
- [ ] **Tool registration sandboxing:** Only explicitly registered tools can be called — there must be no way to invoke arbitrary functions or execute raw strings.

## Deliverable

A binary that MCP Inspector can connect to and list tools against, backed by a full suite of protocol-level tests.

## Notes

- Decision: **Roll minimal MCP protocol** rather than adopt an immature Rust MCP library.
- SSE transport chosen over stdio for broader client compatibility (pi, Claude Desktop, Cursor, etc.).
- The MCP spec is straightforward; JSON-RPC 2.0 + SSE is estimated at ~200 lines.
- **TDD reminder:** Start each task by writing the test that defines "done." If the protocol spec is ambiguous, the test documents the intended behavior.
