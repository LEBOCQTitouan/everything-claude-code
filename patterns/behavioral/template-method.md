---
name: template-method
category: behavioral
tags: [behavioral, inheritance, hook, algorithm-skeleton]
languages: [rust, go, python, typescript]
difficulty: intermediate
---

## Intent

Define the skeleton of an algorithm in a base operation, deferring some steps to subclasses or trait implementors so they can override specific steps without changing the overall structure.

## Problem

Several classes implement similar algorithms with only a few varying steps. Duplicating the entire algorithm in each class leads to code duplication and inconsistent behavior when the shared steps change.

## Solution

Place the invariant algorithm skeleton in a base type and declare the varying steps as abstract methods. Subclasses (or trait implementors in Rust) provide concrete implementations of only the varying steps.

## Language Implementations

### Rust

Rust uses a trait with provided methods for the skeleton and required methods for the varying steps:

```rust
trait DataExporter {
    fn fetch_rows(&self) -> Vec<String>;        // required
    fn format_row(&self, row: &str) -> String;  // required

    fn export(&self) -> String {                 // template method
        self.fetch_rows().iter().map(|r| self.format_row(r)).collect::<Vec<_>>().join("\n")
    }
}

struct CsvExporter;
impl DataExporter for CsvExporter {
    fn fetch_rows(&self) -> Vec<String> { vec!["a,b".into()] }
    fn format_row(&self, row: &str) -> String { row.to_string() }
}
```

### Go

```go
type RowFetcher interface {
    FetchRows() []string
    FormatRow(row string) string
}

func Export(rf RowFetcher) string {
    rows := rf.FetchRows()
    var out []string
    for _, r := range rows {
        out = append(out, rf.FormatRow(r))
    }
    return strings.Join(out, "\n")
}
```

### Python

```python
from abc import ABC, abstractmethod

class DataExporter(ABC):
    @abstractmethod
    def fetch_rows(self) -> list[str]: ...

    @abstractmethod
    def format_row(self, row: str) -> str: ...

    def export(self) -> str:  # template method
        return "\n".join(self.format_row(r) for r in self.fetch_rows())
```

### Typescript

```typescript
abstract class DataExporter {
  protected abstract fetchRows(): string[];
  protected abstract formatRow(row: string): string;

  export(): string {  // template method
    return this.fetchRows().map((r) => this.formatRow(r)).join("\n");
  }
}
```

## When to Use

- When several classes share an algorithm skeleton but differ in specific steps.
- When you want to enforce an invariant sequence of operations while allowing customization.
- When hook methods provide optional extension points in a fixed workflow.

## When NOT to Use

- When the algorithm has no invariant structure -- use [behavioral/strategy](strategy.md) instead.
- When there are more than two or three varying steps -- composition (strategy) scales better.

## Anti-Patterns

- Overriding the template method itself, defeating the invariant skeleton.
- Creating deep inheritance hierarchies to vary tiny steps.
- Using template method when a simple callback or strategy injection is clearer.

## Related Patterns

- [behavioral/strategy](strategy.md) -- composition-based alternative; prefer strategy when steps are independent.
- [creational/factory-method](../creational/factory-method.md) -- a factory method is often a step inside a template method.
- [behavioral/observer](observer.md) -- hook methods resemble observer callbacks.

## References

- Design Patterns: Elements of Reusable Object-Oriented Software (GoF), Chapter 5.
- Refactoring.Guru -- Template Method: https://refactoring.guru/design-patterns/template-method
