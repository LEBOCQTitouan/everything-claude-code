---
name: shell-testing
description: Shell script testing patterns using Bats-core, mocking, assertions, and CI integration for reliable shell script testing.
origin: ECC
---

# Shell Testing Patterns

Testing patterns for shell scripts using Bats-core and related tools.

## When to Activate

- Writing tests for shell scripts
- Setting up shell testing infrastructure
- Debugging failing shell tests
- Adding CI for shell scripts

## Bats-core Setup

### Installation

```bash
# macOS
brew install bats-core

# npm
npm install -g bats

# From source
git clone https://github.com/bats-core/bats-core.git
cd bats-core && ./install.sh /usr/local
```

### Helper Libraries

```bash
git submodule add https://github.com/bats-core/bats-support test/test_helper/bats-support
git submodule add https://github.com/bats-core/bats-assert test/test_helper/bats-assert
git submodule add https://github.com/bats-core/bats-file test/test_helper/bats-file
```

## Test Structure

```bash
#!/usr/bin/env bats

setup() {
    load 'test_helper/bats-support/load'
    load 'test_helper/bats-assert/load'

    TEST_TEMP_DIR="$(mktemp -d)"
}

teardown() {
    rm -rf "$TEST_TEMP_DIR"
}

@test "script prints usage when no arguments" {
    run ./my-script.sh
    assert_failure
    assert_output --partial "Usage:"
}

@test "script processes valid input" {
    run ./my-script.sh --input "test-data"
    assert_success
    assert_output --partial "Processed"
}

@test "script fails on invalid file" {
    run ./my-script.sh --input "/nonexistent"
    assert_failure
    assert_output --partial "not found"
}
```

## Mocking

```bash
# Mock external commands by creating functions
setup() {
    # Mock curl to return test data
    curl() {
        echo '{"status": "ok"}'
    }
    export -f curl
}

@test "api call handles success response" {
    run ./fetch-status.sh
    assert_success
    assert_output --partial "ok"
}
```

## Running Tests

```bash
bats tests/

bats --formatter tap tests/

bats tests/my-script.bats

# Parallel execution
bats --jobs 4 tests/
```

## Quick Reference

| Command | Purpose |
|---------|---------|
| `run command` | Execute and capture status/output |
| `assert_success` | Status code is 0 |
| `assert_failure` | Status code is non-zero |
| `assert_output "text"` | Exact output match |
| `assert_output --partial "text"` | Partial output match |
| `assert_line "text"` | Specific line match |
| `refute_output --partial "text"` | Output does not contain |
