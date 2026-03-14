<!-- Generated: 2026-03-15 | Crates: 6 | Cargo deps: 145 -->

# Dependencies & External Integrations

## Workspace Dependencies (Cargo.toml)

| Crate | Version | Purpose |
|-------|---------|---------|
| `serde` | 1 | Serialization/deserialization (derive) |
| `serde_json` | 1 | JSON parsing and generation |
| `thiserror` | 2 | Ergonomic error type derivation |
| `anyhow` | 1 | Application-level error handling |
| `regex` | 1 | Pattern matching and detection rules |
| `clap` | 4 | CLI argument parsing (derive mode) |
| `clap_complete` | 4 | Shell completion generation |
| `walkdir` | 2 | Recursive directory traversal |
| `crossterm` | 0.28 | Cross-platform terminal control |
| `rustyline` | 15 | REPL line editing with history |
| `log` | 0.4 | Logging facade |
| `env_logger` | 0.11 | Log output configuration |

## Dev Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `proptest` | 1 | Property-based testing (ecc-domain) |

## Per-Crate Dependency Map

```
ecc-domain    → serde, serde_json, regex
ecc-ports     → thiserror, serde_json
ecc-app       → ecc-domain, ecc-ports, anyhow, serde, serde_json, regex, log
ecc-infra     → ecc-ports, serde_json, walkdir, crossterm, rustyline, anyhow
ecc-cli       → ecc-domain, ecc-ports, ecc-app, ecc-infra, clap, clap_complete, anyhow, serde_json, log, env_logger
ecc-test-support → ecc-ports
```

## External Tool Integrations

| Tool | Usage | Required |
|------|-------|----------|
| `git` | Repo detection, file tracking, gitignore management | Optional |
| `claude` CLI | Smart merge via `claude -p` | Optional |

## File System Targets

| Path | Purpose |
|------|---------|
| `~/.claude/agents/` | Agent definitions (30 agents) |
| `~/.claude/commands/` | Slash commands (7 commands) |
| `~/.claude/skills/` | Skill directories (81 skills) |
| `~/.claude/rules/` | Rules by language group (7 groups) |
| `~/.claude/settings.json` | Hooks + deny rules configuration |
| `~/.claude/.ecc-manifest.json` | ECC manifest tracking |
| `~/.claude/claw/sessions/` | NanoClaw REPL session data |
| `~/.claude/claw/session-aliases.json` | Session aliases |
| `.gitignore` | ECC-managed entries (project-level) |
| `CLAUDE.md` | Project instructions (project-level) |

## npm Package Distribution

Published as `@lebocqtitouan/ecc`. Uses platform-specific optional dependencies:

```
npm/
  ├─ ecc-darwin-arm64/    → macOS ARM binary
  ├─ ecc-darwin-x64/      → macOS Intel binary
  ├─ ecc-linux-arm64/     → Linux ARM binary
  ├─ ecc-linux-x64/       → Linux x64 binary
  └─ ecc-win32-x64/       → Windows x64 binary

Included content:
  agents/      → 30 agent definitions
  commands/    → 7 slash commands
  skills/      → 81 skill directories
  rules/       → Language-specific rules (7 groups)
  hooks/       → hooks.json registry
  contexts/    → Context injection files
  mcp-configs/ → MCP server configs
  examples/    → CLAUDE.md templates
  schemas/     → JSON schemas
```

## Total Dependency Count

- **Direct workspace deps:** 12 crates
- **Total resolved (Cargo.lock):** 145 crates (includes transitive)
