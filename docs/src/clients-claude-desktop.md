# Claude Desktop & Claude Code

Both Claude Desktop and Claude Code use stdio transport — they launch `obsidian-mcp` as a subprocess and communicate over stdin/stdout.

## Claude Desktop

Edit `claude_desktop_config.json`:

- **macOS:** `~/Library/Application Support/Claude/claude_desktop_config.json`
- **Windows:** `%APPDATA%\Claude\claude_desktop_config.json`

```json
{
  "mcpServers": {
    "obsidian": {
      "command": "obsidian-mcp",
      "args": [],
      "env": {
        "OBSIDIAN_API_KEY": "your-api-key-here",
        "OBSIDIAN_API_URL": "https://localhost:27124"
      }
    }
  }
}
```

Replace `your-api-key-here` with your actual API key. If the binary isn't on PATH, use the full path in `command`:

```json
"command": "/usr/local/bin/obsidian-mcp"
```

A template is available at `config/claude-desktop.json`.

## Claude Code

Same config format. A template is available at `config/claude-code.json`.

## Verifying

1. Restart Claude Desktop after editing the config
2. In a new conversation, ask Claude to read a note from your vault
3. Claude should list `obsidian_*` tools in its available tools

If tools don't appear, check Claude Desktop's developer console (Ctrl+Shift+I) for MCP connection errors.
