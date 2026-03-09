# Agent Orchestration

## Full Development Flow

```mermaid
flowchart TD
    USER(["User Request"])

    USER --> PLANNER["planner<br/>Breaks task into phases<br/>identifies risks"]

    PLANNER --> ARCH["architect<br/>Defines hexagonal structure<br/>Bounded contexts, ports, aggregates<br/>DDD model"]

    ARCH --> ARCHMOD["architect-module<br/>Designs module internals<br/>Pattern selection<br/>Code organization"]

    ARCHMOD --> UB1["uncle-bob<br/>Pre-implementation review<br/>SOLID + Clean Architecture<br/>Dependency rule audit"]

    UB1 -->|"SOLID violations<br/>Clean Code prescriptions"| ARCHMOD
    UB1 -->|"Layer violations<br/>Port contract issues"| ARCH

    UB1 -->|"Design approved"| CODE["Code written<br/>by Claude Code"]

    CODE --> CR["code-reviewer<br/>Security, quality<br/>best practices"]
    CODE --> UB2["uncle-bob<br/>Post-implementation<br/>Naming, functions, tests<br/>Clean Code audit"]

    CR --> REPORT["Merged Review Report<br/>[Security] findings<br/>[Clean Code] findings<br/>[Clean Architecture] findings"]
    UB2 --> REPORT

    REPORT -->|"Blockers found"| CODE
    REPORT -->|"All clear"| DONE(["Ready to commit"])
```

## Architecture Agent Chain

```mermaid
flowchart LR
    ARCH["architect<br/>(Strategic)"]
    ARCHMOD["architect-module<br/>(Tactical)"]
    UB["uncle-bob<br/>(Consultant)"]

    ARCH -->|"Layer assignment<br/>Port contracts<br/>DDD constraints"| ARCHMOD
    ARCHMOD -->|"Calls after<br/>design proposal"| UB
    UB -->|"SOLID + Clean Code<br/>prescriptions"| ARCHMOD
    UB -->|"Layer/boundary<br/>violations"| ARCH
    ARCHMOD -->|"Escalates boundary<br/>decisions"| ARCH

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
