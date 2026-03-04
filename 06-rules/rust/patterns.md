---
paths:
  - "**/*.rs"
  - "**/Cargo.toml"
---
# Rust Patterns

> Extends [common/patterns.md](../common/patterns.md) with Rust-specific idioms.

## Builder Pattern

Use the builder pattern for structs with many optional fields:

```rust
#[derive(Default)]
pub struct QueryBuilder {
    table: String,
    limit: Option<u32>,
    offset: Option<u32>,
}

impl QueryBuilder {
    pub fn table(mut self, name: impl Into<String>) -> Self {
        self.table = name.into(); self
    }
    pub fn limit(mut self, n: u32) -> Self {
        self.limit = Some(n); self
    }
}
```

## Type State Pattern

Encode state transitions in the type system to prevent invalid states at compile time:

```rust
struct Unverified;
struct Verified;

struct Email<State> {
    value: String,
    _state: PhantomData<State>,
}

impl Email<Unverified> {
    pub fn verify(self) -> Result<Email<Verified>, ValidationError> { ... }
}
```

## Repository Pattern

Define a trait per aggregate root; inject via generics or `dyn`:

```rust
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_id(&self, id: UserId) -> Result<Option<User>>;
    async fn save(&self, user: &User) -> Result<()>;
}
```

## Error Enum per Module

Each module owns its error type and maps it at the boundary:

```rust
#[derive(Debug, thiserror::Error)]
pub enum UserError {
    #[error("user {0} not found")]
    NotFound(UserId),
    #[error("email already taken")]
    DuplicateEmail,
    #[error(transparent)]
    Database(#[from] sqlx::Error),
}
```

## Newtype for Domain Primitives

Wrap primitives to make invalid states unrepresentable:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Email(String);

impl Email {
    pub fn parse(raw: &str) -> Result<Self, ValidationError> {
        if raw.contains('@') { Ok(Self(raw.to_owned())) }
        else { Err(ValidationError::InvalidEmail) }
    }
    pub fn as_str(&self) -> &str { &self.0 }
}
```

## Extension Traits

Add behaviour to foreign types without wrapping them:

```rust
trait ResultExt<T> {
    fn log_err(self) -> Self;
}

impl<T, E: std::fmt::Display> ResultExt<T> for Result<T, E> {
    fn log_err(self) -> Self {
        if let Err(ref e) = self { tracing::error!("{e}"); }
        self
    }
}
```
