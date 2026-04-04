---
paths:
  - "**/*.rs"
  - "**/Cargo.toml"
applies-to: { languages: [rust] }
---
# Rust Security

> Extends [common/security.md](../common/security.md) with Rust-specific rules.

## Unsafe Code

- No `unsafe` blocks without a `// SAFETY:` comment proving the invariant holds
- Minimize `unsafe` scope — wrap it in a safe abstraction immediately
- Audit every `unsafe` block during code review
- Prefer `zerocopy`, `bytemuck`, or similar audited crates over hand-rolled `unsafe` transmutes

## Dependency Auditing

- Run `cargo audit` in CI — block merges on high-severity advisories
- Pin transitive dependencies with `Cargo.lock` committed to the repo
- Prefer crates with active maintenance and security track records
- Avoid crates that pull in large dependency trees for small utilities

## Input Validation

- Validate all external input at the boundary before it enters domain types
- Use newtype wrappers to enforce validation has occurred (see patterns.md)
- Never use `.parse::<i64>().unwrap()` on user input — always propagate errors
- Limit string and collection sizes from untrusted sources

## SQL

- Always use parameterised queries — `sqlx::query!` macros enforce this at compile time
- Never format user input into SQL strings
- Use least-privilege database roles per service

## Secrets and Environment

- Never log secrets, tokens, or passwords — ensure `Debug` impls redact sensitive fields:

```rust
#[derive(Debug)]
pub struct Config {
    pub database_url: String,
    #[debug(skip)]
    pub api_key: String,
}
```

- Load secrets from environment variables or a secrets manager — never hardcode
- Use `secrecy::Secret<String>` to prevent accidental exposure of sensitive values

## Serialisation

- Use `serde` with explicit `#[serde(deny_unknown_fields)]` on inbound API types
- Validate deserialized values — `serde` parsing is not the same as domain validation
- Avoid `serde_json::Value` for user-controlled input; prefer typed structs

## DoS Prevention

- Set timeouts on all outbound HTTP calls (`reqwest::ClientBuilder::timeout`)
- Limit request body size at the framework level (Axum: `DefaultBodyLimit`)
- Rate-limit endpoints that accept unauthenticated input
