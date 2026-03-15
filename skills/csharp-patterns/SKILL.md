---
name: csharp-patterns
description: Modern C# and .NET patterns including records, async/await, DI, middleware, LINQ, and ASP.NET Core best practices for building robust applications.
origin: ECC
---

# C# Development Patterns

Modern C# and .NET patterns for building robust, maintainable applications.

## When to Activate

- Writing new C# code
- Reviewing C# code
- Designing .NET services or APIs
- Refactoring existing C# code

## Core Principles

### 1. Records for Data

```csharp
// Immutable data carriers
public record UserDto(string Name, string Email, int Age);

// With validation
public record CreateUserRequest(string Name, string Email)
{
    public string Name { get; init; } = Name ?? throw new ArgumentNullException(nameof(Name));
    public string Email { get; init; } = Email ?? throw new ArgumentNullException(nameof(Email));
}

// Non-destructive mutation
var updated = user with { Email = "new@example.com" };
```

### 2. Async/Await

```csharp
// Good: Async all the way
public async Task<User> GetUserAsync(int id, CancellationToken ct = default)
{
    var user = await repository.FindByIdAsync(id, ct)
        ?? throw new NotFoundException($"User {id} not found");
    return user;
}

// Bad: Sync over async
public User GetUser(int id)
{
    return repository.FindByIdAsync(id).Result; // Deadlock risk!
}
```

### 3. Dependency Injection

```csharp
// Primary constructor injection (C# 12)
public class OrderService(
    IOrderRepository orders,
    IPaymentGateway payments,
    ILogger<OrderService> logger)
{
    public async Task<Order> PlaceOrderAsync(CreateOrderRequest request)
    {
        logger.LogInformation("Placing order for {UserId}", request.UserId);
        var order = Order.Create(request);
        await orders.SaveAsync(order);
        await payments.ChargeAsync(order.Total);
        return order;
    }
}

// Registration
builder.Services.AddScoped<IOrderRepository, OrderRepository>();
builder.Services.AddScoped<IPaymentGateway, StripePaymentGateway>();
builder.Services.AddScoped<OrderService>();
```

### 4. Minimal API

```csharp
var app = builder.Build();

app.MapGet("/users/{id}", async (int id, IUserService service) =>
    await service.GetByIdAsync(id) is { } user
        ? Results.Ok(user)
        : Results.NotFound());

app.MapPost("/users", async (CreateUserRequest request, IUserService service) =>
{
    var user = await service.CreateAsync(request);
    return Results.Created($"/users/{user.Id}", user);
});
```

### 5. LINQ Best Practices

```csharp
// Good: Readable, chained
var activeAdmins = users
    .Where(u => u.IsActive)
    .Where(u => u.Role == Role.Admin)
    .OrderBy(u => u.Name)
    .Select(u => new UserSummary(u.Id, u.Name))
    .ToList();

// Bad: Complex single expression
var result = users.Where(u => u.IsActive && u.Role == Role.Admin).OrderBy(u => u.Name).Select(u => new UserSummary(u.Id, u.Name)).ToList();
```

### 6. Error Handling

```csharp
// Custom exception hierarchy
public class DomainException : Exception
{
    public string Code { get; }
    protected DomainException(string code, string message) : base(message) => Code = code;
}

public class NotFoundException : DomainException
{
    public NotFoundException(string resource, object id)
        : base("NOT_FOUND", $"{resource} with ID {id} was not found") { }
}

// Global exception handler middleware
app.UseExceptionHandler(error => error.Run(async context =>
{
    var exception = context.Features.Get<IExceptionHandlerFeature>()?.Error;
    var response = exception switch
    {
        NotFoundException e => (StatusCodes.Status404NotFound, e.Message),
        ValidationException e => (StatusCodes.Status400BadRequest, e.Message),
        _ => (StatusCodes.Status500InternalServerError, "An unexpected error occurred")
    };
    context.Response.StatusCode = response.Item1;
    await context.Response.WriteAsJsonAsync(new { error = response.Message });
}));
```

## Quick Reference

| Pattern | Description |
|---------|-------------|
| Records | Immutable data carriers with value equality |
| Primary constructors | DI via constructor parameters (C# 12) |
| Pattern matching | `is`, `switch` expressions for type-safe branching |
| Minimal APIs | Lightweight HTTP endpoints without controllers |
| `IAsyncEnumerable` | Streaming async data |
| `CancellationToken` | Cooperative cancellation in async methods |
