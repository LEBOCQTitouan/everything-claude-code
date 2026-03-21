---
description: "Alias for /spec — auto-classifies and delegates. Use /spec directly for the latest pipeline."
allowed-tools: [Read, Grep, Glob, Bash, Skill, AskUserQuestion]
---

# Plan (Alias for /spec)

This command is an alias for `/spec`. It delegates all work to the spec router.

Invoke the Skill tool:

```
skill: "spec", args: "$ARGUMENTS"
```

If the Skill invocation fails, instruct the user:

> `/plan` has been renamed to `/spec`. Please run `/spec` directly.
