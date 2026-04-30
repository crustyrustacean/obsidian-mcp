# Clients

obsidian-mcp works with any MCP-compatible client. The setup differs by client and transport mode.

## Transport Summary

| Client | Transport | Setup |
|--------|-----------|-------|
| [pi](./clients-pi.md) | stdio | TypeScript extension (included) |
| [Claude Desktop](./clients-claude-desktop.md) | stdio | JSON config file |
| [Claude Code](./clients-claude-desktop.md) | stdio | JSON config file |
| [Cursor / Windsurf](./clients-cursor.md) | SSE | URL in MCP settings |
| [MCP Inspector](./clients-inspector.md) | stdio | CLI command |

## Before You Begin

All clients need:

1. **The binary** — `cargo build --release` and put it on your PATH, or specify the full path
2. **The API key** — from Obsidian → Settings → Community plugins → Local REST API
3. **Obsidian running** — the Local REST API plugin requires Obsidian to be open
