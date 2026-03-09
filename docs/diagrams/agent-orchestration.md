# Agent Orchestration

## Full Development Flow (/plan)

```mermaid
flowchart TD
    USER(["User: /plan"])

    USER --> PLANNER["planner<br/>Breaks task into phases<br/>Test Targets per phase<br/>E2E assessment"]

    PLANNER --> CONFIRM{User confirms?}
    CONFIRM -->|"no / modify"| PLANNER

    CONFIRM -->|"yes"| PHASE["For each phase:"]

    subgraph tdd["TDD Execution Loop"]
        PHASE --> SCAFFOLD["SCAFFOLD<br/>Create interface stubs"]
        SCAFFOLD --> RED["RED<br/>tdd-guide writes<br/>failing tests"]
        RED --> REDC["Commit:<br/>test: add phase tests"]
        REDC --> GREEN["GREEN<br/>tdd-guide implements<br/>minimal code"]
        GREEN --> GREENC["Commit:<br/>feat: implement phase"]
        GREENC --> REFACTOR["REFACTOR<br/>Improve code"]
        REFACTOR --> REFACTORC["Commit:<br/>refactor: improve phase"]
        REFACTORC --> GATE{"Build + full<br/>test suite pass?"}
        GATE -->|"no"| GREEN
        GATE -->|"yes"| NEXT{More phases?}
        NEXT -->|"yes"| PHASE
    end

    NEXT -->|"no"| E2E{"E2E needed?<br/>(from plan assessment)"}
    E2E -->|"yes"| E2EWRITE["e2e-runner<br/>Write + run E2E tests"]
    E2E -->|"no"| E2ERUN["Run existing<br/>E2E suite"]
    E2EWRITE --> CR
    E2ERUN --> CR

    CR["code-reviewer<br/>Mandatory review<br/>on full diff"]

    CR --> ISSUES{CRITICAL/HIGH?}
    ISSUES -->|"yes"| FIX["Fix issues<br/>+ commit"]
    FIX --> CR
    ISSUES -->|"no"| DONE(["Done"])
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
