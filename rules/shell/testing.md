---
paths:
  - "**/*.sh"
  - "**/*.bash"
  - "**/*.zsh"
---
# Shell Testing

> This file extends [common/testing.md](../common/testing.md) with shell specific content.

## Framework

- **Bats-core** (Bash Automated Testing System) for unit tests
- Install: `brew install bats-core` or `npm install -g bats`

## Running Tests

```bash
bats tests/
bats --tap tests/*.bats
```

## Test Structure

```bash
@test "script exits with error on missing argument" {
    run ./my-script.sh
    [ "$status" -eq 1 ]
    [[ "$output" =~ "Usage:" ]]
}
```

## Reference

See skill: `shell-testing` for detailed shell testing patterns with Bats.
