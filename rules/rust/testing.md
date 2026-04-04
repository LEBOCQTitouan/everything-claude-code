---
paths:
  - "**/*.rs"
  - "**/Cargo.toml"
applies-to: { languages: [rust] }
---
# Rust Testing

> Extends [common/testing.md](../common/testing.md) with Rust-specific rules.

## Structure

- Unit tests live in the same file as the code under test, in a `#[cfg(test)]` module
- Integration tests live in `tests/` at the crate root
- Use `tests/common/mod.rs` for shared test helpers

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_email() {
        assert!(Email::parse("user@example.com").is_ok());
    }
}
```

## Assertions

- Prefer `assert_eq!` / `assert_ne!` over `assert!` — better failure messages
- Use `pretty_assertions` crate for complex struct comparisons
- Test `Err` paths explicitly — do not only test the happy path

## Async Tests

- Use `#[tokio::test]` for async test functions
- Use `#[tokio::test(flavor = "multi_thread")]` only when testing concurrent behaviour

```rust
#[tokio::test]
async fn creates_user() {
    let db = setup_test_db().await;
    let result = UserService::create(&db, "alice@example.com").await;
    assert!(result.is_ok());
}
```

## Test Naming

- Name tests after the behaviour: `returns_error_for_duplicate_email`, not `test_create_user_2`
- Group related tests in a nested `mod` with a descriptive name:

```rust
mod when_email_is_invalid {
    #[test]
    fn rejects_missing_at_sign() { ... }

    #[test]
    fn rejects_empty_string() { ... }
}
```

## Mocking and Isolation

- Use trait objects or generic bounds to inject dependencies — avoid hard-coded I/O in unit tests
- Use `mockall` for mocking traits when integration with real dependencies is impractical
- Prefer in-memory SQLite or a test PostgreSQL container over mocking the database layer

## Coverage

- Run `cargo tarpaulin` or `cargo llvm-cov` to measure coverage
- 80% line coverage minimum; 100% on domain logic and error paths
