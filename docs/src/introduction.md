# Introduction

**obsidian-mcp** is a Rust-native MCP (Model Context Protocol) server that bridges an Obsidian vault to any MCP-compatible AI client. It ships as a single statically-compiled binary with two transport modes:

- **stdio** (default) — for Claude Desktop, Claude Code, and [pi](https://pi.dev)
- **SSE** — for Cursor, Windsurf, and other HTTP-based clients

It talks to Obsidian via the [Local REST API](https://github.com/coddingtonbear/obsidian-local-rest-api) plugin, exposing 16 tools for reading, writing, searching, and navigating your vault.

## Why Rust?

This project replaces a Python/FastMCP implementation. The Rust version is:

- **Fast** — sub-millisecond tool dispatch, no GC pauses
- **Small** — 3.8 MB release binary (vs. 50+ MB Python + dependencies)
- **Reliable** — no silent empty-write failures (every write is verified), structured error handling, no panics in tool handlers
- **Self-contained** — statically compiled, no runtime dependencies

## What It's Not

This is not a general-purpose Obsidian plugin. It's an MCP server — it exposes vault operations to AI clients that speak the MCP protocol. The AI decides *what* to read and write; the server provides safe, validated access.

## License

MIT
