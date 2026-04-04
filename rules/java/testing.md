---
paths:
  - "**/*.java"
  - "**/pom.xml"
  - "**/build.gradle"
  - "**/build.gradle.kts"
applies-to: { languages: [java] }
---
# Java Testing

> This file extends [common/testing.md](../common/testing.md) with Java specific content.

## Frameworks

- **JUnit 5** for unit and integration tests
- **Mockito** for mocking
- **AssertJ** for fluent assertions
- **Testcontainers** for integration tests with real dependencies

## Running Tests

```bash
# Maven
mvn test
mvn verify  # includes integration tests

# Gradle
./gradlew test
./gradlew check
```

## Coverage

```bash
# Maven with JaCoCo
mvn test jacoco:report

# Gradle
./gradlew jacocoTestReport
```

## Reference

See skill: `springboot-tdd` for detailed Java/Spring testing patterns.
