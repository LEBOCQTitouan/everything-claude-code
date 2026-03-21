# Swift — Project CLAUDE.md

> Swift project template (SPM, iOS/macOS, or server-side).
> Copy this to your project root and customize for your project.

## Project Overview

**Stack:** Swift 6+, Swift Package Manager, XCTest / Swift Testing, SwiftLint

**Architecture:** [Describe your architecture — MVVM, MV, Clean, TCA, etc.]

## Critical Rules

### Swift Conventions

- Enable strict concurrency checking (`SwiftSettings.enableExperimentalFeature("StrictConcurrency")`)
- Prefer `struct` over `class`; use `class` only when reference semantics are required
- Avoid `!` force-unwrap — use `guard let` / `if let` or provide a default
- Use `async/await` for asynchronous code; avoid callback-based APIs in new code
- Mark shared mutable state with `@MainActor` or use actors to prevent data races

### Code Style

- No emojis in code, comments, or documentation
- Immutability first — `let` over `var`
- Keep functions under 40 lines; extract helpers
- `// MARK: -` sections to organize code within files
- SwiftLint enforces style — fix warnings, never silence with `swiftlint:disable`

### Testing

- TDD: Write tests first using Swift Testing (`@Test`, `#expect`)
- 80% minimum coverage
- Unit-test business logic in isolation with protocols/mocks
- UI tests only for critical user flows
- Never make real network calls in unit tests — use `URLProtocol` stubs or dependency injection

### Security

- No hardcoded secrets — load from Keychain or environment
- Validate all user inputs before processing
- Use `SecureField` for sensitive text inputs
- Enable App Transport Security; no plain HTTP in production

## File Structure

### Swift Package (server or library)

```
Sources/
  MyPackage/
    Domain/          # Business types, protocols
    Services/        # Business logic
    Repositories/    # Data access
    Models/          # Data transfer objects
Tests/
  MyPackageTests/
Package.swift
```

### iOS / macOS App

```
MyApp/
  App/               # App entry point, dependency setup
  Features/          # Feature modules (one folder per feature)
    Home/
      HomeView.swift
      HomeViewModel.swift
  Domain/            # Shared business types and protocols
  Services/          # Shared services (networking, persistence)
  Resources/         # Assets, localizations
MyAppTests/
MyAppUITests/
MyApp.xcodeproj
```

## Key Patterns

### Protocol-based Dependency Injection

```swift
protocol UserRepository: Sendable {
    func find(id: UUID) async throws -> User
    func save(_ user: User) async throws
}

actor UserService {
    private let repository: any UserRepository

    init(repository: some UserRepository) {
        self.repository = repository
    }
}
```

### Typed Errors

```swift
enum AppError: Error, LocalizedError {
    case notFound(id: UUID)
    case unauthorized
    case validationFailed(String)

    var errorDescription: String? {
        switch self {
        case .notFound(let id): "Resource \(id) not found"
        case .unauthorized:     "Unauthorized"
        case .validationFailed(let msg): msg
        }
    }
}
```

### Swift Testing

```swift
import Testing

@Suite("UserService")
struct UserServiceTests {
    @Test("creates user with valid input")
    func createUser() async throws {
        let repo = MockUserRepository()
        let sut = UserService(repository: repo)
        let user = try await sut.create(name: "Alice", email: "alice@example.com")
        #expect(user.name == "Alice")
    }

    @Test("throws on duplicate email", arguments: ["taken@example.com"])
    func duplicateEmail(email: String) async throws {
        let repo = MockUserRepository(existingEmails: [email])
        let sut = UserService(repository: repo)
        await #expect(throws: AppError.self) {
            try await sut.create(name: "Bob", email: email)
        }
    }
}
```

## Environment Variables / Configuration

```swift
// Load from environment (server-side) or Info.plist (app)
enum Config {
    static let apiBaseURL: URL = {
        guard let raw = ProcessInfo.processInfo.environment["API_BASE_URL"],
              let url = URL(string: raw) else {
            fatalError("API_BASE_URL missing or invalid")
        }
        return url
    }()
}
```

## Available Commands

```bash
# Build
swift build                     # Debug build
swift build -c release          # Release build

# Test
swift test                      # Run all tests
swift test --filter UserService # Filter by suite/test name
swift test --parallel           # Run in parallel

# Lint
swiftlint lint                  # Check style
swiftlint lint --fix            # Auto-correct fixable violations

# Package management
swift package update            # Update dependencies
swift package resolve           # Resolve without updating
swift package show-dependencies # Show dependency tree
```

## ECC Workflow

```bash
/spec          # Implementation planning (includes TDD workflow)
/verify        # Quality review
/build-fix     # Fix build / compiler errors
```

## Git Workflow

- Conventional commits: `feat:`, `fix:`, `refactor:`, `docs:`, `test:`
- Never commit to main directly
- PRs require review and passing CI
- CI: `swift build`, `swiftlint`, `swift test --parallel`
- Add `*.xcodeproj/xcuserdata/` and `.build/` to `.gitignore`
