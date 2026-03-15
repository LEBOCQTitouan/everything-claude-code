---
paths:
  - "**/*.sh"
  - "**/*.bash"
  - "**/*.zsh"
---
# Shell Hooks

> This file extends [common/hooks.md](../common/hooks.md) with shell specific content.

## PostToolUse Hooks

Configure in `~/.claude/settings.json`:

- **shfmt**: Auto-format `.sh` files after edit
- **shellcheck**: Run static analysis after editing shell scripts
