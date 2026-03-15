---
paths:
  - "**/*.cs"
  - "**/*.csproj"
  - "**/*.sln"
  - "**/Directory.Build.props"
---
# C# Security

> This file extends [common/security.md](../common/security.md) with C# specific content.

## SQL Injection Prevention

Always use parameterized queries:

```csharp
// GOOD
await connection.ExecuteAsync(
    "SELECT * FROM Users WHERE Id = @Id",
    new { Id = userId });

// BAD
var sql = $"SELECT * FROM Users WHERE Id = '{userId}'";
```

## Dependency Scanning

```bash
dotnet list package --vulnerable
dotnet list package --deprecated
```

## Secrets

```csharp
// Use configuration and secret management
var apiKey = configuration["ApiKey"]
    ?? throw new InvalidOperationException("ApiKey not configured");

// Never hardcode
// var apiKey = "sk-1234..."; // BAD
```

## ASP.NET Security

- Enable HTTPS redirection: `app.UseHttpsRedirection()`
- Configure CORS properly — never use `AllowAnyOrigin()` with credentials
- Use `[Authorize]` attribute on all non-public endpoints
- Validate all model binding with `[Required]`, `[Range]`, etc.
