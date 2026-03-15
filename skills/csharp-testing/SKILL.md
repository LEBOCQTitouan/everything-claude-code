---
name: csharp-testing
description: C# testing patterns using xUnit, FluentAssertions, Moq, and Testcontainers for comprehensive .NET application testing.
origin: ECC
---

# C# Testing Patterns

Testing patterns for .NET applications using xUnit and related tools.

## When to Activate

- Writing tests for C# code
- Setting up test infrastructure for .NET projects
- Debugging failing C# tests
- Adding test coverage to existing code

## xUnit Test Structure

```csharp
public class UserServiceTests
{
    private readonly Mock<IUserRepository> _repository = new();
    private readonly Mock<ILogger<UserService>> _logger = new();
    private readonly UserService _sut;

    public UserServiceTests()
    {
        _sut = new UserService(_repository.Object, _logger.Object);
    }

    [Fact]
    public async Task GetByIdAsync_WhenUserExists_ReturnsUser()
    {
        // Arrange
        var expected = new User(1, "John", "john@example.com");
        _repository.Setup(r => r.FindByIdAsync(1, default))
            .ReturnsAsync(expected);

        // Act
        var result = await _sut.GetByIdAsync(1);

        // Assert
        result.Should().BeEquivalentTo(expected);
    }

    [Fact]
    public async Task GetByIdAsync_WhenUserNotFound_ThrowsNotFoundException()
    {
        // Arrange
        _repository.Setup(r => r.FindByIdAsync(99, default))
            .ReturnsAsync((User?)null);

        // Act
        var act = () => _sut.GetByIdAsync(99);

        // Assert
        await act.Should().ThrowAsync<NotFoundException>()
            .WithMessage("*99*");
    }
}
```

## FluentAssertions

```csharp
// Collections
users.Should().HaveCount(3);
users.Should().Contain(u => u.Name == "John");
users.Should().BeInAscendingOrder(u => u.Name);
users.Should().OnlyContain(u => u.IsActive);

// Objects
result.Should().BeEquivalentTo(expected, options =>
    options.Excluding(u => u.CreatedAt));

// Exceptions
action.Should().Throw<ArgumentNullException>()
    .WithParameterName("userId");
```

## Integration Testing with WebApplicationFactory

```csharp
public class ApiTests : IClassFixture<WebApplicationFactory<Program>>
{
    private readonly HttpClient _client;

    public ApiTests(WebApplicationFactory<Program> factory)
    {
        _client = factory.WithWebHostBuilder(builder =>
        {
            builder.ConfigureServices(services =>
            {
                services.AddScoped<IUserRepository, InMemoryUserRepository>();
            });
        }).CreateClient();
    }

    [Fact]
    public async Task GetUser_ReturnsOk()
    {
        var response = await _client.GetAsync("/users/1");
        response.StatusCode.Should().Be(HttpStatusCode.OK);
    }
}
```

## Theory (Parameterized Tests)

```csharp
[Theory]
[InlineData("", false)]
[InlineData("a", false)]
[InlineData("valid@email.com", true)]
[InlineData("no-at-sign.com", false)]
public void IsValidEmail_ReturnsExpected(string email, bool expected)
{
    EmailValidator.IsValid(email).Should().Be(expected);
}
```

## Running Tests

```bash
dotnet test                                    # Run all tests
dotnet test --filter "Category=Unit"           # Filter by category
dotnet test --collect:"XPlat Code Coverage"    # With coverage
dotnet test --logger "console;verbosity=detailed"  # Verbose output
```

## Quick Reference

| Tool | Purpose |
|------|---------|
| xUnit | Test framework |
| FluentAssertions | Readable assertions |
| Moq / NSubstitute | Mocking |
| Testcontainers | Integration test infrastructure |
| WebApplicationFactory | ASP.NET integration testing |
| Bogus | Test data generation |
