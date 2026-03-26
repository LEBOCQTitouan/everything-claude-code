# Tasks: AskUserQuestion Preview Field for Architecture Comparisons

## Pass Conditions

- [x] PC-001: grill-me contains "preview" (>=3) — `grep -c "preview" skills/grill-me/SKILL.md` — done@2026-03-26T22:12:00Z
- [x] PC-002: grill-me contains "visual alternative" — `grep -c "visual alternative" skills/grill-me/SKILL.md` — done@2026-03-26T22:12:00Z
- [x] PC-003: grill-me contains textual skip rule — `grep -ci "MUST NOT.*preview\|skip.*preview" skills/grill-me/SKILL.md` — done@2026-03-26T22:12:00Z
- [x] PC-004: grill-me contains fallback inline rule — `grep -ci "fallback.*inline\|inline.*Markdown" skills/grill-me/SKILL.md` — done@2026-03-26T22:12:00Z
- [x] PC-005: design.md frontmatter has AskUserQuestion — `head -5 commands/design.md | grep -c "AskUserQuestion"` — done@2026-03-26T22:13:00Z
- [x] PC-006: interface-designer contains preview — `grep -c "preview" agents/interface-designer.md` — done@2026-03-26T22:13:00Z
- [x] PC-007: design.md body contains preview — `grep -c "preview" commands/design.md` — done@2026-03-26T22:13:00Z
- [x] PC-008: design.md single-approach skip — `grep -ci "single.*viable\|one.*viable\|only one" commands/design.md` — done@2026-03-26T22:13:00Z
- [x] PC-009: spec-dev contains preview — `grep -c "preview" commands/spec-dev.md` — done@2026-03-26T22:13:30Z
- [x] PC-010: spec-fix contains preview — `grep -c "preview" commands/spec-fix.md` — done@2026-03-26T22:13:30Z
- [x] PC-011: spec-refactor contains preview — `grep -c "preview" commands/spec-refactor.md` — done@2026-03-26T22:13:30Z
- [x] PC-012: spec-dev textual skip instruction — `grep -ci "textual\|MUST NOT.*preview" commands/spec-dev.md` — done@2026-03-26T22:13:30Z
- [x] PC-013: configure-ecc contains preview — `grep -c "preview" skills/configure-ecc/SKILL.md` — done@2026-03-26T22:14:00Z
- [x] PC-014: interviewer contains preview — `grep -c "preview" agents/interviewer.md` — done@2026-03-26T22:14:00Z
- [x] PC-015: configure-ecc multiSelect exclusion — `grep -ci "multiSelect.*MUST NOT\|MUST NOT.*preview" skills/configure-ecc/SKILL.md` — done@2026-03-26T22:14:00Z
- [x] PC-016: Markdown lint passes — `npm run lint` — skipped (no package.json)
- [x] PC-017: Rust build passes — `cargo build` — done@2026-03-26T22:14:30Z
- [x] PC-018: Rust tests pass — `cargo test` — done@2026-03-26T22:14:30Z
- [x] PC-019: ecc validate passes — `cargo run -- validate agents && cargo run -- validate skills && cargo run -- validate commands` — done@2026-03-26T22:14:30Z

## Post-TDD

- [x] E2E tests — done@2026-03-26T22:15:00Z (none required)
- [x] Code review — done@2026-03-26T22:17:00Z (APPROVE, 1 MEDIUM fixed)
- [x] Doc updates — done@2026-03-26T22:18:00Z (CHANGELOG.md)
- [x] Supplemental docs — done@2026-03-26T22:18:00Z (none — no Rust crates modified)
- [x] Write implement-done.md — done@2026-03-26T22:19:00Z

## Status Trail

| Timestamp | Event |
|-----------|-------|
| 2026-03-26T22:12:00Z | Wave 1 complete: PC-001–004 (grill-me) |
| 2026-03-26T22:13:00Z | Wave 2 complete: PC-005–008 (design pipeline) |
| 2026-03-26T22:13:30Z | Wave 3 complete: PC-009–012 (spec-* commands) |
| 2026-03-26T22:14:00Z | Wave 4 complete: PC-013–015 (configure-ecc + interviewer) |
| 2026-03-26T22:14:30Z | Wave 5 complete: PC-016–019 (final gate) |
| 2026-03-26T22:17:00Z | Code review: APPROVE (1 MEDIUM fixed) |
| 2026-03-26T22:18:00Z | CHANGELOG.md updated |
| 2026-03-26T22:19:00Z | implement-done.md written |
