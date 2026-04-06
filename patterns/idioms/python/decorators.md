---
name: decorators
category: idioms
tags: [idiom, python]
languages: [python]
difficulty: intermediate
---

## Intent

Wrap a function or class with additional behavior using the `@decorator` syntax, applying cross-cutting concerns (logging, caching, validation, retry) without modifying the original function body.

## Problem

Cross-cutting concerns like timing, logging, authentication, and caching repeat across many functions. Inlining this logic pollutes business code. Manual wrapping requires boilerplate that obscures the relationship between the wrapper and the wrapped function.

## Solution

Define a higher-order function that accepts a function and returns a new function with augmented behavior. Apply it with the `@` syntax above the function definition. Use `functools.wraps` to preserve the original function's metadata.

## Language Implementations

### Python

```python
import functools
import time
from typing import Callable, TypeVar, ParamSpec

P = ParamSpec("P")
R = TypeVar("R")

def timed(func: Callable[P, R]) -> Callable[P, R]:
    """Log execution time of the decorated function."""
    @functools.wraps(func)
    def wrapper(*args: P.args, **kwargs: P.kwargs) -> R:
        start = time.perf_counter()
        result = func(*args, **kwargs)
        elapsed = time.perf_counter() - start
        print(f"{func.__name__} took {elapsed:.4f}s")
        return result
    return wrapper

# Decorator with arguments
def retry(max_attempts: int = 3, delay: float = 1.0):
    def decorator(func: Callable[P, R]) -> Callable[P, R]:
        @functools.wraps(func)
        def wrapper(*args: P.args, **kwargs: P.kwargs) -> R:
            last_err: Exception | None = None
            for attempt in range(max_attempts):
                try:
                    return func(*args, **kwargs)
                except Exception as e:
                    last_err = e
                    time.sleep(delay)
            raise last_err  # type: ignore[misc]
        return wrapper
    return decorator

@timed
@retry(max_attempts=3, delay=0.5)
def fetch_data(url: str) -> dict:
    ...
```

## When to Use

- For cross-cutting concerns: logging, timing, caching, retry, auth checks.
- When the same wrapping logic applies to many functions.
- When you want the modification to be visible at the function definition site.

## When NOT to Use

- When the wrapping logic is unique to one function (just inline it).
- When decorator stacking order creates confusing behavior.
- When debugging difficulty from wrapped stack traces outweighs the convenience.

## Anti-Patterns

- Forgetting `@functools.wraps`, losing `__name__`, `__doc__`, and `__module__`.
- Stacking too many decorators, making execution order hard to follow.
- Using decorators for control flow that belongs in the function body.

## Related Patterns

- [context-managers](context-managers.md) -- scoped resource management, often combined with decorators.
- [decorator](../../structural/decorator.md) -- the OOP structural pattern; Python decorators are a language-level implementation.

## References

- Python docs -- Decorators: https://docs.python.org/3/glossary.html#term-decorator
- PEP 318 -- Decorators for Functions and Methods: https://peps.python.org/pep-0318/
- Real Python -- Primer on Decorators: https://realpython.com/primer-on-python-decorators/
