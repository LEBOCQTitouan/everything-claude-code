---
name: structured-errors
category: error-handling
tags: [error-handling, typed-errors, enum, machine-readable]
languages: [rust, go, python, typescript]
difficulty: intermediate
---

## Intent

Define errors as structured, machine-readable types with distinct variants, enabling callers to match on specific failure modes and respond programmatically rather than parsing error strings.

## Problem

String-based errors are fragile — callers must match on substrings that break when messages change. You need errors that are typed, enumerable, and carry structured metadata so that recovery logic can branch on error kind reliably.

## Solution

Define error types as enums (Rust), sentinel values or typed structs (Go), exception hierarchies (Python), or discriminated unions (TypeScript). Each variant carries the data relevant to that failure mode.

**Language matrix:**

| Language | Mechanism | Type |
|----------|-----------|------|
| Rust | `enum` + `thiserror` derive | Native enum |
| Go | Sentinel errors, typed structs implementing `error` | Convention |
| Python | Exception class hierarchy | Native |
| TypeScript | Discriminated union or class hierarchy | Pattern |

## Language Implementations

### Rust

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("file not found: {path}")]
    NotFound { path: String },
    #[error("parse error at line {line}: {detail}")]
    ParseError { line: usize, detail: String },
    #[error("validation failed: {0}")]
    Validation(String),
}

fn load(path: &str) -> Result<Config, ConfigError> {
    if !std::path::Path::new(path).exists() {
        return Err(ConfigError::NotFound { path: path.into() });
    }
    // ...
}

// Caller matches on variant:
match load("app.toml") {
    Err(ConfigError::NotFound { path }) => create_default(&path),
    Err(e) => return Err(e.into()),
    Ok(cfg) => cfg,
}
```

### Go

```go
var ErrNotFound = errors.New("not found")

type ParseError struct {
    Line   int
    Detail string
}

func (e *ParseError) Error() string {
    return fmt.Sprintf("parse error at line %d: %s", e.Line, e.Detail)
}

func load(path string) (*Config, error) {
    _, err := os.Stat(path)
    if errors.Is(err, os.ErrNotExist) {
        return nil, fmt.Errorf("config %s: %w", path, ErrNotFound)
    }
    // ...
}

// Caller uses errors.Is / errors.As:
var pe *ParseError
if errors.As(err, &pe) {
    log.Printf("fix line %d: %s", pe.Line, pe.Detail)
}
```

### Python

```python
class ConfigError(Exception):
    """Base config error."""

class ConfigNotFoundError(ConfigError):
    def __init__(self, path: str) -> None:
        super().__init__(f"file not found: {path}")
        self.path = path

class ConfigParseError(ConfigError):
    def __init__(self, line: int, detail: str) -> None:
        super().__init__(f"parse error at line {line}: {detail}")
        self.line = line
        self.detail = detail

try:
    config = load("app.toml")
except ConfigNotFoundError as e:
    config = create_default(e.path)
except ConfigParseError as e:
    print(f"fix line {e.line}: {e.detail}")
```

### TypeScript

```typescript
type ConfigError =
  | { kind: "not_found"; path: string }
  | { kind: "parse_error"; line: number; detail: string }
  | { kind: "validation"; message: string };

function handleError(err: ConfigError): void {
  switch (err.kind) {
    case "not_found":
      createDefault(err.path);
      break;
    case "parse_error":
      console.error(`fix line ${err.line}: ${err.detail}`);
      break;
    case "validation":
      console.error(err.message);
      break;
  }
}
```

## When to Use

- When callers need to distinguish between failure modes programmatically.
- When errors cross module boundaries and must be part of the public API contract.
- When error metadata (line numbers, field names, codes) is needed for recovery or reporting.

## When NOT to Use

- For internal errors that are always logged and never matched on — a simple string suffices.
- When there is only one possible failure mode.

## Anti-Patterns

- Catch-all error variants like `Other(String)` that become a dumping ground.
- Exposing internal implementation details through error variants.
- Making error enums non-exhaustive when callers need complete matching.

## Related Patterns

- [result-either](result-either.md) — structured errors are the `E` in `Result<T, E>`.
- [error-wrapping](error-wrapping.md) — structured errors can wrap lower-level causes.
- [railway-oriented](railway-oriented.md) — pipeline steps produce typed error variants.

## References

- thiserror crate: https://docs.rs/thiserror/
- Go errors package: https://pkg.go.dev/errors
- Python exception hierarchy: https://docs.python.org/3/library/exceptions.html
