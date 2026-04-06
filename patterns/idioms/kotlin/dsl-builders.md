---
name: dsl-builders
category: idioms
tags: [idiom, kotlin]
languages: [kotlin]
difficulty: advanced
---

## Intent

Create type-safe, declarative domain-specific languages using Kotlin's receiver lambdas and `@DslMarker` annotations, enabling readable configuration that is validated at compile time.

## Problem

Configuration via raw maps, YAML, or fluent builder chains lacks type safety and IDE support. Errors surface at runtime. XML/JSON configs require separate parsing and validation. Developers want to express hierarchical configuration in code with full type checking and autocomplete.

## Solution

Define builder classes with methods that configure properties. Accept `T.() -> Unit` lambdas (receiver lambdas) to scope builder methods. Use `@DslMarker` to prevent accidental access to outer scope receivers, keeping the DSL intuitive.

## Language Implementations

### Kotlin

```kotlin
@DslMarker
annotation class HtmlDsl

@HtmlDsl
class HtmlBuilder {
    private val elements = mutableListOf<String>()

    fun head(block: HeadBuilder.() -> Unit) {
        val builder = HeadBuilder().apply(block)
        elements.add("<head>${builder.build()}</head>")
    }

    fun body(block: BodyBuilder.() -> Unit) {
        val builder = BodyBuilder().apply(block)
        elements.add("<body>${builder.build()}</body>")
    }

    fun build(): String = "<html>${elements.joinToString("")}</html>"
}

@HtmlDsl
class HeadBuilder {
    private var titleText = ""
    fun title(text: String) { titleText = text }
    fun build(): String = "<title>$titleText</title>"
}

@HtmlDsl
class BodyBuilder {
    private val elements = mutableListOf<String>()
    fun h1(text: String) { elements.add("<h1>$text</h1>") }
    fun p(text: String) { elements.add("<p>$text</p>") }
    fun build(): String = elements.joinToString("")
}

fun html(block: HtmlBuilder.() -> Unit): String =
    HtmlBuilder().apply(block).build()

// Usage -- reads like a declarative spec
val page = html {
    head { title("My Page") }
    body {
        h1("Welcome")
        p("Hello, world!")
        // title("wrong") -- compile error: @DslMarker prevents this
    }
}

// Route DSL example (Ktor-style)
fun routing(block: RouteBuilder.() -> Unit) =
    RouteBuilder().apply(block)

class RouteBuilder {
    fun get(path: String, handler: () -> String) { /* ... */ }
    fun post(path: String, handler: () -> String) { /* ... */ }
}
```

## When to Use

- When building configuration, routing, UI layout, or query DSLs.
- When hierarchical structure benefits from a declarative syntax.
- When IDE autocomplete and compile-time validation are valuable.

## When NOT to Use

- When a simple builder pattern or data class is sufficient.
- When the DSL would only be used in 1-2 places (not worth the abstraction).
- When non-Kotlin consumers need to call the API (DSLs are Kotlin-specific).

## Anti-Patterns

- Omitting `@DslMarker`, allowing accidental access to outer scope receivers.
- Making DSL builders mutable after construction (build should be terminal).
- Creating overly nested DSLs that are harder to read than the problem they solve.

## Related Patterns

- [builder](../../creational/builder.md) -- DSL builders extend the builder pattern with receiver lambdas.
- [sealed-classes](sealed-classes.md) -- sealed classes define the node types in DSL ASTs.
- [coroutine-scope](coroutine-scope.md) -- `coroutineScope { }` is a DSL for structured concurrency.

## References

- Kotlin docs -- Type-safe builders: https://kotlinlang.org/docs/type-safe-builders.html
- Kotlin docs -- DslMarker: https://kotlinlang.org/api/latest/jvm/stdlib/kotlin/-dsl-marker/
