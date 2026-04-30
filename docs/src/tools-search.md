# Search Tools

## obsidian_search

Full-text search across the vault.

**Method:** `GET /search/{query}` with `Accept: application/json`

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `query` | string | yes | Search query (max 1000 characters) |

The query is URL-encoded. Returns an array of matching results.

> **Note:** The Obsidian search index has a lag. Programmatic writes may not appear in search results immediately. Use `obsidian_dataview_query` for always-current results.

---

## obsidian_dataview_query

Execute a Dataview DQL query.

**Method:** `POST /search/` with `Content-Type: application/vnd.olrapi.dataview.dql+txt`

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `dql` | string | yes | Dataview DQL query (max 1000 characters) |

Dataview queries are always current — they read directly from the vault index, not the search index.

**Security:** The caller is responsible for query safety. Dataview queries can be resource-intensive but run within Obsidian's Dataview sandbox.

---

## obsidian_jsonlogic_query

Execute a JSONLogic search query.

**Method:** `POST /search/` with `Content-Type: application/vnd.olrapi.jsonlogic+json`

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `logic` | object | yes | JSONLogic query object |

Returns an array of matching results.
