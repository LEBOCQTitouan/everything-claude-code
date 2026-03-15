---
name: rust-testing
description: Rust testing patterns including cfg(test) modules, integration tests, property testing with proptest, coverage with tarpaulin, and fuzzing.
origin: ECC
---

# Rust Testing Patterns

Testing patterns for Rust applications using built-in test framework and community tools.

## When to Activate

- Writing tests for Rust code
- Setting up test infrastructure for Rust projects
- Testing async code with tokio
- Adding coverage to Rust crates

## Unit Tests (cfg(test))

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_input() {
        let result = parse("42");
        assert_eq!(result, Ok(42));
    }

    #[test]
    fn parse_invalid_input() {
        let result = parse("abc");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("invalid"));
    }

    #[test]
    #[should_panic(expected = "index out of bounds")]
    fn panics_on_invalid_index() {
        let v: Vec<i32> = vec![];
        let _ = v[0];
    }
}
```

## Integration Tests

```rust
// tests/api_test.rs
use my_crate::App;

#[tokio::test]
async fn health_check_returns_200() {
    let app = App::new().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/health", app.address()))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 200);
}
```

## Test Helpers and Fixtures

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn make_user(name: &str) -> User {
        User {
            id: Uuid::new_v4(),
            name: name.to_string(),
            email: format!("{}@test.com", name),
            active: true,
        }
    }

    #[test]
    fn deactivate_user() {
        let user = make_user("john");
        let result = user.deactivate();
        assert!(!result.active);
    }
}
```

## Property-Based Testing (proptest)

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn slug_is_always_lowercase(input in "[a-zA-Z0-9 ]{1,100}") {
        let slug = to_slug(&input);
        assert_eq!(slug, slug.to_lowercase());
    }

    #[test]
    fn roundtrip_serialization(user in arb_user()) {
        let json = serde_json::to_string(&user).unwrap();
        let decoded: User = serde_json::from_str(&json).unwrap();
        assert_eq!(user, decoded);
    }
}

// Custom arbitrary
fn arb_user() -> impl Strategy<Value = User> {
    ("[a-z]{3,20}", "[a-z]+@test\\.com").prop_map(|(name, email)| {
        User { name, email, active: true }
    })
}
```

## Async Testing

```rust
#[tokio::test]
async fn concurrent_operations() {
    let (tx, mut rx) = tokio::sync::mpsc::channel(10);

    tokio::spawn(async move {
        tx.send(42).await.unwrap();
    });

    let value = rx.recv().await.unwrap();
    assert_eq!(value, 42);
}

// With timeout
#[tokio::test]
async fn operation_completes_within_timeout() {
    let result = tokio::time::timeout(
        Duration::from_secs(5),
        long_running_operation(),
    ).await;

    assert!(result.is_ok());
}
```

## Coverage

```bash
# Using tarpaulin
cargo install cargo-tarpaulin
cargo tarpaulin --out Html --output-dir coverage/

# Using llvm-cov
cargo install cargo-llvm-cov
cargo llvm-cov --html --output-dir coverage/
```

## Running Tests

```bash
cargo test                      # Run all tests
cargo test -- --nocapture       # Show println output
cargo test test_name            # Run specific test
cargo test --test integration   # Run integration tests only
cargo test -p my-crate          # Run tests for specific crate
cargo test -- --ignored         # Run ignored tests
cargo nextest run               # Faster test runner
```

## Quick Reference

| Tool | Purpose |
|------|---------|
| `cargo test` | Built-in test runner |
| `cargo nextest` | Faster parallel test runner |
| `proptest` | Property-based testing |
| `cargo-tarpaulin` | Code coverage |
| `cargo-llvm-cov` | LLVM-based coverage |
| `cargo-fuzz` | Fuzzing with libFuzzer |
| `miri` | Undefined behavior detection |
| `loom` | Concurrency testing |
