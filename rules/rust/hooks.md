---
paths:
  - "**/*.rs"
  - "**/Cargo.toml"
applies-to: { languages: [rust] }
---
# Rust Hooks

> Extends [common/hooks.md](../common/hooks.md) with Rust-specific automations.

## Recommended PostToolUse Hooks

Configure in `~/.claude/settings.json` or `.claude/settings.json`:

- **rustfmt**: Auto-format `.rs` files after every edit
- **clippy**: Run `cargo clippy -- -D warnings` after editing source files
- **cargo check**: Fast type-check without full compilation after edits

## Example Configuration

```json
{
  "hooks": {
    "PostToolUse": [
      {
        "matcher": "Edit|Write",
        "hooks": [{
          "type": "command",
          "command": "if echo '$CLAUDE_TOOL_INPUT' | grep -q '\\.rs$'; then cargo fmt -- \"$CLAUDE_TOOL_INPUT\" 2>/dev/null; fi"
        }]
      }
    ]
  }
}
```

## Recommended Workflow

1. Edit `.rs` file
2. `cargo fmt` — format
3. `cargo clippy -- -D warnings` — lint
4. `cargo test` — verify nothing broke
5. `cargo audit` — check for known vulnerabilities (before commit)

## Useful Aliases

Add to your shell profile for faster iteration:

```bash
alias cc='cargo clippy -- -D warnings'
alias ct='cargo test'
alias cf='cargo fmt'
alias cb='cargo build'
```
