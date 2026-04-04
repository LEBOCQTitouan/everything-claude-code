---
paths:
  - "**/*.kt"
  - "**/*.kts"
  - "**/build.gradle.kts"
  - "**/settings.gradle.kts"
applies-to: { languages: [kotlin] }
---
# Kotlin Coding Style

> This file extends [common/coding-style.md](../common/coding-style.md) with Kotlin specific content.

## Formatting

- **ktlint** is mandatory for consistent formatting
- **detekt** for static analysis and code smells

## Idiomatic Kotlin

- Use `data class` for data carriers
- Use `sealed class`/`sealed interface` for restricted hierarchies
- Use `when` expressions instead of `if/else` chains
- Prefer expression bodies for single-expression functions
- Use scope functions appropriately: `let`, `apply`, `also`, `run`, `with`

## Naming

- Classes, interfaces, objects: `PascalCase`
- Functions, properties, variables: `camelCase`
- Constants: `SCREAMING_SNAKE_CASE` or `PascalCase` for object properties
- Packages: `lowercase.dotted`

## Null Safety

- Leverage Kotlin's null safety — avoid `!!` (non-null assertion)
- Use safe calls: `obj?.method()`
- Use `requireNotNull()` or `checkNotNull()` for preconditions
- Prefer `val` over `var` — immutable by default

## Error Handling

- Use `Result<T>` or sealed classes for expected failures
- Use `runCatching { }` for exception-to-result conversion
- Throw exceptions for programming errors only

## Reference

See skill: `kotlin-patterns` for comprehensive Kotlin idioms and patterns.
