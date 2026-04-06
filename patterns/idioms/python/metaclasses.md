---
name: metaclasses
category: idioms
tags: [idiom, python]
languages: [python]
difficulty: advanced
---

## Intent

Customize class creation at definition time by intercepting the class construction process, enabling automatic registration, validation of class structure, interface enforcement, or attribute transformation without requiring explicit base class methods.

## Problem

You need to enforce invariants on class definitions (required methods, naming conventions, attribute types) or perform automatic registration of subclasses (plugin systems, serializers). Decorators can modify a class after creation but cannot intercept the creation process itself or affect all subclasses automatically.

## Solution

Define a metaclass by subclassing `type` and overriding `__new__` or `__init_subclass__`. Since Python 3.6, `__init_subclass__` on a base class covers most use cases without a custom metaclass. Reserve full metaclasses for scenarios that `__init_subclass__` cannot handle.

## Language Implementations

### Python

```python
from typing import ClassVar

# Modern approach: __init_subclass__ (Python 3.6+)
class Plugin:
    _registry: ClassVar[dict[str, type]] = {}

    def __init_subclass__(cls, **kwargs):
        super().__init_subclass__(**kwargs)
        name = getattr(cls, "plugin_name", cls.__name__.lower())
        Plugin._registry[name] = cls

class JsonPlugin(Plugin):
    plugin_name = "json"

class YamlPlugin(Plugin):
    plugin_name = "yaml"

# Plugin._registry == {"json": JsonPlugin, "yaml": YamlPlugin}

# Full metaclass: enforce interface at class definition time
class InterfaceMeta(type):
    def __new__(mcs, name: str, bases: tuple, namespace: dict):
        cls = super().__new__(mcs, name, bases, namespace)
        if bases:  # skip the base class itself
            required = {"execute", "validate"}
            missing = required - set(namespace)
            if missing:
                raise TypeError(
                    f"{name} must implement: {', '.join(sorted(missing))}"
                )
        return cls

class Command(metaclass=InterfaceMeta):
    """Base class -- subclasses must define execute() and validate()."""
    pass

# This would raise TypeError at class definition time:
# class BadCommand(Command):
#     def execute(self): ...
#     # missing validate()
```

## When to Use

- For plugin registration systems where subclasses auto-register.
- When enforcing structural contracts on class hierarchies at definition time.
- When `__init_subclass__` is insufficient (need to modify `__new__` behavior).

## When NOT to Use

- For most use cases -- prefer `__init_subclass__`, dataclasses, or decorators first.
- When runtime checks (ABC, Protocol) are sufficient for interface enforcement.
- When the metaclass complexity confuses team members unfamiliar with the pattern.

## Anti-Patterns

- Using metaclasses when a class decorator or `__init_subclass__` would suffice.
- Combining multiple metaclasses (causes metaclass conflict errors).
- Mutating class attributes in metaclass `__new__` in ways that surprise subclass authors.

## Related Patterns

- [decorators](decorators.md) -- class decorators modify classes after creation; simpler for most cases.
- [sealed-classes](../kotlin/sealed-classes.md) -- Kotlin's compile-time restriction on class hierarchies.

## References

- Python docs -- Metaclasses: https://docs.python.org/3/reference/datamodel.html#metaclasses
- PEP 487 -- Simpler customisation of class creation: https://peps.python.org/pep-0487/
- Real Python -- Python Metaclasses: https://realpython.com/python-metaclasses/
