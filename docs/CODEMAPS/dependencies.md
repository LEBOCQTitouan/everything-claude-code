<!-- Generated: 2026-03-14 | Files scanned: 50 | Token estimate: ~500 -->

# Dependencies & External Integrations

## Runtime Dependencies

| Package | Version | Purpose |
|---------|---------|---------|
| `omelette` | ^0.4.17 | Shell tab-completion for `ecc` CLI |

## Dev Dependencies

| Package | Version | Purpose |
|---------|---------|---------|
| `typescript` | ^5.9.3 | TypeScript compiler |
| `@types/node` | ^25.3.5 | Node.js type definitions |
| `tsx` | ^4.21.0 | TypeScript execution for tests |
| `markdownlint-cli` | ^0.48.0 | Markdown linting |

## External Tool Integrations

| Tool | Usage | Required |
|------|-------|----------|
| `bash` | `install.sh` orchestration | Yes |
| `node` (>=18) | Runtime, hook execution | Yes |
| `git` | Repo detection, file tracking, session management | Optional |
| `claude` CLI | Smart merge via `claude -p` | Optional |
| `npx` | Test runner (`npx tsx`) | Dev only |

## File System Targets

| Path | Purpose |
|------|---------|
| `~/.claude/agents/` | Agent definitions (30 agents) |
| `~/.claude/commands/` | Slash commands (6 active + 41 archived) |
| `~/.claude/skills/` | Skill directories (67 skills) |
| `~/.claude/rules/` | Rules by language group |
| `~/.claude/settings.json` | Hooks configuration |
| `~/.claude/.ecc-manifest.json` | ECC manifest tracking |
| `~/.claude/sessions/` | Session data |
| `~/.claude/session-aliases.json` | Session aliases |
| `.gitignore` | ECC-managed entries (project-level) |
| `CLAUDE.md` | Project instructions (project-level) |

## npm Package Distribution

Published as `@lebocqtitouan/ecc`. Included files:
```
bin/         → CLI entry point
dist/        → Compiled JS
agents/      → 30 agent definitions
commands/    → 6 active + 41 archived slash commands
skills/      → 67 skill directories
rules/       → Language-specific rules
hooks/       → hooks.json registry
contexts/    → Context injection files
mcp-configs/ → MCP server configs
examples/    → CLAUDE.md templates
scripts/     → Hook shell scripts, codemap generator
install.sh   → Bash orchestrator
index.d.ts   → Type definitions
```
