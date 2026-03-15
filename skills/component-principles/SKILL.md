---
name: component-principles
description: Evaluate package/module cohesion and coupling using the 6 component principles (REP, CCP, CRP, ADP, SDP, SAP) and main sequence distance formula.
origin: ECC
---

# Component Principles

Methodology for evaluating package/module design using Robert C. Martin's 6 component principles. Measures cohesion, coupling, stability, and abstractness to identify components in the Zone of Pain or Zone of Uselessness.

## When to Activate

- Architecture audits (via `/audit --domain=architecture`)
- Evaluating module boundaries before refactoring
- Reviewing package/crate/module organization
- Assessing dependency graph health
- Planning monorepo or library extraction

## Cohesion Principles (What Goes Together)

### REP — Reuse/Release Equivalence Principle

> The granule of reuse is the granule of release.

Classes and modules grouped into a component should be releasable together. If you can't version and release the component as a coherent unit, it contains unrelated things.

**Detection**:
- Does the component have a single coherent purpose describable in one sentence?
- Would a consumer need all (or most) of the component's exports?
- Are there exports that change for completely different reasons?

**Violation signals**:
- Component contains utilities unrelated to its core domain — HIGH
- Consumer imports one function but gets 50 transitive dependencies — HIGH

### CCP — Common Closure Principle

> Classes that change together belong together.

Group into the same component things that change for the same reason at the same time. This is SRP applied at the component level.

**Detection**:
- Analyze git co-change history: files that change together in >60% of commits belong together
- Check if a single requirement change touches multiple components (shotgun surgery)

**Violation signals**:
- Single feature change requires modifying 3+ components — HIGH
- Files in the same component never change together (< 10% co-change) — MEDIUM

### CRP — Common Reuse Principle

> Don't force users of a component to depend on things they don't need.

Classes in a component should be tightly coupled — if you use one, you use (or at least accept the dependency on) all. This is ISP applied at the component level.

**Detection**:
- For each consumer of a component, count how many exports it actually uses vs total exports
- If consumers typically use < 30% of exports, the component should be split

**Violation signals**:
- Average consumer uses < 25% of component exports — HIGH
- Component has distinct clusters of exports with no cross-usage — MEDIUM

## Coupling Principles (How Components Relate)

### ADP — Acyclic Dependencies Principle

> The dependency graph of components must have no cycles.

Cycles create ripple effects where a change in any component in the cycle can force recompilation/retesting of all others.

**Detection**:
- Build component dependency graph from imports
- Run cycle detection (DFS with back-edge detection)
- For each cycle: list the components involved and the specific imports creating the cycle

**Violation signals**:
- Any cycle in the component dependency graph — CRITICAL
- Cycle involving 3+ components — CRITICAL (harder to break)

**Breaking cycles**:
1. **Dependency Inversion**: introduce an interface in the depended-upon component
2. **New component**: extract the shared dependency into a new component both depend on
3. **Merge**: if two components are always co-dependent, merge them

### SDP — Stable Dependencies Principle

> Depend in the direction of stability.

A component that is hard to change (stable) should not depend on a component that is easy to change (unstable).

**Instability metric**:
```
I = Ce / (Ca + Ce)
```
- `Ca` = afferent coupling (incoming dependencies — who depends on me)
- `Ce` = efferent coupling (outgoing dependencies — who I depend on)
- `I = 0` → maximally stable (many dependents, no dependencies)
- `I = 1` → maximally unstable (no dependents, many dependencies)

**Detection**:
- Calculate I for each component
- For each dependency edge A → B: check that `I(A) >= I(B)` (depend toward stability)

**Violation signals**:
- Stable component (I < 0.3) depends on unstable component (I > 0.7) — HIGH
- Any dependency pointing toward higher instability — MEDIUM

### SAP — Stable Abstractions Principle

> A component should be as abstract as it is stable.

Stable components should be abstract (interfaces, traits, abstract classes) so they can be extended without modification. Unstable components should be concrete.

**Abstractness metric**:
```
A = Na / Nc
```
- `Na` = number of abstract types (interfaces, traits, abstract classes, type aliases)
- `Nc` = total number of types in the component
- `A = 0` → fully concrete
- `A = 1` → fully abstract

## Main Sequence Distance

The relationship between stability and abstractness should follow the **main sequence** line: `A + I = 1`.

```
D = |A + I - 1|
```

- `D = 0` → on the main sequence (ideal)
- `D > 0` → deviation from ideal

### Zone of Pain (low A, low I)

Concrete and stable. Many things depend on it, but it has no abstractions. Every change is painful because it forces changes in all dependents.

- `A < 0.3` AND `I < 0.3` AND `D > 0.3` — HIGH
- Examples: concrete utility libraries with many consumers, shared config objects

**Remediation**: introduce abstractions (interfaces/traits) to allow extension without modification.

### Zone of Uselessness (high A, high I)

Abstract and unstable. Interfaces nobody implements. Abstractions with no dependents.

- `A > 0.7` AND `I > 0.7` AND `D > 0.3` — MEDIUM
- Examples: unused interface hierarchies, over-engineered abstractions

**Remediation**: remove unused abstractions or find concrete implementations.

## Main Sequence Distance Chart

For each top-level component, compute and display:

```
Component          | Ca  | Ce  | I    | Na | Nc | A    | D
-------------------|-----|-----|------|----|----|------|------
ecc-domain         |  4  |  0  | 0.00 | 12 | 20 | 0.60 | 0.40
ecc-ports          |  3  |  1  | 0.25 | 8  | 10 | 0.80 | 0.05
ecc-app            |  2  |  3  | 0.60 | 2  | 15 | 0.13 | 0.27
ecc-infra          |  1  |  2  | 0.67 | 0  | 8  | 0.00 | 0.33
ecc-cli            |  0  |  3  | 1.00 | 0  | 5  | 0.00 | 0.00
```

Flag any component with `D > 0.3` and explain which zone it occupies.

## Finding Format

```
### [COMP-NNN] Finding Title
- **Severity**: CRITICAL | HIGH | MEDIUM | LOW
- **Component**: component name
- **Principle**: REP | CCP | CRP | ADP | SDP | SAP
- **Metrics**: Ca=X, Ce=Y, I=Z, A=W, D=V (where applicable)
- **Evidence**: Concrete data (dependency edges, co-change stats, export usage)
- **Risk**: What breaks if unaddressed
- **Remediation**: Directional fix (what, not how)
```

## Related

- Agent: `agents/component-auditor.md`
- Command: `commands/audit.md`
- Complementary: `skills/architecture-review/SKILL.md`
