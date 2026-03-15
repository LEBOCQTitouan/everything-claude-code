---
paths:
  - "**/*.kt"
  - "**/*.kts"
  - "**/build.gradle.kts"
  - "**/settings.gradle.kts"
---
# Kotlin Testing

> This file extends [common/testing.md](../common/testing.md) with Kotlin specific content.

## Frameworks

- **Kotest** for idiomatic Kotlin testing (preferred)
- **JUnit 5** with Kotlin extensions
- **MockK** for mocking (Kotlin-native)
- **Testcontainers** for integration tests

## Running Tests

```bash
./gradlew test
./gradlew check  # includes detekt + ktlint
```

## Coverage

```bash
./gradlew koverReport
./gradlew jacocoTestReport
```

## Reference

See skill: `kotlin-testing` for detailed Kotlin testing patterns.
