---
name: kotlin-patterns
description: Idiomatic Kotlin patterns including coroutines, Flow, sealed classes, DSL design, and best practices for building robust Kotlin applications.
origin: ECC
---

# Kotlin Development Patterns

Idiomatic Kotlin patterns and best practices for building robust, concise applications.

## When to Activate

- Writing new Kotlin code
- Reviewing Kotlin code
- Designing Kotlin modules or libraries
- Refactoring existing Kotlin code

## Core Principles

### 1. Null Safety

```kotlin
// Good: Safe calls and elvis operator
fun getDisplayName(user: User?): String =
    user?.name?.takeIf { it.isNotBlank() } ?: "Anonymous"

// Good: Smart casting
fun processResult(result: Any) {
    when (result) {
        is Success -> handleSuccess(result.data)
        is Error -> handleError(result.message)
    }
}

// Bad: Non-null assertion — can throw NPE
val name = user!!.name  // Don't do this
```

### 2. Sealed Hierarchies

```kotlin
sealed interface NetworkResult<out T> {
    data class Success<T>(val data: T) : NetworkResult<T>
    data class Error(val code: Int, val message: String) : NetworkResult<Nothing>
    data object Loading : NetworkResult<Nothing>
}

fun <T> NetworkResult<T>.fold(
    onSuccess: (T) -> Unit,
    onError: (Int, String) -> Unit,
    onLoading: () -> Unit = {}
) = when (this) {
    is NetworkResult.Success -> onSuccess(data)
    is NetworkResult.Error -> onError(code, message)
    is NetworkResult.Loading -> onLoading()
}
```

### 3. Coroutines

```kotlin
// Structured concurrency
class UserService(
    private val repository: UserRepository,
    private val dispatcher: CoroutineDispatcher = Dispatchers.IO
) {
    suspend fun getUser(id: String): User = withContext(dispatcher) {
        repository.findById(id) ?: throw NotFoundException("User $id not found")
    }

    suspend fun getUsers(ids: List<String>): List<User> = coroutineScope {
        ids.map { id ->
            async { getUser(id) }
        }.awaitAll()
    }
}
```

### 4. Flow

```kotlin
// Cold stream with operators
fun observeUsers(): Flow<List<User>> = flow {
    while (currentCoroutineContext().isActive) {
        emit(repository.findAll())
        delay(5_000)
    }
}.distinctUntilChanged()
    .catch { e -> emit(emptyList()) }
    .flowOn(Dispatchers.IO)

// StateFlow for UI state
class UserViewModel : ViewModel() {
    private val _state = MutableStateFlow<UiState>(UiState.Loading)
    val state: StateFlow<UiState> = _state.asStateFlow()
}
```

### 5. DSL Design

```kotlin
// Type-safe builder
class HtmlBuilder {
    private val elements = mutableListOf<String>()

    fun div(block: DivBuilder.() -> Unit) {
        elements.add(DivBuilder().apply(block).build())
    }

    fun build(): String = elements.joinToString("\n")
}

fun html(block: HtmlBuilder.() -> Unit): String =
    HtmlBuilder().apply(block).build()

// Usage
val page = html {
    div {
        text("Hello, World!")
    }
}
```

### 6. Extension Functions

```kotlin
// Good: Focused, discoverable extensions
fun <T> List<T>.secondOrNull(): T? = if (size >= 2) this[1] else null

fun String.isValidEmail(): Boolean =
    matches(Regex("^[A-Za-z0-9+_.-]+@[A-Za-z0-9.-]+$"))

// Scope function guidelines
// let: null checks, transformations
// apply: object configuration
// also: side effects (logging, validation)
// run: object configuration + computing result
// with: calling methods on an object
```

## Quick Reference

| Pattern | Description |
|---------|-------------|
| `data class` | Immutable data carrier with auto-generated equals/hashCode/copy |
| `sealed interface` | Restricted type hierarchy for exhaustive `when` |
| `val` over `var` | Immutable by default |
| `?.` / `?:` | Safe call and elvis operator for null handling |
| `coroutineScope` | Structured concurrency boundary |
| `Flow` | Cold asynchronous data stream |
| `suspend` | Coroutine suspension point |
| Extension functions | Add functionality without inheritance |

## Anti-Patterns

```kotlin
// Bad: Using !! everywhere
val name = user!!.name!!.trim()!!

// Bad: Ignoring coroutine cancellation
GlobalScope.launch { /* leaked coroutine */ }

// Bad: Mutable data classes
data class User(var name: String, var age: Int) // Use val

// Bad: Overusing scope functions
user.let { it.also { it.apply { /* nested mess */ } } }
```
