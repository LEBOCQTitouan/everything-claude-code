---
name: panic-recover
category: error-handling
tags: [error-handling, panic, recover, unwind, last-resort]
languages: [rust, go]
difficulty: intermediate
---

## Intent

Handle truly unrecoverable errors by unwinding the stack (panic) and optionally catching the unwind at a boundary (recover) to prevent process termination, serving as a last-resort safety net.

## Problem

Some errors are unrecoverable within the current context — invariant violations, out-of-bounds access, impossible states. Returning a Result/error is inappropriate because the caller cannot meaningfully handle the condition. You need a mechanism to abort the current operation while optionally preserving the process.

## Solution

Use panic to signal an unrecoverable error that unwinds the stack. At process or request boundaries, use a recovery mechanism to catch panics and convert them to errors, preventing one bad request from crashing the entire server.

**Language matrix:**

| Language | Panic mechanism | Recovery mechanism |
|----------|----------------|-------------------|
| Rust | `panic!()` macro, unwind or abort | `std::panic::catch_unwind()` |
| Go | `panic()` built-in | `recover()` in deferred function |

> Python and TypeScript use exceptions for all error severities — there is no distinct panic/recover separation. `SystemExit` and `KeyboardInterrupt` in Python are the closest analogs but are not the same pattern.

## Language Implementations

### Rust

Panic for invariant violations:

```rust
fn get_item(items: &[String], index: usize) -> &str {
    items.get(index).expect("index must be within bounds")
}
```

Catching panics at a boundary (e.g., FFI or request handler):

```rust
use std::panic;

fn handle_request(input: &str) -> Result<String, String> {
    let result = panic::catch_unwind(|| {
        process(input) // may panic on bad invariant
    });

    match result {
        Ok(value) => Ok(value),
        Err(_) => Err("internal error: handler panicked".into()),
    }
}
```

> In Rust, `catch_unwind` only catches unwinding panics. If `panic = "abort"` is set in Cargo.toml, panics terminate the process immediately. Use `catch_unwind` sparingly — prefer `Result` for expected errors.

### Go

Panic for programmer errors:

```go
func MustParseURL(raw string) *url.URL {
    u, err := url.Parse(raw)
    if err != nil {
        panic(fmt.Sprintf("invalid URL %q: %v", raw, err))
    }
    return u
}
```

Recover at request boundary:

```go
func recoveryMiddleware(next http.Handler) http.Handler {
    return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
        defer func() {
            if rec := recover(); rec != nil {
                log.Printf("panic recovered: %v\n%s", rec, debug.Stack())
                http.Error(w, "internal server error", http.StatusInternalServerError)
            }
        }()
        next.ServeHTTP(w, r)
    })
}
```

## When to Use

- For invariant violations that indicate a bug (index out of bounds, impossible enum match).
- At process/request boundaries as a safety net to prevent total shutdown.
- In `Must*` constructor functions where failure means a programming error.

## When NOT to Use

- For expected errors (file not found, invalid input, network timeout) — use Result/error.
- As a general control flow mechanism — panic is for bugs, not business logic.
- Across goroutine/thread boundaries without a recover — panics in goroutines crash the program.

## Anti-Patterns

- Using panic for input validation — callers cannot recover gracefully.
- Recovering from panics and silently continuing without logging.
- Panicking in library code that should return errors to let callers decide.
- Using `catch_unwind` as a substitute for proper error handling in Rust.

## Related Patterns

- [result-either](result-either.md) — the preferred mechanism for expected, recoverable errors.
- [error-wrapping](error-wrapping.md) — recovered panics should be wrapped with context before returning.
- [structured-errors](structured-errors.md) — panic messages lack structure; convert to typed errors after recovery.

## References

- Rust panic documentation: https://doc.rust-lang.org/std/macro.panic.html
- Rust catch_unwind: https://doc.rust-lang.org/std/panic/fn.catch_unwind.html
- Go blog — Defer, Panic, and Recover: https://go.dev/blog/defer-panic-and-recover
- Effective Go — Panic: https://go.dev/doc/effective_go#panic
