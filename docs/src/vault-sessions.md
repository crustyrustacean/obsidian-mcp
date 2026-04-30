# Session Protocols

The vault is designed for AI-driven sessions with a defined start and end.

## Session Start

**Hard rule: Maximum 5 MCP calls at session start.** This keeps context budgets lean.

1. **Always** read `context-manifest.md` first (1 call)
2. Based on the task type, load additional files:

| Task Type | Keywords | Additional Files | Total Calls |
|-----------|----------|-----------------|-------------|
| Coding / Debugging | debug, error, bug, fix, compile | `troubleshooting/` listing | 2 |
| Planning | plan, design, architect, roadmap | `content/ideas-bank.md`, `content/growth-experiments.md` | 3 |
| General | (everything else) | manifest only | 1 |

Task type is determined by keyword matching in the user's initial prompt.

### Do NOT

- Read the entire vault at session start
- Make more than 5 MCP calls during initialization
- Modify `SKILL.md` programmatically

## Session End

When a session concludes:

1. **Write a session log** to `sessions/YYYY-MM-DD-HHMM.md` with session-log frontmatter
2. **Update `context-manifest.md`** with:
   - A reference to the new session log
   - An updated `updated:` date in frontmatter
3. Optionally grep known facts across files for drift (for long sessions)

## Session-End Audit

The `audit::audit_session_end()` function checks:

1. **Session log present** — the specified session log file exists
2. **Manifest valid** — `context-manifest.md` has valid frontmatter of type `manifest`
3. **Manifest fresh** — the `updated` date matches today

All three checks must pass for `all_passed()` to return `true`.

## Session Log Format

```markdown
---
tags:
  - project/<name>
  - type/session-log
  - topic/<coding|planning|general>
created: 2025-06-15
updated: 2025-06-15
type: session-log
status: active
confidence: medium
---

# Session Log — 2025-06-15

**Task type:** Coding / Debugging
**Project:** my-rust-project

## Summary

<what happened>

## Notes

<details>

## Outcomes

<results>
```
