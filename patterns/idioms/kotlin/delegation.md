---
name: delegation
category: idioms
tags: [idiom, kotlin, delegation, by-keyword]
languages: [kotlin]
difficulty: intermediate
---

## Intent

Implement interfaces by delegating method calls to a wrapped object using the `by` keyword, avoiding boilerplate forwarding methods.

## Problem

Implementing the Decorator or Adapter pattern requires manually forwarding every interface method to the wrapped object, producing verbose code that obscures the actual customization.

## Solution

Use Kotlin's `by` keyword in the class declaration to delegate all interface methods to a provided implementation. Override only the methods you want to customize.

## Language Implementations

### Kotlin
```kotlin
interface Logger {
    fun log(msg: String)
    fun error(msg: String)
}

class TimestampLogger(private val inner: Logger) : Logger by inner {
    override fun log(msg: String) = inner.log("[${Instant.now()}] $msg")
    // error() is automatically delegated to inner
}
```

## When to Use

- Decorating or adapting an interface with minimal overrides.
- Property delegation (lazy, observable, map-backed).

## When NOT to Use

- When you need to intercept ALL method calls (use Proxy instead).
- When the delegate lifecycle differs from the wrapper.

## Anti-Patterns

- Delegating to a mutable delegate that changes at runtime.
- Using delegation when composition with explicit forwarding is clearer.

## Related Patterns

- structural/decorator
- structural/proxy

## References

- Kotlin documentation: Delegation.
- Kotlin documentation: Delegated Properties.
