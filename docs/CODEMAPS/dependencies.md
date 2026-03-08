<!-- Generated: 2026-03-08 | Files scanned: 48 | Token estimate: ~500 -->

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
| `~/.claude/agents/` | Agent definitions (24 agents) |
| `~/.claude/commands/` | Slash commands (46 commands) |
| `~/.claude/skills/` | Skill directories (69 skills) |
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
agents/      → 24 agent definitions
commands/    → 46 slash commands
skills/      → 69 skill directories
rules/       → Language-specific rules
hooks/       → hooks.json registry
contexts/    → Context injection files
mcp-configs/ → MCP server configs
examples/    → CLAUDE.md templates
scripts/     → Hook shell scripts, codemap generator
install.sh   → Bash orchestrator
index.d.ts   → Type definitions
```
