# Agent Orchestration

## Full Development Flow

```mermaid
flowchart TD
    USER(["User Request"])

    USER --> PLANNER["planner\nBreaks task into phases\nidentifies risks"]

    PLANNER --> ARCH["architect\nDefines hexagonal structure\nBounded contexts, ports, aggregates\nDDD model"]

    ARCH --> ARCHMOD["architect-module\nDesigns module internals\nPattern selection\nCode organization"]

    ARCHMOD --> UB1["uncle-bob\nPre-implementation review\nSOLID + Clean Architecture\nDependency rule audit"]

    UB1 -->|"SOLID violations\nClean Code prescriptions"| ARCHMOD
    UB1 -->|"Layer violations\nPort contract issues"| ARCH

    UB1 -->|"Design approved"| CODE["Code written\nby Claude Code"]

    CODE --> CR["code-reviewer\nSecurity, quality\nbest practices"]
    CODE --> UB2["uncle-bob\nPost-implementation\nNaming, functions, tests\nClean Code audit"]

    CR --> REPORT["Merged Review Report\n[Security] findings\n[Clean Code] findings\n[Clean Architecture] findings"]
    UB2 --> REPORT

    REPORT -->|"Blockers found"| CODE
    REPORT -->|"All clear"| DONE(["Ready to commit"])
```

## Architecture Agent Chain

```mermaid
flowchart LR
    ARCH["architect\n(Strategic)"]
    ARCHMOD["architect-module\n(Tactical)"]
    UB["uncle-bob\n(Consultant)"]

    ARCH -->|"Layer assignment\nPort contracts\nDDD constraints"| ARCHMOD
    ARCHMOD -->|"Calls after\ndesign proposal"| UB
    UB -->|"SOLID + Clean Code\nprescriptions"| ARCHMOD
    UB -->|"Layer/boundary\nviolations"| ARCH
    ARCHMOD -->|"Escalates boundary\ndecisions"| ARCH

    style ARCH fill:#1a1a2e,color:#fff
    style ARCHMOD fill:#16213e,color:#fff
    style UB fill:#0f3460,color:#fff
```

## Responsibilities Split

| Agent | Scope | Enforces |
|---|---|---|
| **architect** | System-wide | Hexagonal Architecture, DDD strategic (bounded contexts, aggregates, ports) |
| **architect-module** | Single layer/module | Module internals, pattern selection, code efficiency |
| **uncle-bob** | Design + code | SOLID, Clean Architecture dependency rule, Clean Code (naming, functions, tests) |
| **planner** | Feature scope | Implementation phases, risk assessment |
| **code-reviewer** | Changed code | Security, quality, regressions |
