# Dual Transport

obsidian-mcp supports two MCP transport modes, chosen via the `--transport` flag:

## stdio (default)

```
Client ←stdin/stdout→ obsidian-mcp ←HTTPS→ Obsidian REST API
```

The binary reads newline-delimited JSON-RPC from stdin and writes responses to stdout. Log output goes to stderr only — stdout is reserved for MCP protocol traffic.

**When to use:** Claude Desktop, Claude Code, pi, MCP Inspector, any client that launches the binary as a subprocess.

**MCP handshake:**

```jsonl
{"jsonrpc":"2.0","id":1,"method":"initialize","params":null}
{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"2024-11-05","capabilities":{"tools":{}},"serverInfo":{"name":"obsidian-mcp","version":"0.5.0"}}}
{"jsonrpc":"2.0","method":"notifications/initialized"}
{"jsonrpc":"2.0","id":2,"method":"tools/list","params":null}
{"jsonrpc":"2.0","id":2,"result":{"tools":[...]}}
```

**Notifications** (requests without an `id` field) are silently acknowledged without writing a response, per the JSON-RPC 2.0 spec.

## SSE

```
Client ←HTTP/SSE→ obsidian-mcp ←HTTPS→ Obsidian REST API
```

The binary starts an Axum HTTP server with two endpoints:

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/sse` | GET | Server-Sent Events stream for server-initiated messages |
| `/mcp` | POST | JSON-RPC 2.0 request/response |

**When to use:** Cursor, Windsurf, and any HTTP-based MCP client.

**Limits:** Max 10 concurrent SSE connections (returns 429 when exceeded). Max 1 MB request body (returns 413 when exceeded).

## Choosing a Transport

| Client | Transport | Config |
|--------|-----------|--------|
| pi | stdio | `.pi/extensions/obsidian/` |
| Claude Desktop | stdio | `command` + `args` in `claude_desktop_config.json` |
| Claude Code | stdio | `command` + `args` |
| Cursor | SSE | `url` field in MCP settings |
| MCP Inspector | stdio | Launches as subprocess |
