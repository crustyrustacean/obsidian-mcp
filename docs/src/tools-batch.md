# Batch & Convenience Tools

## obsidian_batch_read

Read multiple notes in parallel.

**Method:** Parallel `GET /vault/{path}` calls

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `paths` | string[] | yes | Array of note paths (max 20) |

Returns a map of path → content. Failed reads are reported per-path with an `[ERROR]` prefix rather than failing the entire operation. All paths are pre-validated before any HTTP calls are made.

---

## obsidian_periodic_note

Get a periodic note from the Periodic Notes plugin.

**Method:** `GET /periodic/{period}` with `Accept: text/markdown`

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `period` | string | yes | One of `daily`, `weekly`, `monthly` |

Invalid period values return an error.

---

## obsidian_recent_changes

Get recently changed notes, sorted by modification time (most recent first).

**Method:** `GET /vault/` (root directory listing, recursive walk)

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `limit` | integer | no | Max results (1–100, default 10) |

The tool recursively walks the vault directory tree, collects file entries with metadata (including mtime), sorts by most recent modification time, and returns the top N entries. Uses full paths (not relative), fixing a silent bug from the Python predecessor.
