# Open Questions & Decisions

## Resolved

1. **Which Rust MCP library?** → Roll minimal JSON-RPC + SSE transport manually. Ecosystem is too young.
2. **Target MCP clients?** → pi (maybe with an extension), but also Claude Desktop / Cursor for broad compatibility.
3. **SSE vs stdio transport?** → SSE transport for broader client compatibility.
4. **Vault location?** → New, dedicated vault (not a subfolder of an existing vault).

## Rust-Specific Considerations

| Topic | Detail |
|-------|--------|
| SSL | Obsidian uses a self-signed cert. Use `reqwest::ClientBuilder::danger_accept_invalid_certs(true)` or load the cert from Obsidian's plugin data directory. |
| Async | Use `tokio` runtime. Batch reads benefit from `futures::future::join_all` for parallel requests. |
| Error handling | Use `thiserror` for typed errors, `anyhow` for top-level propagation. Never panic inside a tool handler. |
| MCP ecosystem | Rust MCP library ecosystem is young. Rolling stdio JSON-RPC manually is ~200 lines. |
| Binary size | Release build with `opt-level = "z"` and `lto = true` should stay under 5MB. |
| Config | Use a `.env` file or environment variable `OBSIDIAN_API_KEY`; load with the `dotenvy` crate. |
