---
paths:
  - "**/*.rs"
  - "**/Cargo.toml"
  - "**/Cargo.lock"
---
# Rust Coding Style

> Extends [common/coding-style.md](../common/coding-style.md) with Rust-specific rules.

## Formatting

- **rustfmt** is mandatory — run `cargo fmt` before every commit, no exceptions
- **clippy** is mandatory — `cargo clippy -- -D warnings` must pass with zero warnings
- Use `#![deny(clippy::all, clippy::pedantic)]` in lib crates

## Naming

- Types, traits, enums: `PascalCase`
- Functions, variables, modules: `snake_case`
- Constants and statics: `SCREAMING_SNAKE_CASE`
- Lifetime parameters: short lowercase (`'a`, `'b`, `'conn`)

## Ownership and Borrowing

- Prefer borrowing (`&T`, `&mut T`) over cloning unless ownership is genuinely needed
- Prefer `&str` over `&String` in function parameters
- Prefer `&[T]` over `&Vec<T>` in function parameters
- Return owned types (`String`, `Vec<T>`) when the caller needs ownership
- Use `Cow<'_, str>` when the value may or may not need allocation

## Error Handling

- No `.unwrap()` or `.expect()` in production code — propagate with `?`
- Use `thiserror` for library/domain errors
- Use `anyhow` only in binary crates (`main.rs`) or integration tests
- Each module defines its own error enum — never use `Box<dyn Error>` in public APIs
- Always add context when wrapping errors:

```rust
// BAD
file.read_to_string(&mut buf)?;

// GOOD
file.read_to_string(&mut buf)
    .with_context(|| format!("failed to read {}", path.display()))?;
```

## Types and Traits

- Derive `Debug` on all public types
- Derive `Clone`, `PartialEq`, `Eq`, `Hash` only when actually needed
- No `unsafe` blocks without a `// SAFETY:` comment explaining the invariant
- Use newtypes to prevent mixing semantically distinct values:

```rust
struct UserId(Uuid);
struct OrderId(Uuid);
// Not: fn process(user: Uuid, order: Uuid)
```

## Async

- Use `tokio` as the async runtime — do not mix runtimes
- Prefer `async fn` over manually boxing futures
- Never block in async context — use `tokio::task::spawn_blocking` for CPU-heavy work
- Annotate long-lived futures with `#[allow(clippy::future_not_send)]` only when justified

## Reference

See skill: `rust-api` (if installed) for comprehensive Rust patterns.
