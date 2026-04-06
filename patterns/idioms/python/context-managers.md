---
name: context-managers
category: idioms
tags: [idiom, python]
languages: [python]
difficulty: beginner
---

## Intent

Guarantee deterministic resource acquisition and release using the `with` statement, ensuring cleanup happens even when exceptions occur, without manual `try/finally` blocks.

## Problem

Resources like files, database connections, locks, and temporary directories require explicit cleanup. `try/finally` blocks are verbose and easy to forget, especially when managing multiple resources. Nested `try/finally` for multiple resources becomes deeply indented.

## Solution

Implement the context manager protocol (`__enter__` and `__exit__`) on a class, or use `contextlib.contextmanager` to write a generator-based context manager. The `with` statement calls `__enter__` on entry and `__exit__` on exit (including exceptions).

## Language Implementations

### Python

```python
from contextlib import contextmanager
import tempfile
import os

# Class-based context manager
class TempDirectory:
    def __enter__(self) -> str:
        self.path = tempfile.mkdtemp()
        return self.path

    def __exit__(self, exc_type, exc_val, exc_tb) -> bool:
        import shutil
        shutil.rmtree(self.path, ignore_errors=True)
        return False  # do not suppress exceptions

# Generator-based context manager (simpler for most cases)
@contextmanager
def database_transaction(conn):
    cursor = conn.cursor()
    try:
        yield cursor
        conn.commit()
    except Exception:
        conn.rollback()
        raise
    finally:
        cursor.close()

# Usage
with TempDirectory() as tmpdir:
    path = os.path.join(tmpdir, "data.txt")
    with open(path, "w") as f:
        f.write("content")
# tmpdir is cleaned up here, even on exception

# Multiple context managers (Python 3.10+)
with (
    open("input.txt") as src,
    open("output.txt", "w") as dst,
):
    dst.write(src.read())
```

## When to Use

- For any resource that must be released: files, connections, locks, temp dirs.
- When setup/teardown logic is paired and must not be separated.
- When managing multiple resources that depend on each other.

## When NOT to Use

- When the resource has no meaningful cleanup (plain data objects).
- When the scope of the resource extends beyond a single code block.
- When async cleanup is needed (use `async with` and `__aenter__`/`__aexit__`).

## Anti-Patterns

- Returning `True` from `__exit__` to silently suppress all exceptions.
- Performing heavy computation in `__exit__` that can itself fail.
- Using a context manager when a simple `try/finally` is clearer for one-off cases.

## Related Patterns

- [raii](../rust/raii.md) -- Rust's Drop trait is the equivalent deterministic cleanup mechanism.
- [decorators](decorators.md) -- `contextlib.contextmanager` uses a decorator to create context managers.
- [generators](generators.md) -- generator-based context managers use `yield` to split enter/exit.

## References

- Python docs -- Context Manager Types: https://docs.python.org/3/reference/datamodel.html#context-managers
- PEP 343 -- The "with" Statement: https://peps.python.org/pep-0343/
- contextlib module: https://docs.python.org/3/library/contextlib.html
