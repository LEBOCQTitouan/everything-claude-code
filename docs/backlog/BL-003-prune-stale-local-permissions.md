---
id: BL-003
title: Prune stale local permissions
tier: 1
scope: LOW
target: direct edit
status: implemented
created: 2026-03-20
file: .claude/settings.local.json
---

## Action

Remove these accumulated debugging/stale entries from the allows list: `Bash(done)`, `Bash(__NEW_LINE_eb74b2b7edcc8afa__...)`, `Bash(echo "exit: $?")`, `Bash(bash -n .claude/hooks/spec-schema-check.sh)` (file doesn't exist), `Bash(bash -n .claude/hooks/solution-coverage-check.sh)` (file doesn't exist), `Bash(for f:*)`, `Bash(do echo:*)`, `Bash(sed 's/\\.md$//')`, `Bash(sed 's/\`//g')`. Keep all meaningful git/cargo/npm/find allows. After pruning, review what remains — if any entry references a path or script that no longer exists, remove it too.
