---
name: factory-method
category: creational
tags: [creational, polymorphism, extensibility]
languages: [rust, go, python, typescript]
difficulty: beginner
---

## Intent

Define an interface for creating an object, but let subclasses or implementations decide which class to instantiate. Factory Method lets a class defer instantiation to subclasses.

## Problem

A class needs to create objects but should not be tightly coupled to the concrete types it creates. The exact type to instantiate may depend on runtime context or configuration, and adding new types should not require modifying existing code.

## Solution

Declare a factory method that returns a common interface or trait. Concrete implementations override or implement this method to return a specific type. Callers use only the abstract interface, remaining unaware of the concrete type.

## Language Implementations

### Rust

```rust
trait Transport {
    fn deliver(&self);
}

struct Truck;
struct Ship;

impl Transport for Truck {
    fn deliver(&self) { println!("Delivering by truck"); }
}

impl Transport for Ship {
    fn deliver(&self) { println!("Delivering by ship"); }
}

fn create_transport(mode: &str) -> Box<dyn Transport> {
    match mode {
        "truck" => Box::new(Truck),
        "ship"  => Box::new(Ship),
        other   => panic!("Unknown transport: {other}"),
    }
}
```

### Go

```go
type Transport interface {
    Deliver()
}

type Truck struct{}
type Ship struct{}

func (t Truck) Deliver() { fmt.Println("Delivering by truck") }
func (s Ship) Deliver()  { fmt.Println("Delivering by ship") }

func CreateTransport(mode string) Transport {
    switch mode {
    case "truck": return Truck{}
    case "ship":  return Ship{}
    default:      panic("unknown transport: " + mode)
    }
}
```

### Python

```python
from abc import ABC, abstractmethod

class Transport(ABC):
    @abstractmethod
    def deliver(self) -> None: ...

class Truck(Transport):
    def deliver(self) -> None:
        print("Delivering by truck")

class Ship(Transport):
    def deliver(self) -> None:
        print("Delivering by ship")

def create_transport(mode: str) -> Transport:
    match mode:
        case "truck": return Truck()
        case "ship":  return Ship()
        case _: raise ValueError(f"Unknown transport: {mode}")
```

### Typescript

```typescript
interface Transport {
  deliver(): void;
}

class Truck implements Transport {
  deliver(): void { console.log("Delivering by truck"); }
}

class Ship implements Transport {
  deliver(): void { console.log("Delivering by ship"); }
}

function createTransport(mode: string): Transport {
  switch (mode) {
    case "truck": return new Truck();
    case "ship":  return new Ship();
    default:      throw new Error(`Unknown transport: ${mode}`);
  }
}
```

## When to Use

- When you cannot anticipate the class of objects to create upfront.
- When you want to provide extension points for subclasses or plugins.
- When the creation logic should be centralised and reused.

## When NOT to Use

- When only one concrete type will ever be needed — a direct constructor call is simpler.
- When the factory logic grows excessively complex; consider Abstract Factory instead.

## Anti-Patterns

- Returning concrete types from the factory instead of the interface — defeats the purpose.
- Putting business logic inside the factory method — keep factories focused on construction.
- Using a factory just to wrap `new` with no added value.

## Related Patterns

- [abstract-factory](abstract-factory.md) — groups related factories together.
- [builder](builder.md) — handles complex construction step by step.
- [prototype](prototype.md) — creates objects by copying existing instances.

## References

- Design Patterns: Elements of Reusable Object-Oriented Software (GoF), Chapter 3.
- Refactoring.Guru — Factory Method: https://refactoring.guru/design-patterns/factory-method
