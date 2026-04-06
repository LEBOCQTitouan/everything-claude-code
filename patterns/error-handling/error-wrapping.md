---
name: error-wrapping
category: error-handling
tags: [error-handling, context, wrapping, chain]
languages: [rust, go, python, typescript]
difficulty: beginner
---

## Intent

Add contextual information to errors as they propagate up the call stack, preserving the original cause while making each layer's contribution to the failure visible.

## Problem

A low-level "file not found" error is meaningless without context — which file? Which operation? Why was it needed? You need to wrap errors with context at each layer without losing the original cause.

## Solution

At each call-site boundary, wrap the error with a message describing what the current layer was trying to do. The original error is preserved as the "cause" or "source," forming a chain that can be inspected for debugging or matched for recovery.

**Language matrix:**

| Language | Mechanism | Type |
|----------|-----------|------|
| Rust | `anyhow::context()`, `thiserror` `#[from]` | Library |
| Go | `fmt.Errorf("...: %w", err)` | Stdlib |
| Python | `raise XError(...) from original` | Native |
| TypeScript | `new Error("msg", { cause: original })` | Native (ES2022) |

## Language Implementations

### Rust

```rust
use anyhow::{Context, Result};
use std::fs;

fn load_config(path: &str) -> Result<Config> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("failed to read config file: {path}"))?;
    let config: Config = toml::from_str(&content)
        .context("failed to parse config as TOML")?;
    Ok(config)
}
// Error chain: "failed to read config file: app.toml" -> "No such file (os error 2)"
```

### Go

```go
func loadConfig(path string) (*Config, error) {
    data, err := os.ReadFile(path)
    if err != nil {
        return nil, fmt.Errorf("load config %s: %w", path, err)
    }
    var cfg Config
    if err := toml.Unmarshal(data, &cfg); err != nil {
        return nil, fmt.Errorf("parse config %s: %w", path, err)
    }
    return &cfg, nil
}

// Unwrap chain with errors.Is / errors.As
```

### Python

```python
class ConfigError(Exception):
    """Raised when configuration loading fails."""

def load_config(path: str) -> dict:
    try:
        with open(path) as f:
            return toml.load(f)
    except FileNotFoundError as e:
        raise ConfigError(f"config file not found: {path}") from e
    except toml.TOMLDecodeError as e:
        raise ConfigError(f"invalid TOML in {path}") from e

# Access chain: err.__cause__ points to the original exception
```

### TypeScript

```typescript
class ConfigError extends Error {
  constructor(message: string, cause?: Error) {
    super(message, { cause });
    this.name = "ConfigError";
  }
}

function loadConfig(path: string): Config {
  try {
    const content = fs.readFileSync(path, "utf-8");
    return JSON.parse(content);
  } catch (e) {
    throw new ConfigError(`failed to load config: ${path}`, e as Error);
  }
}

// Access chain: error.cause
```

## When to Use

- At every layer boundary where an error crosses from one module to another.
- When debugging requires knowing the full context chain.
- When different callers need different context for the same underlying failure.

## When NOT to Use

- When the error already contains sufficient context (avoid redundant wrapping).
- At leaf functions where there is nothing meaningful to add.

## Anti-Patterns

- Wrapping without adding new information: `fmt.Errorf("error: %w", err)`.
- Losing the original cause by creating a new error instead of wrapping.
- Including sensitive data (passwords, tokens) in error context messages.

## Related Patterns

- [result-either](result-either.md) — wrapping is applied to the error variant of a Result.
- [structured-errors](structured-errors.md) — wrapping with typed enums instead of strings.
- [railway-oriented](railway-oriented.md) — `.map_err()` is the wrapping point in railway chains.

## References

- anyhow crate: https://docs.rs/anyhow/
- Go error wrapping: https://go.dev/blog/go1.13-errors
- MDN Error cause: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Error/cause
