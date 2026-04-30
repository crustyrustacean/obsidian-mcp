# obsidian-mcp

A Rust-native MCP (Model Context Protocol) server that bridges an Obsidian vault to any MCP-compatible AI client. Single statically-compiled binary using SSE transport.

## Prerequisites

- [Obsidian](https://obsidian.md) with the [Local REST API](https://github.com/coddingtonbear/obsidian-local-rest-api) plugin installed and enabled
- The Local REST API plugin's API key (found in Obsidian → Settings → Community plugins → Local REST API)
- An MCP-compatible AI client (Claude Desktop, Cursor, etc.)

## Quick Start

### 1. Build

```bash
cargo build --release
```

The binary will be at `target/release/obsidian-mcp` (or `obsidian-mcp.exe` on Windows).

### 2. Configure

Copy the example env file and add your API key:

```bash
cp config/.env.example .env
# Edit .env and set OBSIDIAN_API_KEY
```

**Never commit your `.env` file to version control.**

### 3. Test Connection

```bash
obsidian-mcp --test-connection
```

You should see:
```
✓ Connected to Obsidian Local REST API successfully
```

If it fails, check that:
- Obsidian is running
- The Local REST API plugin is enabled
- Your API key is correct
- The API URL matches (default: `https://localhost:27124`)

### 4. Start the Server

```bash
obsidian-mcp --host 127.0.0.1 --port 3000
```

The server exposes two endpoints:
- `http://127.0.0.1:3000/mcp` — JSON-RPC 2.0 request/response
- `http://127.0.0.1:3000/sse` — Server-Sent Events stream

## Client Configuration

### Claude Desktop

Add to your `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "obsidian": {
      "command": "obsidian-mcp",
      "args": ["--host", "127.0.0.1", "--port", "3000"],
      "env": {
        "OBSIDIAN_API_KEY": "<your-api-key>",
        "OBSIDIAN_API_URL": "https://localhost:27124"
      }
    }
  }
}
```

See `config/claude-desktop.json` for a template.

### Cursor / Windsurf

Add to your MCP settings (SSE transport):

```json
{
  "mcp": {
    "servers": {
      "obsidian": {
        "url": "http://127.0.0.1:3000/sse"
      }
    }
  }
}
```

See `config/cursor.json` for a template.

### Other SSE Clients

Connect to `http://127.0.0.1:3000/sse` for the event stream and send JSON-RPC requests to `http://127.0.0.1:3000/mcp`.

## CLI Flags

| Flag | Default | Description |
|------|---------|-------------|
| `--test-connection` | — | Test Obsidian API connectivity and exit |
| `--env-file` | `.env` | Path to .env file |
| `--host` | `127.0.0.1` | MCP server bind address |
| `--port` | `3000` | MCP server port |

## Tools (16)

| Group | Tools |
|-------|-------|
| Read/Write | `obsidian_read_note`, `obsidian_read_note_metadata`, `obsidian_write_note`, `obsidian_append_note`, `obsidian_patch_note`, `obsidian_delete_note` |
| Search | `obsidian_search`, `obsidian_dataview_query`, `obsidian_jsonlogic_query` |
| Batch/Convenience | `obsidian_batch_read`, `obsidian_periodic_note`, `obsidian_recent_changes` |
| Navigate | `obsidian_list_directory`, `obsidian_get_tags`, `obsidian_server_status`, `obsidian_open_note` |

## Security Notes

- **Network binding:** The server binds to `127.0.0.1` by default (localhost only). Do NOT bind to `0.0.0.0` unless you understand the implications — the API key is transmitted with every request.
- **Self-signed certs:** The server accepts Obsidian's self-signed HTTPS certificate. This is required because the Local REST API plugin uses a self-signed cert.
- **Path traversal:** All vault paths are validated against directory traversal (`..`, absolute paths, null bytes).
- **Write verification:** Every write operation reads the note back to verify it was saved.
- **Rate limiting:** Each tool is limited to 2 requests/second with a burst of 5.
- **Delete safety:** `obsidian_delete_note` requires `confirm: true`.
- **Dataviewjs risk:** Content written to the vault can contain Obsidian directives like `dataviewjs` blocks that execute JavaScript. The AI client is responsible for not writing unintended executable content.
- **Logging:** API keys and vault note content are never included in log output.

## Vault Structure

The server expects an Obsidian vault with this layout:

```
vault/
  context-manifest.md       # Entry point — AI reads this first
  _index.md                 # Dataview queries for navigation
  content/
    ideas-bank.md           # Capture ideas
    growth-experiments.md   # Track experiments
  research/                 # Research notes
  sessions/                 # One note per session
  troubleshooting/          # Debugging notes
```

See `SKILL.md` for AI session protocols.

## Cross-Compilation

```bash
# Linux (x86_64)
cargo build --release --target x86_64-unknown-linux-gnu

# macOS (Apple Silicon)
cargo build --release --target aarch64-apple-darwin

# Windows
cargo build --release --target x86_64-pc-windows-msvc
```

## Troubleshooting

### "Connection failed"

- Ensure Obsidian is running and the Local REST API plugin is enabled
- Verify your API key in the `.env` file
- Check the API URL matches the plugin's configuration (default: `https://localhost:27124`)
- On first use, you may need to accept the self-signed certificate in a browser

### Search results are stale

The Obsidian full-text search index has a lag. Programmatic writes may not appear in search results immediately. Use `obsidian_dataview_query` instead — Dataview queries are always current.

### "Rate limit exceeded"

Wait the indicated retry duration. The per-tool limit is 2 req/s with burst of 5.

## License

MIT
