---
paths:
  - "**/*.c"
  - "**/*.cpp"
  - "**/*.h"
  - "**/*.hpp"
  - "**/*.cc"
  - "**/*.cxx"
  - "**/CMakeLists.txt"
applies-to: { languages: [cpp] }
---
# C/C++ Hooks

> This file extends [common/hooks.md](../common/hooks.md) with C/C++ specific content.

## PostToolUse Hooks

Configure in `~/.claude/settings.json`:

- **clang-format**: Auto-format `.cpp`/`.h` files after edit
- **clang-tidy**: Run static analysis after editing C/C++ files
- **cppcheck**: Run bug detection on modified files
