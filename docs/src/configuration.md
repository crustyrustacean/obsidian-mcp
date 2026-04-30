# Configuration

## Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `OBSIDIAN_API_KEY` | yes | — | API key from the Obsidian Local REST API plugin |
| `OBSIDIAN_API_URL` | no | `https://localhost:27124` | Base URL for the Obsidian Local REST API |

## .env File

The server reads `.env` files directly (does not pollute the process environment). Create one from the template:

```bash
cp config/.env.example .env
```

```ini
# Obsidian MCP Server — .env file
# NEVER commit this file to version control

OBSIDIAN_API_KEY=your-api-key-here
OBSIDIAN_API_URL=https://localhost:27124
```

### Loading Order

1. Variables from the `.env` file are read first
2. Process environment variables override `.env` values
3. The `--env-file` flag specifies an alternative path (default: `.env` in the current directory)

### Why Not `dotenvy`?

The server reads `.env` files directly instead of using `dotenvy`. The `dotenvy` crate sets process environment variables globally, which causes test interference. Direct parsing is also more secure — no global state pollution.

## API URL

The default `https://localhost:27124` matches the Local REST API plugin's default configuration. If you've changed the plugin's port, update `OBSIDIAN_API_URL` accordingly.

The plugin uses HTTPS with a self-signed certificate. The server accepts this certificate automatically.
