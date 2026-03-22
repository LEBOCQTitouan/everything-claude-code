# 0011. Command Narrative Convention

Date: 2026-03-22

## Status

Accepted

## Context

The 22 ECC command files executed phases silently or with minimal status output. Of ~35 agent delegation points, none explained which agent was being launched or why. Of ~12 gate/blocking points, only ~3 provided remediation guidance. The implement command's multi-PC TDD loop ran with zero conversational narration between PCs. Users could not understand what choices Claude was making at each step.

## Decision

Establish a "narrate before acting" convention for all ECC commands:

1. Create a shared `skills/narrative-conventions/SKILL.md` defining four narration patterns: agent delegation, gate failure, progress, and result communication.
2. Every command file references this skill and includes inline narrative instructions at delegation points, gate checks, and phase transitions.
3. Narrative instructs Claude *what to communicate* (fact + reasoning), never *how to word it*.
4. Tone: neutral technical, active voice, present tense. Narrative appears before the action it describes.

## Consequences

- **Positive**: Users can follow what Claude is doing and why at every step of every command.
- **Positive**: Shared skill ensures consistent narration patterns — future commands inherit the convention.
- **Positive**: Gate failures now explain remediation, reducing user confusion.
- **Negative**: Command files grow slightly (~5-15 lines each) with narrative instructions.
- **Negative**: Future command authors must include narrative instructions, adding a small maintenance burden.
