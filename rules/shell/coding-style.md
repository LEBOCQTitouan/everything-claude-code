---
paths:
  - "**/*.sh"
  - "**/*.bash"
  - "**/*.zsh"
applies-to: { files: ["*.sh"] }
---
# Shell Coding Style

> This file extends [common/coding-style.md](../common/coding-style.md) with shell specific content.

## Formatting

- **shfmt** is mandatory for consistent formatting
- **shellcheck** is mandatory for static analysis:
  ```bash
  shellcheck -x script.sh
  ```

## Safety Flags

Always start scripts with strict mode:

```bash
#!/usr/bin/env bash
set -euo pipefail
```

- `set -e`: Exit on error
- `set -u`: Error on undefined variables
- `set -o pipefail`: Pipe failure propagation

## Naming

- Variables: `snake_case` or `SCREAMING_SNAKE_CASE` for exports
- Functions: `snake_case`
- Scripts: `kebab-case.sh`

## Quoting

- Always double-quote variables: `"${variable}"`
- Use `"$@"` not `$@` for argument forwarding
- Single quotes for literal strings without expansion

## Reference

See skill: `shell-patterns` for comprehensive shell scripting patterns and safety practices.
