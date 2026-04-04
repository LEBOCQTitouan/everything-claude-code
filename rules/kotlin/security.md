---
paths:
  - "**/*.kt"
  - "**/*.kts"
  - "**/build.gradle.kts"
  - "**/settings.gradle.kts"
applies-to: { languages: [kotlin] }
---
# Kotlin Security

> This file extends [common/security.md](../common/security.md) with Kotlin specific content.

## SQL Injection Prevention

Use parameterized queries (same as Java):

```kotlin
// GOOD — Exposed DSL
Users.select { Users.id eq userId }

// GOOD — JDBC
connection.prepareStatement("SELECT * FROM users WHERE id = ?").apply {
    setString(1, userId)
}

// BAD
connection.createStatement().executeQuery("SELECT * FROM users WHERE id = '$userId'")
```

## Dependency Scanning

```bash
./gradlew dependencyCheckAnalyze
```

## Secrets

```kotlin
val apiKey = System.getenv("API_KEY")
    ?: throw IllegalStateException("API_KEY not configured")
```

## Coroutine Safety

- Use `SupervisorJob` to prevent cascading failures
- Handle exceptions in coroutine scopes with `CoroutineExceptionHandler`
- Never expose `GlobalScope` — use structured concurrency
