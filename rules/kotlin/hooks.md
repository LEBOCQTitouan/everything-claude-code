---
paths:
  - "**/*.kt"
  - "**/*.kts"
  - "**/build.gradle.kts"
  - "**/settings.gradle.kts"
---
# Kotlin Hooks

> This file extends [common/hooks.md](../common/hooks.md) with Kotlin specific content.

## PostToolUse Hooks

Configure in `~/.claude/settings.json`:

- **ktlint**: Auto-format `.kt` files after edit
- **detekt**: Run static analysis after editing Kotlin files
- **gradle check**: Verify build and tests on modified files
