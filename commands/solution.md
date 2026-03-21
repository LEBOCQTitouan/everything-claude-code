---
description: "Alias for /design — technical solution design. Use /design directly for the latest pipeline."
allowed-tools: [Bash, Task, Read, Grep, Glob, LS, Write, TodoWrite, TodoRead, EnterPlanMode, ExitPlanMode]
---

# Solution (Alias for /design)

This command is an alias for `/design`. It delegates all work to the design command.

Invoke the Skill tool:

```
skill: "design", args: "$ARGUMENTS"
```

If the Skill invocation fails, instruct the user:

> `/solution` has been renamed to `/design`. Please run `/design` directly.
