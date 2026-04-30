# Vault

The server expects an Obsidian vault with a specific structure and frontmatter schema. This chapter covers the layout, schema, and session protocols.

The vault design supports an AI "second brain" workflow:

1. **Session start** — the AI reads `context-manifest.md` (1 call), then loads task-specific files (≤ 5 total calls)
2. **During session** — the AI reads, writes, and searches the vault using the 16 tools
3. **Session end** — the AI writes a session log and updates the manifest

The vault is a dedicated, standalone directory — not a subfolder of an existing vault. This isolation prevents accidental access to personal notes.
