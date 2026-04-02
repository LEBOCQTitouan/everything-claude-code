---
id: BL-002
title: Pin all MCP package versions
tier: 1
scope: LOW
target: direct edit
status: "implemented"
created: 2026-03-20
file: mcp-configs/mcp-servers.json
---

## Action

For each of the 8 unpinned `npx -y` entries (`@modelcontextprotocol/server-github`, `firecrawl-mcp`, `@modelcontextprotocol/server-memory`, `@modelcontextprotocol/server-sequential-thinking`, `@railway/mcp-server`, `exa-mcp-server`, `@context7/mcp-server`, `@modelcontextprotocol/server-filesystem`), look up the current latest version on npm and pin it explicitly (e.g., `@modelcontextprotocol/server-github@2.2.0`). Add a `_pinned_date` field in the `_comments` block with today's date and a quarterly audit reminder. `@magicuidesign/mcp@latest` and `@supabase/mcp-server-supabase@latest` also need pinning — `@latest` is not a pin.
