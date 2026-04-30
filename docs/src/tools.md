# Tools

The server exposes 16 tools organized into four groups:

| Group | Count | Tools |
|-------|-------|-------|
| Read / Write | 6 | Read, read metadata, write, append, patch, delete |
| Search | 3 | Full-text search, Dataview DQL, JSONLogic |
| Batch & Convenience | 3 | Batch read, periodic note, recent changes |
| Navigate | 4 | List directory, get tags, server status, open note |

## Common Behaviors

### Path Validation

All tools that accept a `path` parameter validate it before making any API call:

- Rejects `..` (directory traversal)
- Rejects absolute paths (`/foo` or `C:\foo`)
- Rejects null bytes
- Rejects paths longer than 512 characters
- Paths are URL-encoded before interpolation into request URLs

### Write Verification

`write_note`, `append_note`, and `patch_note` all verify the write by reading the note back immediately and checking that byte count > 2. This guards against silent empty-write failures (a bug that lost an entire session in the Python predecessor).

### Rate Limiting

Every tool call passes through the per-tool rate limiter (2 req/s sustained, burst of 5). If the limit is exceeded, the tool returns a `RateLimitExceeded` error with a `retry_after_seconds` hint.

### DataviewJS Risk

`write_note`, `append_note`, and `patch_note` write content as-is. If the content contains `dataviewjs` blocks, they will execute JavaScript in Obsidian. The AI client is responsible for not writing unintended executable content. This risk is documented in the tool descriptions.
