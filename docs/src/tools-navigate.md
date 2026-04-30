# Navigate Tools

## obsidian_list_directory

List the contents of a directory in the vault.

**Method:** `GET /vault/{path}/` with `Accept: application/json`

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `path` | string | no | Directory path (empty string or omit for root) |

Returns a structured list of entries with names and types (`file` or `folder`).

---

## obsidian_get_tags

Get the tag hierarchy with counts.

**Method:** `GET /vault-tags/` with `Accept: application/json`

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| *(none)* | | | |

Returns a JSON object of tags and their frequencies.

---

## obsidian_server_status

Health check — test connectivity to the Obsidian Local REST API.

**Method:** `GET /` with `Authorization` header

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| *(none)* | | | |

Returns a success message if the API is reachable and the API key is valid. Returns a structured error otherwise.

Also available via `--test-connection` CLI flag (runs the check and exits).

---

## obsidian_open_note

Trigger the Obsidian UI to open a note.

**Method:** `POST /open/{path}`

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `path` | string | yes | Path to the note to open |

This tells the Obsidian app to navigate to the note in its UI. Useful for directing the user's attention to a specific file.
