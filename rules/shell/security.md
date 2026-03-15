---
paths:
  - "**/*.sh"
  - "**/*.bash"
  - "**/*.zsh"
---
# Shell Security

> This file extends [common/security.md](../common/security.md) with shell specific content.

## Command Injection

- Never use `eval` with user input
- Quote all variable expansions: `"${var}"`
- Use arrays for command construction, not string concatenation:

```bash
# GOOD
cmd=("curl" "-s" "-H" "Authorization: Bearer ${token}" "${url}")
"${cmd[@]}"

# BAD
cmd="curl -s -H 'Authorization: Bearer $token' $url"
eval "$cmd"
```

## Temporary Files

- Use `mktemp` for temporary files, never hardcoded paths:
  ```bash
  tmpfile=$(mktemp) || exit 1
  trap 'rm -f "$tmpfile"' EXIT
  ```

## Secret Handling

- Never echo secrets to stdout or logs
- Use `read -rs` for password input
- Clear sensitive variables: `unset password`
