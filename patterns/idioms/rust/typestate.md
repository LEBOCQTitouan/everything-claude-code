---
name: typestate
category: idioms
tags: [idiom, rust]
languages: [rust]
difficulty: advanced
---

## Intent

Encode valid state transitions of an object into the type system so that illegal transitions cause compile-time errors rather than runtime failures.

## Problem

State machines implemented with enums or boolean flags allow invalid transitions at runtime. A connection that has not been opened can be read from, or a builder can call `build()` before required fields are set. Runtime checks add overhead and are easy to forget.

## Solution

Represent each state as a distinct type (often a zero-sized struct). Transition methods consume `self` and return a new type representing the next state. `PhantomData` marks the state in generic wrappers. Only methods valid for the current state are available.

## Language Implementations

### Rust

```rust
use std::marker::PhantomData;

// State types -- zero-sized, no runtime cost
struct Draft;
struct Published;
struct Archived;

struct Article<S> {
    title: String,
    body: String,
    _state: PhantomData<S>,
}

impl Article<Draft> {
    pub fn new(title: String) -> Self {
        Self { title, body: String::new(), _state: PhantomData }
    }

    pub fn set_body(mut self, body: String) -> Self {
        Self { title: self.title, body, _state: PhantomData }
    }

    pub fn publish(self) -> Article<Published> {
        Article { title: self.title, body: self.body, _state: PhantomData }
    }
}

impl Article<Published> {
    pub fn archive(self) -> Article<Archived> {
        Article { title: self.title, body: self.body, _state: PhantomData }
    }
}
// Article<Archived> has no transition methods -- terminal state
```

**ECC codebase usage:** Builder patterns in `ecc-domain` use phantom-type state markers to enforce required-field ordering at compile time. The `ConnectionBuilder` example in `patterns/creational/builder.md` demonstrates `NeedsHost -> NeedsPort -> Ready` typestate progression used across the domain layer.

## When to Use

- When a type has a well-defined state machine with few states.
- When invalid transitions are a source of bugs.
- When you want zero-cost compile-time enforcement.

## When NOT to Use

- When the number of states is large or dynamic (use an enum with runtime checks).
- When states need to be stored heterogeneously in a collection.
- When the added type complexity outweighs the safety benefit.

## Anti-Patterns

- Using typestate for more than ~5 states, creating a combinatorial explosion of impl blocks.
- Storing typestate objects in `Vec<Box<dyn Any>>` and downcasting, negating the benefit.
- Exposing internal state types in the public API when they are an implementation detail.

## Related Patterns

- [builder](../../creational/builder.md) -- typestate builder is the most common application.
- [newtype](newtype.md) -- state marker types are often newtypes.
- [enum-dispatch](enum-dispatch.md) -- runtime alternative when states must be stored dynamically.

## References

- Cliffle -- Rust Typestate Pattern: https://cliffle.com/blog/rust-typestate/
- Ana Hobden -- The Typestate Pattern in Rust: https://hoverbear.org/blog/rust-state-machine-pattern/
