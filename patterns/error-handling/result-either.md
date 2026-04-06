---
name: result-either
category: error-handling
tags: [error-handling, result, either, type-safe]
languages: [rust, go, python, typescript]
difficulty: beginner
---

## Intent

Represent the outcome of a fallible operation as an explicit type that forces callers to handle both success and failure paths, eliminating unhandled exceptions and null-pointer surprises.

## Problem

Exceptions are invisible in function signatures — callers can forget to catch them. Returning null/nil for errors loses context. You need a mechanism that makes failure a first-class value the type system can enforce.

## Solution

Return a sum type (Result, Either, or tuple) that encodes success or failure. The caller must destructure the return value before accessing the success payload, making error handling impossible to skip.

**Language matrix:**

| Language | Mechanism | Type |
|----------|-----------|------|
| Rust | `Result<T, E>` | Native enum |
| Go | `(value, error)` tuple | Language convention |
| Python | Exceptions (stdlib); `result` lib for Result type | Convention / library |
| TypeScript | Discriminated union or `neverthrow` | Library / pattern |

## Language Implementations

### Rust

```rust
use std::fs;
use std::io;

fn read_config(path: &str) -> Result<String, io::Error> {
    fs::read_to_string(path)
}

fn main() {
    match read_config("config.toml") {
        Ok(content) => println!("loaded: {}", content.len()),
        Err(e) => eprintln!("failed to load config: {e}"),
    }
}
```

### Go

```go
func readConfig(path string) (string, error) {
    data, err := os.ReadFile(path)
    if err != nil {
        return "", fmt.Errorf("read config: %w", err)
    }
    return string(data), nil
}

func main() {
    content, err := readConfig("config.toml")
    if err != nil {
        log.Fatal(err)
    }
    fmt.Printf("loaded: %d bytes\n", len(content))
}
```

### Python

Idiomatic Python uses exceptions, but explicit Result types are possible:

```python
from dataclasses import dataclass
from typing import Generic, TypeVar, Union

T = TypeVar("T")
E = TypeVar("E")

@dataclass(frozen=True)
class Ok(Generic[T]):
    value: T

@dataclass(frozen=True)
class Err(Generic[E]):
    error: E

Result = Union[Ok[T], Err[E]]

def read_config(path: str) -> Result[str, str]:
    try:
        with open(path) as f:
            return Ok(f.read())
    except FileNotFoundError:
        return Err(f"file not found: {path}")

match read_config("config.toml"):
    case Ok(content):
        print(f"loaded: {len(content)} bytes")
    case Err(msg):
        print(f"error: {msg}")
```

### TypeScript

```typescript
type Result<T, E> =
  | { ok: true; value: T }
  | { ok: false; error: E };

function readConfig(path: string): Result<string, string> {
  try {
    const content = fs.readFileSync(path, "utf-8");
    return { ok: true, value: content };
  } catch {
    return { ok: false, error: `failed to read ${path}` };
  }
}

const result = readConfig("config.toml");
if (result.ok) {
  console.log(`loaded: ${result.value.length} bytes`);
} else {
  console.error(result.error);
}
```

## When to Use

- When callers must explicitly handle failure — no silent swallowing.
- When error context needs to flow through the call stack as data.
- When building libraries where exception behavior would surprise users.

## When NOT to Use

- For truly exceptional, unrecoverable conditions (use panic/throw).
- When the ecosystem convention is exceptions (e.g., idiomatic Python) and deviation would confuse collaborators.

## Anti-Patterns

- Unwrapping Result without handling the error case (`.unwrap()` in production Rust).
- Returning a Result but setting the error to a generic string with no structure.
- Mixing Result returns with thrown exceptions in the same API.

## Related Patterns

- [railway-oriented](railway-oriented.md) — chains Result-returning functions for pipeline composition.
- [error-wrapping](error-wrapping.md) — adds context layers to error values.
- [structured-errors](structured-errors.md) — gives errors machine-readable structure.

## References

- Rust std::result: https://doc.rust-lang.org/std/result/
- Go error handling: https://go.dev/blog/error-handling-and-go
- neverthrow (TypeScript): https://github.com/supermacro/neverthrow
