---
paths:
  - "**/*.cs"
  - "**/*.csproj"
  - "**/*.sln"
  - "**/Directory.Build.props"
applies-to: { languages: [csharp] }
---
# C# Testing

> This file extends [common/testing.md](../common/testing.md) with C# specific content.

## Frameworks

- **xUnit** for unit and integration tests (preferred)
- **FluentAssertions** for readable assertions
- **Moq** or **NSubstitute** for mocking
- **Testcontainers** for integration tests

## Running Tests

```bash
dotnet test
dotnet test --collect:"XPlat Code Coverage"
dotnet test --filter "Category=Unit"
```

## Coverage

```bash
dotnet test --collect:"XPlat Code Coverage"
reportgenerator -reports:coverage.cobertura.xml -targetdir:coveragereport
```

## Reference

See skill: `csharp-testing` for detailed C# testing patterns.
