---
name: generators
category: idioms
tags: [idiom, python]
languages: [python]
difficulty: intermediate
---

## Intent

Produce a sequence of values lazily using `yield`, computing each element on demand rather than materializing the entire collection in memory, enabling processing of arbitrarily large or infinite data streams.

## Problem

Building a full list in memory for large datasets wastes RAM and delays the first result until all items are computed. Processing a 10GB file line by line should not require 10GB of memory. Infinite sequences (sensor streams, paginated APIs) cannot be represented as finite lists.

## Solution

Define a function with `yield` instead of `return`. Each call to `next()` resumes the function from where it last yielded. Generator expressions provide a compact syntax for simple cases. Generators implement the iterator protocol and compose with `for` loops, `itertools`, and other generators.

## Language Implementations

### Python

```python
from typing import Generator, Iterator
from pathlib import Path

# Generator function
def read_large_file(path: Path) -> Generator[str, None, None]:
    with open(path) as f:
        for line in f:
            yield line.strip()

# Generator expression
squares = (x * x for x in range(1_000_000))

# Composing generators -- pipeline pattern
def parse_records(lines: Iterator[str]) -> Generator[dict, None, None]:
    for line in lines:
        if line and not line.startswith("#"):
            key, _, value = line.partition("=")
            yield {"key": key.strip(), "value": value.strip()}

def filter_valid(records: Iterator[dict]) -> Generator[dict, None, None]:
    for record in records:
        if record["key"] and record["value"]:
            yield record

# Lazy pipeline -- processes one line at a time
lines = read_large_file(Path("config.txt"))
records = parse_records(lines)
valid = filter_valid(records)
for record in valid:
    print(record)

# yield from -- delegate to sub-generator
def flatten(nested: list[list[int]]) -> Generator[int, None, None]:
    for inner in nested:
        yield from inner
```

## When to Use

- When processing large datasets that should not fit entirely in memory.
- When building data pipelines that compose multiple transformation stages.
- When implementing infinite or on-demand sequences.

## When NOT to Use

- When you need random access to elements (generators are forward-only).
- When the full collection is needed multiple times (generators are single-use).
- When the sequence is small and a list comprehension is clearer.

## Anti-Patterns

- Calling `list()` on a generator meant for streaming, negating the memory benefit.
- Using generators for side-effect-only operations (prefer explicit loops).
- Ignoring that generators are single-use -- iterating twice silently yields nothing.

## Related Patterns

- [context-managers](context-managers.md) -- `contextlib.contextmanager` uses generators for enter/exit.
- [decorators](decorators.md) -- generators can be decorated for logging, timing, or buffering.

## References

- Python docs -- Generator expressions: https://docs.python.org/3/reference/expressions.html#generator-expressions
- PEP 255 -- Simple Generators: https://peps.python.org/pep-0255/
- PEP 380 -- Syntax for Delegating to a Subgenerator: https://peps.python.org/pep-0380/
