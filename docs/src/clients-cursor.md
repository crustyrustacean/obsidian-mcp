# Cursor / Windsurf

Cursor and Windsurf use SSE transport — they connect to a running HTTP server rather than spawning a subprocess.

## Setup

1. Start the MCP server in SSE mode:

   ```bash
   obsidian-mcp --transport sse --port 3000
   ```

2. In Cursor, open Settings → MCP and add:

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

A template is available at `config/cursor.json`.

## Keeping It Running

The SSE server must stay running while you use Cursor. Options:

- **Systemd** (Linux): Create a user service that starts on login
- **LaunchAgent** (macOS): Create a plist in `~/Library/LaunchAgents/`
- **Startup script**: Add `obsidian-mcp --transport sse &` to your shell profile
- **tmux**: Run in a tmux session

## Security Note

The SSE server binds to `127.0.0.1` by default (localhost only). Do NOT bind to `0.0.0.0` — the API key is transmitted with every request and there is no authentication on the MCP endpoint itself.
