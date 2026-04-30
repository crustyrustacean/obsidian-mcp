# CLI Reference

```bash
obsidian-mcp [OPTIONS]
```

## Options

| Flag | Default | Description |
|------|---------|-------------|
| `--test-connection` | — | Test Obsidian API connectivity and exit |
| `--env-file <PATH>` | `.env` | Path to a `.env` file |
| `--transport <MODE>` | `stdio` | Transport mode: `stdio` or `sse` |
| `--host <ADDR>` | `127.0.0.1` | Bind address (SSE only) |
| `--port <PORT>` | `3000` | Listen port (SSE only) |
| `--help` | — | Show help |
| `--version` | — | Show version |

## Examples

### Test connectivity

```bash
obsidian-mcp --test-connection
```

### Stdio mode (default)

```bash
obsidian-mcp
```

For use with Claude Desktop, Claude Code, pi, MCP Inspector.

### SSE mode

```bash
obsidian-mcp --transport sse --port 3000
```

For use with Cursor, Windsurf, and HTTP-based clients.

### Custom .env path

```bash
obsidian-mcp --env-file /path/to/.env.production
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Connection test failed or fatal error |
