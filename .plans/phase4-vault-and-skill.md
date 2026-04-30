# Phase 4 — Vault Structure & Skill File

**Objective:** Set up the Obsidian vault schema and document how to use the server for an AI client.

**Duration:** Week 5

**TDD Discipline:** Write failing tests before any implementation. Even vault structure and frontmatter enforcement are testable — validate schemas and protocols before building them out.

## Tests (write first)

### Vault Structure

- [x] **Test: vault directory layout is valid** — assert that the expected directories (`content/`, `research/`, `sessions/`, `troubleshooting/`) and root files (`context-manifest.md`, `_index.md`) exist when the vault is initialized.
- [x] **Test: vault initializer creates missing directories** — start with an empty directory; run the vault init function; assert all required folders and seed files are created.
- [x] **Test: vault initializer is idempotent** — run init twice on the same directory; assert no data is overwritten and no errors are raised.

### Frontmatter Schema

- [x] **Test: valid frontmatter parses correctly** — provide a complete YAML frontmatter block; assert it deserializes into the schema struct with all fields populated.
- [x] **Test: missing required fields produce errors** — omit `tags`, `created`, or `type`; assert a validation error is returned with a clear message.
- [x] **Test: invalid `type` enum value is rejected** — provide `type: foobar`; assert the enum parser rejects it.
- [x] **Test: `updated` timestamp is auto-set on write** — write a note; assert the `updated` field matches the current date (within tolerance).

### Skill File / Session Protocols

- [x] **Test: session start respects max 5 MCP calls** — simulate a session start; count the MCP calls made; assert the count is ≤ 5.
- [x] **Test: routing table dispatches correctly** — for each task type (coding/debugging, planning, general), assert the correct set of files is loaded.
- [x] **Test: session end writes a log** — simulate a session end; assert a new file appears in `sessions/` with valid frontmatter.
- [x] **Test: session end updates manifest** — simulate a session end; assert `context-manifest.md` is updated with the latest session reference and `updated` timestamp.

## Implementation (make tests pass)

### Vault Structure

- [x] Create dedicated vault with this layout:
  ```
  vault/
    context-manifest.md       # entry point; AI reads this first
    _index.md                 # Dataview live queries by type/status/recency
    content/
      ideas-bank.md
      growth-experiments.md
    research/
    sessions/                 # one note per session, auto-written at session end
    troubleshooting/
  ```

### Frontmatter Schema

- [x] Define and enforce frontmatter for every note:
  ```yaml
  tags:
    - project/<name>
    - type/<type>
    - topic/<topic>
  created: YYYY-MM-DD
  updated: YYYY-MM-DD
  type: session-log | idea | research | troubleshooting | manifest | index
  status: active | completed | abandoned
  confidence: high | medium | low
  ```

### Skill / README File

- [x] Document session start protocol: read `context-manifest.md` (1 call), then load 1–4 task-specific files based on task type
- [x] Define routing table:
  - coding/debugging → load `troubleshooting/`
  - planning → load `content/`
  - general → manifest only
- [x] Document session end protocol: write session log, update manifest, grep known facts across files for drift
- [x] Hard rule: **max 5 MCP calls at session start**

## Security Review

- [x] **Vault isolation:** The vault must be a dedicated, standalone directory. Verify that no tool can read or write outside the vault root. Path traversal checks from Phase 3 must also prevent vault escapes.
- [x] **Frontmatter injection:** YAML frontmatter is parsed — ensure no malicious YAML (e.g., deserialization attacks) can be injected via note content. Use a safe YAML parser that does not support anchors/aliases or custom tags. (Using serde_yaml which is safe for typed deserialization.)
- [ ] **Session log sensitivity:** Session logs may contain sensitive information (API keys, credentials discussed during a session). Document the risk and consider a `sensitive: true` frontmatter field that triggers content redaction in search results.
- [x] **Manifest integrity:** `context-manifest.md` is the AI's entry point. If corrupted or maliciously modified, it could misdirect the AI. Validate its structure on every read; reject malformed manifests gracefully.
- [x] **Skill file as trust boundary:** The skill/README file instructs the AI on how to behave. Treat it as a trust boundary — only the user should edit it. Document that the server does not auto-modify the skill file.
- [x] **Max MCP call enforcement:** The "max 5 calls at session start" rule is enforced in code (MAX_SESSION_START_CALLS constant and routing table call counts verified by tests).

## Deliverable

Populated vault with context manifest, skill file committed to repo, all structure/schema/protocol tests passing.
