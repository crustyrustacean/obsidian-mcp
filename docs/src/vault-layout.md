# Directory Layout

```
vault/
├── context-manifest.md       # Entry point — AI reads this first
├── _index.md                 # Dataview queries for navigation
├── content/
│   ├── ideas-bank.md         # Capture ideas
│   └── growth-experiments.md # Track experiments
├── research/                 # Research notes
├── sessions/                 # One note per session
└── troubleshooting/          # Debugging notes
```

## Required Directories

| Directory | Purpose |
|-----------|---------|
| `content/` | Substantive notes — ideas, experiments, analysis |
| `research/` | Research notes and references |
| `sessions/` | Auto-generated session logs |
| `troubleshooting/` | Debug notes, error histories |

## Required Files

| File | Purpose |
|------|---------|
| `context-manifest.md` | AI entry point — active sessions, current focus, key decisions |
| `_index.md` | Dataview live queries for quick navigation by type, status, recency |

## Initializing a Vault

The `vault::init_vault()` function creates the directory structure and seed files. It's idempotent — running it twice won't overwrite existing files.

The seed files contain valid frontmatter and placeholder content. The `context-manifest.md` is the most important — it's the first thing an AI reads at session start.

## Validation

The `vault::validate_vault()` function checks that all required directories and root files exist. Returns a `VaultValidation` struct with lists of missing items.
