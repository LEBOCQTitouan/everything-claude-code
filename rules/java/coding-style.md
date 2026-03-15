---
paths:
  - "**/*.java"
  - "**/pom.xml"
  - "**/build.gradle"
  - "**/build.gradle.kts"
---
# Java Coding Style

> This file extends [common/coding-style.md](../common/coding-style.md) with Java specific content.

## Formatting

- Use project-level formatter config (`.editorconfig` or IDE settings)
- **google-java-format** or **Spotless** for consistent formatting

## Modern Java (17+)

- Use records for data carriers: `record User(String name, int age) {}`
- Use sealed interfaces for restricted hierarchies
- Use pattern matching: `if (obj instanceof String s) { ... }`
- Use text blocks for multi-line strings
- Prefer `var` for local variables when type is obvious

## Naming

- Classes, interfaces, enums, records: `PascalCase`
- Methods, variables, parameters: `camelCase`
- Constants: `SCREAMING_SNAKE_CASE`
- Packages: `lowercase.dotted`

## Immutability

- Prefer immutable objects — use `final` fields, records, or `@Value` (Lombok)
- Use `List.of()`, `Map.of()`, `Set.of()` for immutable collections
- Return `Collections.unmodifiable*` wrappers when exposing internal collections

## Error Handling

- Use checked exceptions for recoverable conditions
- Use unchecked exceptions for programming errors
- Never catch `Exception` or `Throwable` without re-throwing
- Always include context in exception messages

## Reference

See skill: `java-coding-standards` for comprehensive Java conventions and patterns.
