---
tags:
  - type/index
created: 2026-04-30
updated: 2026-04-30
type: index
status: active
confidence: high
---

# Vault Index

Dataview live queries for quick navigation.

## Recent Session Logs

```dataview
LIST FROM "sessions"
SORT file.mtime DESC
LIMIT 10
```

## Active Ideas

```dataview
TABLE status, confidence
FROM "content"
WHERE type = "idea"
SORT updated DESC
```

## Open Troubleshooting

```dataview
TABLE status, confidence
FROM "troubleshooting"
WHERE status = "active"
SORT updated DESC
```
