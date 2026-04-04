---
paths:
  - "**/*.kt"
  - "**/*.kts"
  - "**/build.gradle.kts"
  - "**/settings.gradle.kts"
applies-to: { languages: [kotlin] }
---
# Kotlin Patterns

> This file extends [common/patterns.md](../common/patterns.md) with Kotlin specific content.

## Sealed Classes for State

```kotlin
sealed interface Result<out T> {
    data class Success<T>(val value: T) : Result<T>
    data class Failure(val error: Throwable) : Result<Nothing>
}

fun <T> Result<T>.getOrThrow(): T = when (this) {
    is Result.Success -> value
    is Result.Failure -> throw error
}
```

## Extension Functions

```kotlin
fun String.toSlug(): String =
    lowercase().replace(Regex("[^a-z0-9]+"), "-").trim('-')
```

## Dependency Injection (Koin/Hilt)

```kotlin
// Koin
val appModule = module {
    single<UserRepository> { UserRepositoryImpl(get()) }
    factory { UserService(get()) }
}
```

## Reference

See skill: `kotlin-patterns` for comprehensive Kotlin patterns including coroutines, Flow, and DSL design.
