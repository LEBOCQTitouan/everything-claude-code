---
name: rust-patterns
description: Idiomatic Rust patterns including ownership, error handling, trait design, async patterns, and best practices for building robust Rust applications.
origin: ECC
---

# Rust Development Patterns

Idiomatic Rust patterns and best practices for building safe, performant, and maintainable applications.

## When to Activate

- Writing new Rust code
- Reviewing Rust code
- Designing Rust crates or modules
- Refactoring existing Rust code

## Core Principles

### 1. Ownership and Borrowing

```rust
// Good: Borrow when you don't need ownership
fn process(data: &str) -> String {
    data.to_uppercase()
}

// Good: Take ownership when you need it
fn consume(data: String) -> Result<(), Error> {
    store.save(data)?;
    Ok(())
}

// Good: Use Cow for flexible ownership
fn normalize(input: &str) -> Cow<'_, str> {
    if input.contains(' ') {
        Cow::Owned(input.replace(' ', "_"))
    } else {
        Cow::Borrowed(input)
    }
}

// Prefer &str over &String, &[T] over &Vec<T>
fn find(items: &[Item], name: &str) -> Option<&Item> {
    items.iter().find(|i| i.name == name)
}
```

### 2. Error Handling

```rust
// Define domain errors with thiserror
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("user {0} not found")]
    NotFound(String),
    #[error("validation failed: {0}")]
    Validation(String),
    #[error("database error")]
    Database(#[from] sqlx::Error),
}

// Propagate with context
fn load_config(path: &Path) -> Result<Config, AppError> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read config from {}", path.display()))?;

    let config: Config = toml::from_str(&content)
        .with_context(|| "failed to parse config")?;

    Ok(config)
}

// Never use unwrap in production code
// Use ? for propagation, expect() only with invariant explanation
```

### 3. Trait Design

```rust
// Small, focused traits
pub trait Repository {
    type Item;
    type Error;

    fn find_by_id(&self, id: &str) -> Result<Option<Self::Item>, Self::Error>;
    fn save(&self, item: &Self::Item) -> Result<(), Self::Error>;
}

// Blanket implementations
impl<T: Display> Loggable for T {
    fn log(&self) {
        println!("{}", self);
    }
}

// Trait objects vs generics
// Use generics: when you need zero-cost abstraction
// Use trait objects (dyn Trait): when you need runtime polymorphism
fn process_static(handler: impl Handler) { } // Monomorphized
fn process_dynamic(handler: &dyn Handler) { } // Dynamic dispatch
```

### 4. Builder Pattern

```rust
#[derive(Debug)]
pub struct Config {
    host: String,
    port: u16,
    workers: usize,
}

#[derive(Default)]
pub struct ConfigBuilder {
    host: Option<String>,
    port: Option<u16>,
    workers: Option<usize>,
}

impl ConfigBuilder {
    pub fn host(mut self, host: impl Into<String>) -> Self {
        self.host = Some(host.into());
        self
    }

    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    pub fn build(self) -> Result<Config, &'static str> {
        Ok(Config {
            host: self.host.ok_or("host is required")?,
            port: self.port.unwrap_or(8080),
            workers: self.workers.unwrap_or_else(num_cpus::get),
        })
    }
}
```

### 5. Async Patterns

```rust
// Use tokio for async runtime
#[tokio::main]
async fn main() -> Result<()> {
    let server = Server::bind(&addr).serve(app);

    // Graceful shutdown
    let graceful = server.with_graceful_shutdown(shutdown_signal());
    graceful.await?;
    Ok(())
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c().await.expect("install signal handler");
}

// Concurrent operations with join
let (users, orders) = tokio::try_join!(
    fetch_users(&client),
    fetch_orders(&client),
)?;

// CPU-bound work off the async runtime
let result = tokio::task::spawn_blocking(move || {
    compute_heavy_task(data)
}).await?;
```

### 6. Iterator Patterns

```rust
// Chain iterators for efficient processing
let active_emails: Vec<String> = users.iter()
    .filter(|u| u.is_active)
    .map(|u| u.email.clone())
    .collect();

// Custom iterator
impl Iterator for Fibonacci {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.a + self.b;
        self.a = self.b;
        self.b = next;
        Some(self.a)
    }
}
```

## Quick Reference

| Pattern | Description |
|---------|-------------|
| `&str` over `&String` | Prefer slices over owned references |
| `?` operator | Propagate errors concisely |
| `thiserror` | Derive Error implementations |
| `Cow<'_, str>` | Flexible ownership — borrow or own |
| `impl Trait` | Zero-cost static dispatch |
| `#[derive(Debug)]` | Always derive Debug on public types |
| `Default` | Implement for builder defaults |
| `From`/`Into` | Idiomatic type conversions |

## Anti-Patterns

```rust
// Bad: unwrap in production
let value = map.get("key").unwrap();

// Bad: Clone to satisfy borrow checker
let data = expensive_data.clone(); // Think about lifetimes first

// Bad: Box<dyn Error> in library code
fn process() -> Result<(), Box<dyn Error>> { } // Use custom errors

// Bad: String for everything
fn process(name: String, id: String) { } // Use newtypes

// Bad: Blocking in async context
async fn handler() {
    std::thread::sleep(Duration::from_secs(1)); // Use tokio::time::sleep
}
```
