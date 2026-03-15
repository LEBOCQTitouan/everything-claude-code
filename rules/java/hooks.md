---
paths:
  - "**/*.java"
  - "**/pom.xml"
  - "**/build.gradle"
  - "**/build.gradle.kts"
---
# Java Hooks

> This file extends [common/hooks.md](../common/hooks.md) with Java specific content.

## PostToolUse Hooks

Configure in `~/.claude/settings.json`:

- **google-java-format/spotless**: Auto-format `.java` files after edit
- **spotbugs**: Run static analysis after editing Java files
- **checkstyle**: Verify code style compliance on modified files
