# Everything Claude Code

> Production-ready agents, skills, hooks, commands, and rules for Claude Code — with a Rust CLI (`ecc`) that installs and manages them.
>
> Forked from [affaan-m/everything-claude-code](https://github.com/affaan-m/everything-claude-code) by [@affaanmustafa](https://x.com/affaanmustafa) — Anthropic Hackathon Winner.

## What Is This?

A curated collection of subagents, slash commands, skills, hooks, and rules for software development with [Claude Code](https://claude.ai/code). This fork adds an opinionated layer enforcing Hexagonal Architecture, DDD, and Clean Code via specialized architect and reviewer agents, plus a Rust CLI to install everything in one command.

## Quick Start

```bash
# Install the CLI
curl -fsSL https://raw.githubusercontent.com/LEBOCQTitouan/everything-claude-code/main/scripts/get-ecc.sh | bash

# Global setup — installs agents, commands, skills, rules, hooks to ~/.claude/
ecc install typescript          # common + TypeScript rules
ecc install typescript python   # multiple stacks

# Per-project setup
cd /your/project && ecc init    # auto-detects language, creates CLAUDE.md + hooks

# Verify everything works
ecc audit
```

## Shell Completions

```bash
eval "$(ecc completion zsh)"   # add to ~/.zshrc — see docs/getting-started.md for bash/fish
```

## Documentation

| Doc | Purpose |
|-----|---------|
| [Getting Started](docs/getting-started.md) | Extended setup, usage, MCP config, repo structure |
| [Architecture](docs/ARCHITECTURE.md) | System design and hexagonal layout |
| [CLAUDE.md](CLAUDE.md) | Claude Code project instructions |
| [Contributing](CONTRIBUTING.md) | How to contribute |

## Credits

Original project: **[affaan-m/everything-claude-code](https://github.com/affaan-m/everything-claude-code)** by [@affaanmustafa](https://x.com/affaanmustafa).
Built from an Anthropic Hackathon winner. Guides: [Shorthand](https://x.com/affaanmustafa/status/2012378465664745795) | [Longform](https://x.com/affaanmustafa/status/2014040193557471352).

## License

MIT
