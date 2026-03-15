---
paths:
  - "**/*.cs"
  - "**/*.csproj"
  - "**/*.sln"
  - "**/Directory.Build.props"
---
# C# Coding Style

> This file extends [common/coding-style.md](../common/coding-style.md) with C# specific content.

## Formatting

- **dotnet format** is mandatory for consistent formatting
- Use `.editorconfig` for project-level style settings
- Enable nullable reference types: `<Nullable>enable</Nullable>`

## Modern C# (10+)

- Use records for immutable data: `record User(string Name, int Age);`
- Use file-scoped namespaces: `namespace MyApp;`
- Use pattern matching: `if (obj is string { Length: > 0 } s)`
- Use global usings for common namespaces
- Use raw string literals for multi-line text

## Naming

- Types, methods, properties, events: `PascalCase`
- Parameters, local variables: `camelCase`
- Private fields: `_camelCase` (with underscore prefix)
- Constants: `PascalCase`
- Interfaces: `IPascalCase` (with `I` prefix)

## Immutability

- Prefer records over classes for data carriers
- Use `readonly` struct for value types
- Use `init` accessors for immutable properties
- Use `ImmutableArray<T>`, `ImmutableList<T>` from `System.Collections.Immutable`

## Error Handling

- Throw `ArgumentException`, `InvalidOperationException` for programming errors
- Use result patterns for expected failures
- Never catch `Exception` without re-throwing
- Use `ThrowIfNull()` for argument validation

## Reference

See skill: `csharp-patterns` for comprehensive C# patterns and .NET conventions.
