---
name: composite
category: structural
tags: [structural, tree, recursive, hierarchy]
languages: [rust, go, python, typescript]
difficulty: intermediate
---

## Intent

Compose objects into tree structures to represent part-whole hierarchies. Composite lets clients treat individual objects and compositions of objects uniformly.

## Problem

You have a hierarchy of objects where containers hold other objects (which may themselves be containers). Client code must distinguish between leaf and composite nodes, leading to conditional logic scattered throughout the codebase. You need a uniform interface for both.

## Solution

Define a common trait or interface that both leaf and composite nodes implement. Composite nodes hold a collection of children implementing the same interface and delegate operations recursively. In Rust, use an enum to represent the tree, leveraging pattern matching for traversal.

## Language Implementations

### Rust

Enum-based composite with recursive traversal:

```rust
enum FileNode {
    File { name: String, size: u64 },
    Dir { name: String, children: Vec<FileNode> },
}

impl FileNode {
    fn total_size(&self) -> u64 {
        match self {
            FileNode::File { size, .. } => *size,
            FileNode::Dir { children, .. } => children.iter().map(|c| c.total_size()).sum(),
        }
    }
}
```

### Go

```go
type FileNode interface {
    TotalSize() uint64
}

type File struct{ Name string; Size uint64 }
func (f *File) TotalSize() uint64 { return f.Size }

type Dir struct{ Name string; Children []FileNode }
func (d *Dir) TotalSize() uint64 {
    var total uint64
    for _, c := range d.Children { total += c.TotalSize() }
    return total
}
```

### Python

```python
from dataclasses import dataclass, field

@dataclass
class File:
    name: str
    size: int
    def total_size(self) -> int:
        return self.size

@dataclass
class Dir:
    name: str
    children: list = field(default_factory=list)
    def total_size(self) -> int:
        return sum(c.total_size() for c in self.children)
```

### Typescript

```typescript
interface FileNode {
  totalSize(): number;
}

class File implements FileNode {
  constructor(public name: string, public size: number) {}
  totalSize(): number { return this.size; }
}

class Dir implements FileNode {
  constructor(public name: string, public children: FileNode[] = []) {}
  totalSize(): number { return this.children.reduce((s, c) => s + c.totalSize(), 0); }
}
```

## When to Use

- When you have tree-structured data (file systems, UI components, org charts).
- When clients should treat leaf and composite objects uniformly.
- When operations naturally recurse over the hierarchy.

## When NOT to Use

- When the structure is flat — a simple list is sufficient.
- When leaf and composite operations differ significantly and a uniform interface would be forced.

## Anti-Patterns

- Adding child-management methods to leaf nodes that throw "unsupported operation" — violates LSP.
- Building deeply nested composites without depth limits, risking stack overflow on traversal.
- Using composite when the relationship is not genuinely hierarchical.

## Related Patterns

- [structural/decorator](decorator.md) — often used with composite; decorator wraps a single component while composite aggregates many.
- [behavioral/iterator](../behavioral/iterator.md) — can traverse composite structures.
- [behavioral/visitor](../behavioral/visitor.md) — can execute operations across a composite tree without modifying node classes.

## References

- Design Patterns: Elements of Reusable Object-Oriented Software (GoF), Chapter 4.
- Refactoring.Guru — Composite: https://refactoring.guru/design-patterns/composite
