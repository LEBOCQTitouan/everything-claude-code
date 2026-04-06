---
name: command
category: behavioral
tags: [behavioral, encapsulation, undo, queue]
languages: [rust, go, python, typescript]
difficulty: intermediate
---

## Intent

Encapsulate a request as an object, thereby letting you parameterize clients with different requests, queue or log requests, and support undo operations.

## Problem

You need to issue requests to objects without knowing anything about the operation being requested or the receiver. You also want to support undo, redo, or deferred execution, which requires reifying the request.

## Solution

Create a command interface with an `execute` method. Each concrete command stores a receiver and the parameters needed to invoke it. Clients create command objects and pass them to an invoker that executes, queues, or logs them.

## Language Implementations

### Rust

```rust
trait Command {
    fn execute(&self);
    fn undo(&self);
}

struct PrintCmd { text: String }
impl Command for PrintCmd {
    fn execute(&self) { println!("{}", self.text); }
    fn undo(&self) { println!("undo: {}", self.text); }
}

fn run(cmds: &[&dyn Command]) {
    for cmd in cmds { cmd.execute(); }
}
```

### Go

```go
type Command interface {
    Execute()
    Undo()
}

type PrintCmd struct{ Text string }

func (p PrintCmd) Execute() { fmt.Println(p.Text) }
func (p PrintCmd) Undo()    { fmt.Println("undo:", p.Text) }

func Run(cmds []Command) {
    for _, c := range cmds { c.Execute() }
}
```

### Python

```python
from typing import Protocol

class Command(Protocol):
    def execute(self) -> None: ...
    def undo(self) -> None: ...

class PrintCmd:
    def __init__(self, text: str) -> None:
        self.text = text
    def execute(self) -> None: print(self.text)
    def undo(self) -> None: print(f"undo: {self.text}")
```

### Typescript

```typescript
interface Command {
  execute(): void;
  undo(): void;
}

class PrintCmd implements Command {
  constructor(private text: string) {}
  execute(): void { console.log(this.text); }
  undo(): void { console.log(`undo: ${this.text}`); }
}

function run(cmds: Command[]): void {
  cmds.forEach((c) => c.execute());
}
```

## When to Use

- When you need undo/redo functionality.
- When you need to queue, schedule, or log operations.
- When you want to decouple the invoker from the receiver of a request.

## When NOT to Use

- When operations are simple and do not need undo, queuing, or logging.
- When adding a command layer introduces unnecessary indirection for trivial actions.

## Anti-Patterns

- Commands that silently mutate shared state, making undo unreliable.
- Creating command classes for every trivial operation instead of using closures.
- Commands that depend on the invoker's internal state.

## Related Patterns

- [behavioral/memento](memento.md) -- can store state snapshots for undo alongside commands.
- [behavioral/strategy](strategy.md) -- both encapsulate behavior; command focuses on requests, strategy on algorithms.
- [behavioral/chain-of-responsibility](chain-of-responsibility.md) -- commands can be passed along a chain.

## References

- Design Patterns: Elements of Reusable Object-Oriented Software (GoF), Chapter 5.
- Refactoring.Guru -- Command: https://refactoring.guru/design-patterns/command
