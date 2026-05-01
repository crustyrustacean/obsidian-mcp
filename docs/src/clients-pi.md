# pi

[pi](https://pi.dev) doesn't speak MCP natively — it uses TypeScript extensions with `pi.registerTool()`. The project includes a ready-made extension at `extension/`.

## Setup

1. Build the binary and put it on your PATH:

   ```bash
   cargo build --release
   cp target/release/obsidian-mcp /usr/local/bin/  # or equivalent
   ```

2. Copy the extension to your pi extensions directory:

   ```bash
   cp -r extension ~/.pi/agent/extensions/obsidian
   ```

3. Set your API key in your environment (e.g., in `~/.profile` or `~/.zshrc`):

   ```bash
   export OBSIDIAN_API_KEY=your-api-key-here
   ```

4. Start pi. The extension auto-loads and registers all 16 tools.

## How It Works

The extension spawns `obsidian-mcp --transport stdio` as a background process:

1. On `session_start`, the extension spawns the binary and performs the MCP initialize handshake
2. Each `pi.registerTool()` call maps to a `tools/call` JSON-RPC request over stdin/stdout
3. On `session_shutdown`, the extension kills the subprocess

## Configuration

| Environment Variable | Default | Description |
|---------------------|---------|-------------|
| `OBSIDIAN_MCP_BINARY` | `obsidian-mcp` | Path to the binary (if not on PATH) |
| `OBSIDIAN_API_KEY` | *(required)* | Obsidian Local REST API key |
| `OBSIDIAN_API_URL` | `https://localhost:27124` | Obsidian API base URL |

## Verifying

Start pi and type:

```
/obsidian_server_status
```

If the extension is working, the model will call `obsidian_server_status` and report whether the Obsidian API is reachable.
