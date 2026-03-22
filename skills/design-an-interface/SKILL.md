---
name: design-an-interface
description: Interface design methodology for divergent exploration using parallel design alternatives and comparative synthesis.
origin: ECC
---

# Design an Interface

Methodology for exploring multiple radically different interface designs for a module or port, based on John Ousterhout's "Design It Twice" principle.

## When to Activate

Trigger phrases: "design an interface", "design it twice", "explore interface options", "compare API shapes", "what should the port look like"

## Methodology

Spawn parallel sub-agents, each constrained to a radically different design philosophy. The `interface-designer` agent handles orchestration.

### Default Constraints (4 mandatory)

1. **minimize method count** — aim for 1-3 methods max
2. **maximize flexibility** — support many use cases
3. **optimize for the most common case** — make the 80% path trivial
4. **named paradigm** — agent chooses a radically different paradigm (Actor model, Builder pattern, Monad, etc.) for maximum divergence

An optional 5th constraint can be user-supplied.

### Evaluation Dimensions (5)

Compare designs on:
1. **Interface simplicity** — fewer concepts to learn
2. **General-purpose vs specialized** — breadth of applicability
3. **Implementation efficiency** — cost of implementing the interface
4. **Depth** — small interface hiding significant complexity = good
5. **Ease of correct use vs ease of misuse** — pit of success

### Output Format

Each sub-agent produces: interface signature (in detected language), usage example, what it hides internally, and tradeoffs.

## Anti-Patterns

- **DO NOT let sub-agents produce similar designs** — enforce radical difference via divergence review
- **DO NOT skip comparison** — the value is in the contrast
- **DO NOT implement anything** — this is purely about interface shape

## Output

Results are persisted to `docs/designs/{module}-interface-{date}.md`.
