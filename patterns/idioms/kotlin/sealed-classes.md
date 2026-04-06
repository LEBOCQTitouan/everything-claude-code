---
name: sealed-classes
category: idioms
tags: [idiom, kotlin]
languages: [kotlin]
difficulty: intermediate
---

## Intent

Restrict a class hierarchy to a known, finite set of subtypes at compile time, enabling exhaustive `when` expressions that catch missing cases as compiler errors, modeling algebraic data types in Kotlin.

## Problem

Open class hierarchies allow any module to add subtypes, making exhaustive handling impossible. A `when` expression on an open type requires a mandatory `else` branch that silently absorbs new subtypes. The compiler cannot warn when a new variant is added but not handled.

## Solution

Declare a `sealed class` (or `sealed interface`). All direct subtypes must be declared in the same package (same module since Kotlin 1.5). The compiler knows all variants, so `when` expressions without an `else` branch produce compile errors when a variant is missing.

## Language Implementations

### Kotlin

```kotlin
// Sealed class hierarchy -- all subtypes known at compile time
sealed class Result<out T> {
    data class Success<T>(val value: T) : Result<T>()
    data class Failure(val error: AppError) : Result<Nothing>()
    data object Loading : Result<Nothing>()
}

sealed class AppError {
    data class NotFound(val id: String) : AppError()
    data class Unauthorized(val reason: String) : AppError()
    data class Validation(val fields: List<String>) : AppError()
}

// Exhaustive when -- compile error if a variant is missing
fun <T> handleResult(result: Result<T>): String = when (result) {
    is Result.Success -> "Got: ${result.value}"
    is Result.Failure -> handleError(result.error)
    is Result.Loading -> "Loading..."
    // No else needed -- compiler verifies exhaustiveness
}

fun handleError(error: AppError): String = when (error) {
    is AppError.NotFound -> "Not found: ${error.id}"
    is AppError.Unauthorized -> "Access denied: ${error.reason}"
    is AppError.Validation -> "Invalid fields: ${error.fields.joinToString()}"
}

// Sealed interface for multiple inheritance
sealed interface Shape {
    data class Circle(val radius: Double) : Shape
    data class Rectangle(val width: Double, val height: Double) : Shape
}

fun area(shape: Shape): Double = when (shape) {
    is Shape.Circle -> Math.PI * shape.radius * shape.radius
    is Shape.Rectangle -> shape.width * shape.height
}
```

## When to Use

- When modeling domain states, results, errors, or events with a fixed set of variants.
- When exhaustive handling is critical (missing a case should be a compile error).
- When replacing Java-style enum with variants that carry different data.

## When NOT to Use

- When the hierarchy must be extensible by external modules (use interfaces).
- When all variants have the same fields (use an enum class).
- When there are too many variants (>15) making `when` blocks unwieldy.

## Anti-Patterns

- Adding an `else` branch to `when` on sealed types (defeats exhaustiveness).
- Nesting sealed classes too deeply, creating complex type navigation.
- Using sealed classes for a single variant (just use a data class).

## Related Patterns

- [discriminated-unions](../typescript/discriminated-unions.md) -- TypeScript's equivalent using tagged unions.
- [enum-dispatch](../rust/enum-dispatch.md) -- Rust's equivalent using `match` on enum variants.
- [dsl-builders](dsl-builders.md) -- sealed classes define the node types in type-safe DSLs.

## References

- Kotlin docs -- Sealed classes: https://kotlinlang.org/docs/sealed-classes.html
- Kotlin KEEP -- Sealed class improvements: https://github.com/Kotlin/KEEP/blob/master/proposals/sealed-class-inheritance.md
