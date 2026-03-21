---
id: BL-001
title: Block auto-enable of MCP servers
tier: 1
scope: LOW
target: direct edit
status: open
created: 2026-03-20
file: ~/.claude/settings.json
---

## Action

Add `"enableAllProjectMcpServers": false` at top level. This prevents any cloned repo's `.mcp.json` from silently activating MCP servers. Single highest-impact security fix — without it, a malicious repo can exfiltrate your filesystem via a crafted MCP server the moment you open it in Claude Code.
