---
name: kotlin-testing
description: Kotlin testing patterns using Kotest, MockK, coroutine testing with runTest, and Testcontainers for comprehensive Kotlin application testing.
origin: ECC
---

# Kotlin Testing Patterns

Testing patterns for Kotlin applications using Kotest, MockK, and coroutine testing.

## When to Activate

- Writing tests for Kotlin code
- Setting up test infrastructure for Kotlin/JVM projects
- Testing coroutines and Flow
- Adding coverage to Kotlin code

## Kotest

### Spec Styles

```kotlin
class UserServiceTest : FunSpec({
    val repository = mockk<UserRepository>()
    val service = UserService(repository)

    test("getUser returns user when found") {
        coEvery { repository.findById("1") } returns User("1", "John")

        val result = service.getUser("1")

        result.name shouldBe "John"
    }

    test("getUser throws when not found") {
        coEvery { repository.findById("99") } returns null

        shouldThrow<NotFoundException> {
            service.getUser("99")
        }.message shouldContain "99"
    }
})
```

### Property-Based Testing

```kotlin
class StringUtilsTest : FunSpec({
    test("slug is always lowercase with hyphens") {
        checkAll(Arb.string(1..100)) { input ->
            val slug = input.toSlug()
            slug shouldMatch Regex("^[a-z0-9-]*$")
        }
    }
})
```

## MockK

```kotlin
// Mocking
val repo = mockk<UserRepository>()
every { repo.findById("1") } returns User("1", "John")
coEvery { repo.saveAsync(any()) } just Runs

// Verification
verify(exactly = 1) { repo.findById("1") }
coVerify { repo.saveAsync(match { it.name == "John" }) }

// Relaxed mocks
val logger = mockk<Logger>(relaxed = true)
```

## Coroutine Testing

```kotlin
class UserServiceTest : FunSpec({
    test("concurrent user fetch") {
        runTest {
            val service = UserService(FakeRepository(), testDispatcher)

            val users = service.getUsers(listOf("1", "2", "3"))

            users shouldHaveSize 3
        }
    }

    test("flow emits updates") {
        runTest {
            val flow = service.observeUsers()

            flow.take(3).toList() shouldHaveSize 3
        }
    }
})
```

## Running Tests

```bash
./gradlew test                        # Run all tests
./gradlew test --tests "*.UserService*"  # Filter
./gradlew koverReport                 # Coverage report
```

## Quick Reference

| Tool | Purpose |
|------|---------|
| Kotest | Kotlin-native test framework |
| MockK | Kotlin-native mocking |
| `runTest` | Coroutine test runner |
| Kover | Kotlin code coverage |
| Testcontainers | Integration test infrastructure |
| Kotest property | Property-based testing |
