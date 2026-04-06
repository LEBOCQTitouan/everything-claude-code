---
name: memento
category: behavioral
tags: [behavioral, snapshot, undo, history]
languages: [rust, go, python, typescript]
difficulty: intermediate
---

## Intent

Without violating encapsulation, capture and externalize an object's internal state so the object can be restored to that state later.

## Problem

You need undo/redo or checkpointing, but accessing an object's internals to save its state breaks encapsulation. Exposing fields for serialization couples the snapshot format to the object's private structure.

## Solution

The originator creates a memento -- an opaque snapshot of its internal state. A caretaker stores mementos without inspecting them. The originator can restore itself from any memento it previously created.

## Language Implementations

### Rust

```rust
#[derive(Clone)]
struct EditorMemento { content: String, cursor: usize }

struct Editor { content: String, cursor: usize }

impl Editor {
    fn save(&self) -> EditorMemento {
        EditorMemento { content: self.content.clone(), cursor: self.cursor }
    }
    fn restore(&mut self, m: EditorMemento) {
        self.content = m.content;
        self.cursor = m.cursor;
    }
}
```

### Go

```go
type EditorMemento struct{ content string; cursor int }

type Editor struct{ content string; cursor int }

func (e *Editor) Save() EditorMemento {
    return EditorMemento{content: e.content, cursor: e.cursor}
}
func (e *Editor) Restore(m EditorMemento) {
    e.content = m.content; e.cursor = m.cursor
}
```

### Python

```python
from dataclasses import dataclass

@dataclass(frozen=True)
class EditorMemento:
    content: str
    cursor: int

class Editor:
    def __init__(self) -> None:
        self.content = ""
        self.cursor = 0

    def save(self) -> EditorMemento:
        return EditorMemento(content=self.content, cursor=self.cursor)

    def restore(self, m: EditorMemento) -> None:
        self.content = m.content
        self.cursor = m.cursor
```

### Typescript

```typescript
interface EditorMemento {
  readonly content: string;
  readonly cursor: number;
}

class Editor {
  content = "";
  cursor = 0;

  save(): EditorMemento { return { content: this.content, cursor: this.cursor }; }
  restore(m: EditorMemento): void { this.content = m.content; this.cursor = m.cursor; }
}
```

## When to Use

- When you need undo/redo functionality and must preserve encapsulation.
- When you need to create snapshots or checkpoints of an object's state.
- When direct access to internal state would violate the object's boundaries.

## When NOT to Use

- When the object's state is trivially serializable and encapsulation is not a concern.
- When mementos would consume excessive memory (consider incremental snapshots instead).

## Anti-Patterns

- Exposing memento internals to the caretaker, defeating encapsulation.
- Storing too many mementos without a pruning strategy, exhausting memory.
- Using mementos for cross-version persistence when the internal structure may change.

## Related Patterns

- [behavioral/command](command.md) -- commands with undo often use mementos to store pre-execution state.
- [behavioral/state](state.md) -- state represents current behavior; memento captures a snapshot of it.
- [creational/prototype](../creational/prototype.md) -- prototype clones an object; memento captures internal state only.

## References

- Design Patterns: Elements of Reusable Object-Oriented Software (GoF), Chapter 5.
- Refactoring.Guru -- Memento: https://refactoring.guru/design-patterns/memento
