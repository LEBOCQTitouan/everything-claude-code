# ADR 0053: Handler Trait for Hook Dispatch

## Status

Accepted (2026-04-06)

## Context

The hook dispatch system in `ecc-app/src/hook/mod.rs` uses a large `match` statement to route hook IDs to handler functions. Adding a new handler requires modifying the central dispatch function — violating the Open-Closed Principle. The co-change coupling between the dispatch file and handler modules is high (38% co-change rate per evolution analysis).

## Decision

Introduce a `Handler` trait with a `HashMap`-based registry:

```rust
pub trait Handler: Send + Sync {
    fn hook_id(&self) -> &str;
    fn handle(&self, stdin: &str, ports: &HookPorts<'_>) -> HookResult;
}
```

The registry is checked before the existing match, allowing new handlers to be added by implementing the trait and registering, without modifying the dispatch function. The existing match remains as a fallback for handlers not yet migrated.

## Consequences

- New handlers can be added without touching `hook/mod.rs`
- The cartography handlers (`stop:cartography`, `start:cartography`) serve as the proof-of-concept migration
- The existing match statement is preserved for backward compatibility
- Future work can incrementally migrate more handlers to the registry
