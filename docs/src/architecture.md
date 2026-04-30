# Architecture

obsidian-mcp is a thin MCP protocol layer over the Obsidian Local REST API. The architecture separates concerns cleanly:

```
┌─────────────────────────────────────────────────────┐
│                   MCP Client                        │
│          (pi, Claude Desktop, Cursor, etc.)         │
└──────────┬──────────────────────┬───────────────────┘
           │                      │
     stdio transport         SSE transport
           │                      │
┌──────────▼──────────┐ ┌───────▼──────────────┐
│   src/stdio.rs      │ │    src/server.rs     │
│   NDJSON framing    │ │    Axum + SSE        │
└──────────┬──────────┘ └───────┬──────────────┘
           │                      │
           └──────┬───────────────┘
                  │
         ┌────────▼────────┐
         │  dispatch_request()  │  ← shared dispatch
         │  src/server.rs       │
         └────────┬────────┘
                  │
         ┌────────▼────────┐
         │   ToolRegistry   │  ← 16 registered tools
         │   src/tools.rs   │
         └────────┬────────┘
                  │
         ┌────────▼────────┐
         │  ObsidianClient  │  ← HTTP calls to Obsidian
         │  src/client.rs   │
         └────────┬────────┘
                  │
         ┌────────▼────────┐
         │  Obsidian Local  │
         │  REST API Plugin │
         └─────────────────┘
```

Key design principles:

1. **Transport-agnostic dispatch** — `dispatch_request()` is shared by stdio and SSE. Adding a new transport means writing a framing layer, not re-implementing logic.
2. **Per-tool headers** — each Obsidian API call sets its own `Accept`/`Content-Type` explicitly. No shared header set.
3. **Write verification** — every write operation reads the note back and checks byte count > 2.
4. **No panics in tool handlers** — all errors flow through `Result` types and surface as MCP tool errors with `isError: true`.
5. **Rate limiting** — per-tool GCRA rate limiter protects the Obsidian REST API (which shares resources with the editor).
