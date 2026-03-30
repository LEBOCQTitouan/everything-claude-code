---
name: comms-adapter
description: Pluggable destination adapter patterns for the comms pipeline. Documents file-output (MVP) and interface contracts for future platform integrations.
origin: ECC
---

# Comms Adapter — Destination Patterns

## File Adapter (MVP)

All content is written as markdown files to `comms/drafts/{channel}/YYYY-MM-DD-{slug}.md`.

### Draft File Format

```markdown
---
channel: social
platform: x
generated: 2026-03-30T12:00:00Z
status: draft
---

Your tweet content here. #hashtag
```

### Directory Structure

```
comms/drafts/
  social/
    2026-03-30-v5-release.md
  blog/
    2026-03-30-three-tier-memory.md
  devblog/
    2026-03-30-sqlite-fts5-integration.md
  docs-site/
    2026-03-30-memory-cli-reference.md
```

## Future Adapter Contracts

| Adapter | API | Auth | Status |
|---------|-----|------|--------|
| Buffer | REST `POST /updates/create` | OAuth2 | Planned |
| Typefully | REST `POST /drafts` | API key | Planned |
| Mastodon | REST `POST /api/v1/statuses` | OAuth2 | Planned |
| Bluesky | AT Protocol `com.atproto.repo.createRecord` | App password | Planned |

Each adapter receives a `Draft` with: channel, platform, content, metadata. Returns: draft URL or error.
