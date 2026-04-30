# Quick Start

## Prerequisites

- [Obsidian](https://obsidian.md) with the [Local REST API](https://github.com/coddingtonbear/obsidian-local-rest-api) plugin installed and enabled
- The plugin's API key (found in Obsidian → Settings → Community plugins → Local REST API)
- An MCP-compatible AI client (pi, Claude Desktop, Cursor, etc.)

## Build

```bash
git clone https://github.com/crustyrustacean/obsidian-mcp.git
cd obsidian-mcp
cargo build --release
```

The binary appears at `target/release/obsidian-mcp` (or `.exe` on Windows).

## Configure

```bash
cp config/.env.example .env
```

Edit `.env` and set your API key:

```ini
OBSIDIAN_API_KEY=your-api-key-here
OBSIDIAN_API_URL=https://localhost:27124
```

**Never commit your `.env` file.** It's in `.gitignore` by default.

## Test Connection

```bash
obsidian-mcp --test-connection
```

You should see:

```
✓ Connected to Obsidian Local REST API successfully
```

If it fails, check that Obsidian is running and the plugin is enabled.

## Run

**Stdio mode** (default — for pi, Claude Desktop, Claude Code):

```bash
obsidian-mcp
```

The server reads JSON-RPC from stdin and writes responses to stdout.

**SSE mode** (for Cursor, Windsurf):

```bash
obsidian-mcp --transport sse --port 3000
```

The server exposes `http://127.0.0.1:3000/mcp` and `http://127.0.0.1:3000/sse`.

## Connect Your Client

See the [Clients](./clients.md) chapter for setup instructions for each supported client.
