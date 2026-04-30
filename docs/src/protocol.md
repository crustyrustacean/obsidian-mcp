# Protocol Layer

The MCP protocol is implemented as a thin layer on top of JSON-RPC 2.0.

## JSON-RPC 2.0

All requests and responses follow the [JSON-RPC 2.0 spec](https://www.jsonrpc.org/specification):

- Every request has `jsonrpc: "2.0"`, an `id`, a `method`, and optional `params`
- Every response has `jsonrpc: "2.0"`, the echoed `id`, and either `result` or `error`
- The `jsonrpc` version field is validated at deserialization time and again at runtime (defense-in-depth)
- Invalid JSON returns `-32700` (ParseError)
- Unknown methods return `-32601` (MethodNotFound)
- Invalid parameters return `-32602` (InvalidParams)

## MCP Methods

The server supports three MCP methods:

### `initialize`

Returns server capabilities and protocol version.

```json
{"jsonrpc":"2.0","id":1,"method":"initialize","params":null}
→ {"jsonrpc":"2.0","id":1,"result":{
    "protocolVersion":"2024-11-05",
    "capabilities":{"tools":{}},
    "serverInfo":{"name":"obsidian-mcp","version":"0.5.0"}
}}
```

### `tools/list`

Returns descriptors for all 16 tools.

```json
{"jsonrpc":"2.0","id":2,"method":"tools/list","params":null}
→ {"jsonrpc":"2.0","id":2,"result":{"tools":[...]}}
```

Each descriptor has `name`, `description`, and `inputSchema` (JSON Schema).

### `tools/call`

Dispatches to a registered tool by name.

```json
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{
    "name":"obsidian_read_note",
    "arguments":{"path":"context-manifest.md"}
}}
→ {"jsonrpc":"2.0","id":3,"result":{
    "content":[{"type":"text","text":"# Context Manifest\n..."}]
}}
```

On error, the response includes `isError: true`:

```json
→ {"jsonrpc":"2.0","id":3,"error":{
    "code":-32000,
    "message":"Obsidian API returned HTTP 404: ...",
    "data":{"isError":true}
}}
```

## Error Codes

| Code | Meaning | When |
|------|---------|------|
| -32700 | ParseError | Invalid JSON in request |
| -32600 | InvalidRequest | Missing or wrong `jsonrpc` version |
| -32601 | MethodNotFound | Unknown method name |
| -32602 | InvalidParams | Missing required parameter, path traversal, etc. |
| -32000 | ServerError | Obsidian API error, rate limit, write verification failure |
