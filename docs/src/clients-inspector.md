# MCP Inspector

The [MCP Inspector](https://github.com/modelcontextprotocol/inspector) is a debugging tool for testing MCP servers.

## Running

```bash
npx @modelcontextprotocol/inspector obsidian-mcp
```

The inspector launches `obsidian-mcp` in stdio mode and provides a web UI for sending requests and inspecting responses.

## What to Test

1. **Initialize** — click "Connect" to send the `initialize` handshake
2. **Tools list** — verify all 16 tools appear with correct schemas
3. **Tool calls** — test each tool against a running Obsidian instance
4. **Error handling** — try invalid paths, missing params, unreachable API

## Environment Variables

Set these before launching the inspector:

```bash
export OBSIDIAN_API_KEY=your-api-key-here
export OBSIDIAN_API_URL=https://localhost:27124
npx @modelcontextprotocol/inspector obsidian-mcp
```
