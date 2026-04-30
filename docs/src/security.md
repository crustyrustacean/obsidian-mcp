# Security

## Threat Model

The server accepts requests from a trusted MCP client (running on the same machine) and forwards them to the Obsidian Local REST API (also on the same machine). The trust boundary is the local machine.

## Protections

### Path Traversal

All `path` parameters are validated before any HTTP call:

- `..` is rejected (directory traversal)
- Absolute paths (`/foo`, `C:\foo`) are rejected
- Null bytes are rejected
- Paths longer than 512 characters are rejected
- Paths are URL-encoded before interpolation (prevents `%2F..%2F` injection)

### Delete Confirmation

`obsidian_delete_note` requires `confirm: true`. Without it, the tool returns `DeleteConfirmationRequired` instead of deleting. This prevents accidental deletion from a misformatted MCP call.

### Batch Limits

`obsidian_batch_read` enforces a maximum of 20 paths per call. All paths are pre-validated before any HTTP calls are made.

### Rate Limiting

Each tool has a per-tool rate limit of 2 requests/second sustained with a burst of 5. This protects the Obsidian REST API, which runs as a plugin inside the Obsidian app and shares resources with the editor.

### Network Binding

The SSE server binds to `127.0.0.1` by default (localhost only). The MCP endpoint has no authentication — if exposed beyond localhost, anyone can call tools with your API key.

**Do NOT bind to `0.0.0.0` unless you understand the implications.**

### Logging

- API keys are never logged or printed
- Vault note content is never logged
- Only the API URL is logged at startup

### Write Verification

Every write operation (write/append/patch) reads the note back and checks byte count > 2. This guards against silent empty-write failures.

### Error Sanitization

Obsidian API error responses (which may contain filesystem paths or internal details) are truncated to 200 characters and have backslashes normalized before being returned as MCP tool errors.

## Known Risks

### Self-Signed Certificates

The server accepts Obsidian's self-signed HTTPS certificate (`danger_accept_invalid_certs(true)`). This is required because the Local REST API plugin uses a self-signed cert. Future improvement: pin the cert fingerprint or load it from the plugin's data directory.

### DataviewJS Injection

Content written via `write_note`, `append_note`, or `patch_note` can contain `dataviewjs` blocks that execute JavaScript in Obsidian. The AI client is responsible for not writing unintended executable content. This risk is documented in the tool descriptions.

### No Content Size Limits

Write operations accept unbounded content. A malicious or confused AI could write megabytes. Future improvement: add a configurable content size limit.

### TLS Verification Disabled Globally

`danger_accept_invalid_certs(true)` disables TLS verification for all requests made by the HTTP client, not just to the Obsidian API. Since the client only talks to the Obsidian API, this is acceptable in practice.
