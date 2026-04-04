---
name: abstract-factory
category: creational
tags: [creational, families, polymorphism]
languages: [rust, go, python, typescript]
difficulty: intermediate
---

## Intent

Provide an interface for creating families of related or dependent objects without specifying their concrete classes.

## Problem

A system must be independent of how its products are created, composed, and represented. You need to ensure that a family of related objects is always used together (e.g., UI widgets for a theme) without coupling to specific implementations.

## Solution

Declare an abstract factory interface with creation methods for each product type. Provide concrete factory implementations for each product family. Clients use only the factory interface; swapping the factory changes the entire product family at once.

## Language Implementations

### Rust

```rust
trait Button { fn render(&self); }
trait Checkbox { fn render(&self); }

trait UIFactory {
    fn create_button(&self) -> Box<dyn Button>;
    fn create_checkbox(&self) -> Box<dyn Checkbox>;
}

struct DarkButton;
struct DarkCheckbox;
struct LightButton;
struct LightCheckbox;

impl Button for DarkButton { fn render(&self) { println!("Dark button"); } }
impl Checkbox for DarkCheckbox { fn render(&self) { println!("Dark checkbox"); } }
impl Button for LightButton { fn render(&self) { println!("Light button"); } }
impl Checkbox for LightCheckbox { fn render(&self) { println!("Light checkbox"); } }

struct DarkTheme;
struct LightTheme;

impl UIFactory for DarkTheme {
    fn create_button(&self) -> Box<dyn Button> { Box::new(DarkButton) }
    fn create_checkbox(&self) -> Box<dyn Checkbox> { Box::new(DarkCheckbox) }
}

impl UIFactory for LightTheme {
    fn create_button(&self) -> Box<dyn Button> { Box::new(LightButton) }
    fn create_checkbox(&self) -> Box<dyn Checkbox> { Box::new(LightCheckbox) }
}
```

### Go

```go
type Button interface { Render() }
type Checkbox interface { Render() }

type UIFactory interface {
    CreateButton() Button
    CreateCheckbox() Checkbox
}

type DarkTheme struct{}
func (DarkTheme) CreateButton() Button   { return darkButton{} }
func (DarkTheme) CreateCheckbox() Checkbox { return darkCheckbox{} }

type darkButton struct{}
func (darkButton) Render() { fmt.Println("Dark button") }
type darkCheckbox struct{}
func (darkCheckbox) Render() { fmt.Println("Dark checkbox") }
```

### Python

```python
from abc import ABC, abstractmethod

class Button(ABC):
    @abstractmethod
    def render(self) -> None: ...

class Checkbox(ABC):
    @abstractmethod
    def render(self) -> None: ...

class UIFactory(ABC):
    @abstractmethod
    def create_button(self) -> Button: ...
    @abstractmethod
    def create_checkbox(self) -> Checkbox: ...

class DarkTheme(UIFactory):
    def create_button(self) -> Button: return DarkButton()
    def create_checkbox(self) -> Checkbox: return DarkCheckbox()

class DarkButton(Button):
    def render(self) -> None: print("Dark button")

class DarkCheckbox(Checkbox):
    def render(self) -> None: print("Dark checkbox")
```

### Typescript

```typescript
interface Button { render(): void; }
interface Checkbox { render(): void; }

interface UIFactory {
  createButton(): Button;
  createCheckbox(): Checkbox;
}

class DarkTheme implements UIFactory {
  createButton(): Button { return { render: () => console.log("Dark button") }; }
  createCheckbox(): Checkbox { return { render: () => console.log("Dark checkbox") }; }
}

class LightTheme implements UIFactory {
  createButton(): Button { return { render: () => console.log("Light button") }; }
  createCheckbox(): Checkbox { return { render: () => console.log("Light checkbox") }; }
}
```

## When to Use

- When a system must work with multiple families of related products.
- When you want to enforce that products from the same family are always used together.
- When you want to swap entire product families at runtime or compile time.

## When NOT to Use

- When you only have a single product family — Factory Method is simpler.
- When the family structure changes frequently — each new product requires updating every concrete factory.

## Anti-Patterns

- Embedding product-family selection logic inside client code instead of swapping the factory.
- Creating factories that produce unrelated objects — keep families cohesive.

## Related Patterns

- [factory-method](factory-method.md) — the building block that abstract factories use per product.
- [builder](builder.md) — focuses on constructing a single complex object step by step.
- [prototype](prototype.md) — factories can use prototypes internally to create products.

## References

- Design Patterns: Elements of Reusable Object-Oriented Software (GoF), Chapter 3.
- Refactoring.Guru — Abstract Factory: https://refactoring.guru/design-patterns/abstract-factory
