---
paths:
  - "**/*.cs"
  - "**/*.csproj"
  - "**/*.sln"
  - "**/Directory.Build.props"
---
# C# Patterns

> This file extends [common/patterns.md](../common/patterns.md) with C# specific content.

## Dependency Injection

```csharp
// Register services in Program.cs
builder.Services.AddScoped<IUserRepository, UserRepository>();
builder.Services.AddScoped<IUserService, UserService>();

// Constructor injection
public class UserService(IUserRepository repository, ILogger<UserService> logger)
{
    public async Task<User?> GetByIdAsync(int id) =>
        await repository.FindByIdAsync(id);
}
```

## Result Pattern

```csharp
public record Result<T>
{
    public T? Value { get; init; }
    public string? Error { get; init; }
    public bool IsSuccess => Error is null;

    public static Result<T> Success(T value) => new() { Value = value };
    public static Result<T> Failure(string error) => new() { Error = error };
}
```

## Repository Pattern

```csharp
public interface IUserRepository
{
    Task<User?> FindByIdAsync(int id);
    Task<IReadOnlyList<User>> FindAllAsync();
    Task<User> CreateAsync(User user);
}
```

## Reference

See skill: `csharp-patterns` for comprehensive C# patterns including async/await, LINQ, and middleware.
