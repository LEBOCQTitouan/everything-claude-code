---
name: error-wrapping
category: idioms
tags: [idiom, go]
languages: [go]
difficulty: beginner
---

## Intent

Build structured error chains using `fmt.Errorf` with the `%w` verb, enabling callers to inspect root causes with `errors.Is` and `errors.As` while preserving context at each layer.

## Problem

Returning raw errors loses context about where and why the error occurred. String concatenation (`"failed: " + err.Error()`) destroys the error chain, preventing programmatic inspection. Callers cannot distinguish between transient and permanent failures.

## Solution

Wrap errors with `fmt.Errorf("context: %w", err)` to add human-readable context while preserving the original error for programmatic inspection. Use `errors.Is` for sentinel comparison and `errors.As` for type assertion through the chain.

## Language Implementations

### Go

```go
import (
    "errors"
    "fmt"
    "os"
)

var ErrNotFound = errors.New("not found")

func readConfig(path string) ([]byte, error) {
    data, err := os.ReadFile(path)
    if err != nil {
        if errors.Is(err, os.ErrNotExist) {
            return nil, fmt.Errorf("config %s: %w", path, ErrNotFound)
        }
        return nil, fmt.Errorf("reading config %s: %w", path, err)
    }
    return data, nil
}

func loadSettings() error {
    _, err := readConfig("/etc/app/config.yaml")
    if err != nil {
        return fmt.Errorf("loading settings: %w", err)
    }
    return nil
}

// Caller inspection:
// err := loadSettings()
// if errors.Is(err, ErrNotFound) { /* handle missing config */ }
//
// var pathErr *os.PathError
// if errors.As(err, &pathErr) { /* inspect filesystem details */ }
```

## When to Use

- At every function boundary where an error is returned from a callee.
- When callers need to distinguish error types programmatically.
- When debugging requires knowing the full call path.

## When NOT to Use

- When the error is a sentinel that should not carry additional context.
- When wrapping would expose internal implementation details to external callers.
- When using `%v` instead of `%w` is intentional to break the error chain.

## Anti-Patterns

- Using `%v` when you intend callers to unwrap the error (breaks `errors.Is`).
- Wrapping the same error multiple times with identical context.
- Logging the error AND returning it wrapped (causes duplicate log entries).

## Related Patterns

- [context-propagation](context-propagation.md) -- context carries metadata alongside error chains.
- [error-handling-strategy](../../error-handling/strategy.md) -- broader error handling architecture.

## References

- Go Blog -- Working with Errors in Go 1.13: https://go.dev/blog/go1.13-errors
- Go standard library -- errors package: https://pkg.go.dev/errors
