# Read / Write Tools

## obsidian_read_note

Read a note from the vault as markdown.

**Method:** `GET /vault/{path}` with `Accept: text/markdown`

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `path` | string | yes | Path within the vault (e.g., `notes/my-note.md`) |

Returns the note content as a text string.

---

## obsidian_read_note_metadata

Read a note's frontmatter as JSON.

**Method:** `GET /vault/{path}` with `Accept: application/json`

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `path` | string | yes | Path within the vault |

Returns the parsed frontmatter as a JSON object.

---

## obsidian_write_note

Write (create or replace) a note. The write is verified by reading the note back.

**Method:** `PUT /vault/{path}` with `Content-Type: text/markdown`

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `path` | string | yes | Path within the vault |
| `content` | string | yes | Markdown content to write |

Returns a success message on verification. If the read-back shows ≤ 2 bytes, returns `WriteVerificationFailed`.

---

## obsidian_append_note

Append content to an existing note. The write is verified.

**Method:** `POST /vault/{path}` with `Content-Type: text/markdown`

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `path` | string | yes | Path within the vault |
| `content` | string | yes | Markdown content to append |

---

## obsidian_patch_note

Update a specific heading within a note. The write is verified.

**Method:** `PATCH /vault/{path}` with `Content-Type: text/markdown` and `Heading` header

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `path` | string | yes | Path within the vault |
| `heading` | string | yes | Heading to target (e.g., `## Section`) |
| `content` | string | yes | Content to place under the heading |

---

## obsidian_delete_note

Delete a note from the vault. Requires explicit confirmation.

**Method:** `DELETE /vault/{path}`

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `path` | string | yes | Path within the vault |
| `confirm` | boolean | yes | Must be `true` to confirm deletion |

If `confirm` is not `true`, returns `DeleteConfirmationRequired` error. This prevents accidental deletion from a misformatted MCP call.
