# Troubleshooting

## "Connection failed" on --test-connection

- Ensure Obsidian is running
- Ensure the Local REST API plugin is installed and enabled
- Verify your API key in the `.env` file matches the one in Obsidian → Settings → Community plugins → Local REST API
- Check the API URL matches (default: `https://localhost:27124`)
- On first use, you may need to accept the self-signed certificate in a browser (open `https://localhost:27124` and click through the warning)

## Search results are stale

The Obsidian full-text search index has a lag. Programmatic writes may not appear in search results immediately.

**Fix:** Use `obsidian_dataview_query` instead — Dataview queries are always current because they read directly from the vault index.

## "Rate limit exceeded"

Each tool is limited to 2 requests/second sustained with a burst of 5. Wait the indicated `retry_after_seconds` duration before retrying.

If you hit this frequently, the AI client may be making too many calls in a loop. Check the session protocol — the max 5 calls at session start is designed to prevent this.

## "Write verification failed"

This means the note was written but the read-back showed ≤ 2 bytes. Possible causes:

- The Obsidian API accepted the write but didn't persist it (rare)
- The note path is invalid or the vault is read-only
- Obsidian is closing/crashing during the write

Try the write again. If it persists, check the Obsidian console for errors.

## "Delete requires confirm=true"

You called `obsidian_delete_note` without `confirm: true`. This is intentional — it prevents accidental deletion. Re-send the request with `"confirm": true` in the arguments.

## Tools don't appear in Claude Desktop

1. Check `claude_desktop_config.json` is in the right location
2. Ensure the `command` path is correct (use a full path if the binary isn't on PATH)
3. Restart Claude Desktop after editing the config
4. Open the developer console (Ctrl+Shift+I) for MCP connection errors

## pi extension not loading

1. Ensure the extension is in `~/.pi/agent/extensions/obsidian/` (with `index.ts` and `package.json`)
2. Ensure `obsidian-mcp` is on your PATH or set `OBSIDIAN_MCP_BINARY`
3. Run `/reload` in pi to pick up the extension
4. Check stderr output from the binary (visible in pi's notification area for WARN/ERROR)

## SSE connection drops

- The SSE server has a max of 10 concurrent connections. If exceeded, new connections get 429.
- The keep-alive interval prevents idle connection drops.
- If behind a proxy, ensure the proxy doesn't buffer SSE responses.
