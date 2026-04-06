---
name: context-propagation
category: idioms
tags: [idiom, go]
languages: [go]
difficulty: intermediate
---

## Intent

Thread cancellation signals, deadlines, and request-scoped values through the call chain using `context.Context`, enabling cooperative shutdown and timeout propagation without global state.

## Problem

Long-running operations (HTTP requests, database queries, goroutines) need a way to know when their work is no longer needed. Without a cancellation mechanism, leaked goroutines accumulate, timed-out requests continue consuming resources, and request-scoped metadata (trace IDs, auth tokens) must be passed as extra parameters.

## Solution

Accept `context.Context` as the first parameter of every function that performs I/O or may block. Use `context.WithCancel`, `context.WithTimeout`, or `context.WithDeadline` to create derived contexts. Check `ctx.Done()` in long-running loops.

## Language Implementations

### Go

```go
import (
    "context"
    "fmt"
    "time"
)

func fetchData(ctx context.Context, url string) ([]byte, error) {
    // Respect parent deadline or cancellation
    ctx, cancel := context.WithTimeout(ctx, 5*time.Second)
    defer cancel()

    req, err := http.NewRequestWithContext(ctx, "GET", url, nil)
    if err != nil {
        return nil, fmt.Errorf("creating request: %w", err)
    }

    resp, err := http.DefaultClient.Do(req)
    if err != nil {
        return nil, fmt.Errorf("fetching %s: %w", url, err)
    }
    defer resp.Body.Close()

    return io.ReadAll(resp.Body)
}

// Long-running worker that checks for cancellation
func processItems(ctx context.Context, items []string) error {
    for _, item := range items {
        select {
        case <-ctx.Done():
            return ctx.Err() // context.Canceled or context.DeadlineExceeded
        default:
            if err := process(item); err != nil {
                return fmt.Errorf("processing %s: %w", item, err)
            }
        }
    }
    return nil
}
```

## When to Use

- In every function that performs I/O, network calls, or may block.
- When coordinating shutdown across multiple goroutines.
- When propagating request-scoped metadata (trace IDs, auth tokens).

## When NOT to Use

- For passing optional function parameters (use functional options instead).
- For storing application-level configuration (use dependency injection).
- In pure computation functions that cannot be meaningfully cancelled.

## Anti-Patterns

- Storing `context.Context` in a struct field (contexts should flow through call chains).
- Using `context.Background()` deep in the call stack instead of propagating the parent.
- Putting large objects in context values (use context for metadata, not data).

## Related Patterns

- [error-wrapping](error-wrapping.md) -- context errors (`DeadlineExceeded`) are wrapped through the chain.
- [functional-options](functional-options.md) -- for configuration, not request-scoped data.

## References

- Go Blog -- Go Concurrency Patterns: Context: https://go.dev/blog/context
- Go standard library -- context package: https://pkg.go.dev/context
