# Design: BL-064 Sub-Spec B — Element Registry and Cross-Reference Matrix

## Prerequisites (Sub-Spec A Contract)

Sub-Spec B imports the following from `ecc-domain::cartography` and `ecc-app` — these
must exist before any Phase in this design executes:

| Symbol | Module | Kind |
|--------|--------|------|
| `merge_section` | `ecc_domain::cartography::merge` | `fn` |
| `derive_slug` | `ecc_domain::cartography::slug` | `fn` |
| `validate_journey` | `ecc_domain::cartography::validation` | `fn` |
| `validate_flow` | `ecc_domain::cartography::validation` | `fn` |
| `check_staleness` | `ecc_domain::cartography::staleness` | `fn` |
| `calculate_coverage` | `ecc_domain::cartography::coverage` | `fn` |
| `SessionDelta` | `ecc_domain::cartography::types` | `struct` |
| `ProjectType` | `ecc_domain::cartography::types` | `enum` |
| `validate_cartography` use-case | `ecc_app::validate_cartography` | `mod` |
| `start_cartography` / `end_cartography` handlers | `ecc_app::hook::handlers::tier3_session::cartography` | `fn` |

No Sub-Spec B file re-implements any of the above.

---

## File Changes Table

Files are listed in dependency order — each file's dependencies appear above it.

| # | File | Change | Layer | Depends On |
|---|------|--------|-------|------------|
| 1 | `crates/ecc-domain/src/cartography/element_types.rs` | New | Entity | Sub-Spec A `cartography` module |
| 2 | `crates/ecc-domain/src/cartography/element_validation.rs` | New | Entity | File #1 |
| 3 | `crates/ecc-domain/src/cartography/cross_reference.rs` | New | Entity | File #1 |
| 4 | `crates/ecc-domain/src/cartography/mod.rs` | Modify: add `pub mod element_types; pub mod element_validation; pub mod cross_reference;` | Entity | Files #1–3 |
| 5 | `crates/ecc-app/src/hook/handlers/tier3_session/cartography.rs` | Modify: add `scaffold_elements_dir`, extend post-loop to dispatch element generator and INDEX regen | UseCase | Files #1–4 |
| 6 | `crates/ecc-app/src/validate_cartography.rs` | Modify: scan `elements/`, call `validate_element`, staleness check, INDEX presence warn | UseCase + Adapter | Files #2–3 |
| 7 | `agents/cartography-element-generator.md` | New | Framework | Files #1–4 (runtime) |
| 8 | `agents/cartographer.md` | Modify: add element-dispatch step and INDEX regeneration step | Framework | File #7 |
| 9 | `docs/adr/NNN-cartography-post-loop-element-generation.md` | New | Framework | — |
| 10 | `docs/adr/NNN-cartography-two-tier-element-types.md` | New | Framework | — |
| 11 | `docs/adr/NNN-cartography-index-full-regen.md` | New | Framework | — |
| 12 | `docs/domain/bounded-contexts.md` | Modify: add element types to cartography bounded context | Framework | Files #1–3 |
| 13 | `CHANGELOG.md` | Modify: add Sub-Spec B entry | Framework | — |

Note: test code lives in `#[cfg(test)]` modules inline within files #1–6 — not counted as separate files.

---

## Pass Conditions Table

Each PC has: ID, type, description, command (verbatim bash), expected output, and verified ACs.

### Phase 1: Domain Types

**PC-001** — Unit: `ElementType` has all required variants
Command: `cargo test -p ecc-domain --lib cartography::element_types::tests::element_type_all_variants`
Expected: `test result: ok`
Verifies: AC-010.1

**PC-002** — Unit: `ElementType::Unknown` serialises to `"unknown"` and round-trips
Command: `cargo test -p ecc-domain --lib cartography::element_types::tests::element_type_unknown_serde`
Expected: `test result: ok`
Verifies: AC-010.4

**PC-003** — Unit: `ElementEntry` JSON round-trip preserves all fields
Command: `cargo test -p ecc-domain --lib cartography::element_types::tests::element_entry_json_roundtrip`
Expected: `test result: ok`
Verifies: AC-010.2

**PC-004** — Unit: `infer_element_type_from_crate()` maps all 9 crate names
Command: `cargo test -p ecc-domain --lib cartography::element_types::tests::crate_type_inference`
Expected: `test result: ok`
Verifies: AC-010.6

**PC-005** — Unit: `infer_element_type_from_path()` maps directory prefixes to agent/command/skill/hook/rule; fallback Unknown
Command: `cargo test -p ecc-domain --lib cartography::element_types::tests::path_type_inference`
Expected: `test result: ok`
Verifies: AC-012.7

**PC-006** — Structural: no `std::fs`, `std::process`, or `std::net` in `cartography/`
Command: `grep -rn 'std::fs\|std::process\|std::net' crates/ecc-domain/src/cartography/ ; echo "EXIT:$?"`
Expected: last line is `EXIT:1` (grep exits 1 when no match found)
Verifies: AC-010.3

**PC-007** — Unit: `derive_slug` imported from Sub-Spec A module, no new slug function declared in `element_types.rs`
Command: `grep -n 'fn derive_slug\|fn slug' crates/ecc-domain/src/cartography/element_types.rs ; echo "EXIT:$?"`
Expected: last line is `EXIT:1` (no local slug fn)
Verifies: AC-010.5

**PC-008** — Compile: Sub-Spec A symbols importable from `element_types.rs` (compile check)
Command: `cargo check -p ecc-domain 2>&1; echo "EXIT:$?"`
Expected: last line is `EXIT:0`
Verifies: AC-010.7

---

### Phase 2: Domain Validation and Cross-Reference

**PC-009** — Unit: `validate_element()` returns `Ok(())` for file with all four sections
Command: `cargo test -p ecc-domain --lib cartography::element_validation::tests::valid_element_all_sections`
Expected: `test result: ok`
Verifies: AC-011.1

**PC-010** — Unit: `validate_element()` returns `Err` listing missing section names
Command: `cargo test -p ecc-domain --lib cartography::element_validation::tests::missing_sections_reported`
Expected: `test result: ok`
Verifies: AC-011.2

**PC-011** — Unit: `validate_element()` is order-independent (sections in random order)
Command: `cargo test -p ecc-domain --lib cartography::element_validation::tests::section_order_independent`
Expected: `test result: ok`
Verifies: AC-011.4

**PC-012** — Unit: `build_cross_reference_matrix()` produces rows=elements, columns=journeys+flows, cells=Y/blank
Command: `cargo test -p ecc-domain --lib cartography::cross_reference::tests::matrix_structure`
Expected: `test result: ok`
Verifies: AC-014.1, AC-014.4

**PC-013** — Unit: `build_cross_reference_matrix()` renders journey columns before flow columns
Command: `cargo test -p ecc-domain --lib cartography::cross_reference::tests::journey_columns_before_flow`
Expected: `test result: ok`
Verifies: AC-014.4

**PC-014** — Unit: `build_cross_reference_matrix()` with empty element list returns header-only table
Command: `cargo test -p ecc-domain --lib cartography::cross_reference::tests::empty_element_list`
Expected: `test result: ok`
Verifies: AC-014.1

---

### Phase 3: Handler Extensions

**PC-015** — Unit: `scaffold_elements_dir()` creates `docs/cartography/elements/` and README when absent
Command: `cargo test -p ecc-app --lib hook::handlers::tier3_session::cartography::tests::scaffold_creates_elements_dir`
Expected: `test result: ok`
Verifies: AC-016.1, AC-012.8

**PC-016** — Unit: `scaffold_elements_dir()` is idempotent when directory already exists
Command: `cargo test -p ecc-app --lib hook::handlers::tier3_session::cartography::tests::scaffold_idempotent`
Expected: `test result: ok`
Verifies: AC-016.2

**PC-017** — Unit: element generator dispatched AFTER journey+flow generators complete (ordering assertion via MockExecutor call log)
Command: `cargo test -p ecc-app --lib hook::handlers::tier3_session::cartography::tests::element_dispatch_order`
Expected: `test result: ok`
Verifies: AC-013.1

**PC-018** — Unit: element generator not dispatched when delta has no element targets
Command: `cargo test -p ecc-app --lib hook::handlers::tier3_session::cartography::tests::no_dispatch_without_targets`
Expected: `test result: ok`
Verifies: AC-013.4

**PC-019** — Unit: on element generator failure, git reset runs and no archive produced
Command: `cargo test -p ecc-app --lib hook::handlers::tier3_session::cartography::tests::element_failure_resets_git`
Expected: `test result: ok`
Verifies: AC-013.3

**PC-020** — Unit: successful element generation causes `git add docs/cartography/` to be staged
Command: `cargo test -p ecc-app --lib hook::handlers::tier3_session::cartography::tests::element_success_stages_files`
Expected: `test result: ok`
Verifies: AC-013.2

**PC-021** — Unit: INDEX.md fully replaced (old content gone, new content present)
Command: `cargo test -p ecc-app --lib hook::handlers::tier3_session::cartography::tests::index_full_replacement`
Expected: `test result: ok`
Verifies: AC-014.2, AC-014.7

**PC-021b** — Unit: INDEX.md regeneration runs after element generators (ordering assertion)
Command: `cargo test -p ecc-app --lib hook::handlers::tier3_session::cartography::tests::index_regen_after_elements`
Expected: `test result: ok`
Verifies: AC-014.3

---

### Phase 4: Validate CLI Extensions

**PC-022** — Unit: `validate_cartography` scans `elements/` and returns errors for invalid files
Command: `cargo test -p ecc-app --lib validate_cartography::tests::invalid_element_exits_with_error`
Expected: `test result: ok`
Verifies: AC-011.3

**PC-023** — Unit: `validate_cartography` reports warning (not error) when `INDEX.md` absent
Command: `cargo test -p ecc-app --lib validate_cartography::tests::missing_index_warns_not_errors`
Expected: `test result: ok`
Verifies: AC-014.5

**PC-024** — Unit: `validate_cartography` reports warning with missing slugs when INDEX.md is stale
Command: `cargo test -p ecc-app --lib validate_cartography::tests::stale_index_warns_missing_slugs`
Expected: `test result: ok`
Verifies: AC-014.6

**PC-025** — Unit: `validate_cartography` exits cleanly when `elements/` is missing (no error)
Command: `cargo test -p ecc-app --lib validate_cartography::tests::missing_elements_dir_clean_exit`
Expected: `test result: ok`
Verifies: AC-016.3

**PC-026** — Unit: staleness check includes element files with CARTOGRAPHY-META marker
Command: `cargo test -p ecc-app --lib validate_cartography::tests::staleness_includes_elements`
Expected: `test result: ok`
Verifies: AC-015.1

**PC-027** — Unit: coverage calculation includes element-referenced sources
Command: `cargo test -p ecc-app --lib validate_cartography::tests::coverage_includes_element_sources`
Expected: `test result: ok`
Verifies: AC-015.2

**PC-028** — Unit: coverage below 50% includes all source types (journeys+flows+elements) in gap report
Command: `cargo test -p ecc-app --lib validate_cartography::tests::low_coverage_includes_all_gap_types`
Expected: `test result: ok`
Verifies: AC-015.3

---

### Phase 5: Agents, Config, Gates

**PC-029** — Schema: `agents/cartography-element-generator.md` passes `ecc validate agents`
Command: `cargo run -p ecc-cli -- validate agents 2>&1 | grep -c 'cartography-element-generator'`
Expected: `0` (no error lines mentioning the agent)
Verifies: AC-012.1 (skeleton)

**PC-030** — Schema: `agents/cartographer.md` passes `ecc validate agents` after modification
Command: `cargo run -p ecc-cli -- validate agents 2>&1 | grep -c 'cartographer'`
Expected: `0`
Verifies: AC-013.1 (agent wiring)

**PC-031** — File-existence: `agents/cartography-element-generator.md` has required frontmatter fields
Command: `grep -c 'name:\|description:\|tools:\|model:' agents/cartography-element-generator.md`
Expected: `4` (at least 4 matching lines)
Verifies: AC-012.1

**PC-032** — Agent-content: generator instructs `merge_section` usage for existing files
Command: `grep -c 'merge_section\|delta.merge\|delta merge\|merge section' agents/cartography-element-generator.md`
Expected: `>= 1`
Verifies: AC-012.2

**PC-033** — Agent-content: generator instructs relative Markdown links in Relationships section
Command: `grep -c 'relative.*link\|relative Markdown\|\[.*\](\.\./' agents/cartography-element-generator.md`
Expected: `>= 1`
Verifies: AC-012.3

**PC-034** — Agent-content: generator instructs Participating Flows with relative links
Command: `grep -c 'Participating Flows\|flows.*link\|relative.*flow' agents/cartography-element-generator.md`
Expected: `>= 1`
Verifies: AC-012.4

**PC-035** — Agent-content: generator instructs Participating Journeys with relative links
Command: `grep -c 'Participating Journeys\|journeys.*link\|relative.*journey' agents/cartography-element-generator.md`
Expected: `>= 1`
Verifies: AC-012.5

**PC-036** — Agent-content: generator instructs GAP marker for unknown element type
Command: `grep -c 'GAP\|unknown.*marker\|element_type.*unknown' agents/cartography-element-generator.md`
Expected: `>= 1`
Verifies: AC-012.6

**PC-037** — File-content: `agents/cartographer.md` references element generator dispatch and INDEX
Command: `grep -c 'cartography-element-generator\|element.*dispatch\|INDEX' agents/cartographer.md`
Expected: `>= 2`
Verifies: AC-013.1, AC-014.3

**PC-038** — Integration: full test suite passes
Command: `cargo test --workspace 2>&1 | tail -5`
Expected: `test result: ok` in tail, no `FAILED`
Verifies: AC-010.1–010.7, AC-011.1–011.4, AC-012.1–012.8, AC-013.1–013.4, AC-014.1–014.7, AC-015.1–015.3, AC-016.1–016.3 (gate)

**PC-039** — Lint: `cargo clippy` zero warnings
Command: `cargo clippy --workspace -- -D warnings 2>&1; echo "EXIT:$?"`
Expected: last line is `EXIT:0`
Verifies: (build quality gate)

**PC-040** — Build: release build succeeds
Command: `cargo build --release 2>&1; echo "EXIT:$?"`
Expected: last line is `EXIT:0`
Verifies: (build quality gate)

---

## AC → PC Coverage Matrix

| AC | PC(s) |
|----|-------|
| AC-010.1 | PC-001, PC-038 |
| AC-010.2 | PC-003 |
| AC-010.3 | PC-006 |
| AC-010.4 | PC-002 |
| AC-010.5 | PC-007 |
| AC-010.6 | PC-004 |
| AC-010.7 | PC-008 |
| AC-011.1 | PC-009 |
| AC-011.2 | PC-010 |
| AC-011.3 | PC-022 |
| AC-011.4 | PC-011 |
| AC-012.1 | PC-029, PC-031 |
| AC-012.2 | PC-032 (agent instructs merge_section for delta-merge) |
| AC-012.3 | PC-033 (agent instructs relative links in Relationships) |
| AC-012.4 | PC-034 (agent instructs Participating Flows links) |
| AC-012.5 | PC-035 (agent instructs Participating Journeys links) |
| AC-012.6 | PC-036 (agent instructs GAP marker for unknown type) |
| AC-012.7 | PC-005 |
| AC-012.8 | PC-015 |
| AC-013.1 | PC-017, PC-030, PC-037 |
| AC-013.2 | PC-020 |
| AC-013.3 | PC-019 |
| AC-013.4 | PC-018 |
| AC-014.1 | PC-012, PC-014 |
| AC-014.2 | PC-021 (full replacement — old content gone, new content present) |
| AC-014.3 | PC-021b, PC-037 |
| AC-014.4 | PC-012, PC-013 |
| AC-014.5 | PC-023 |
| AC-014.6 | PC-024 |
| AC-014.7 | PC-021 (path assertion: write target is `elements/INDEX.md`) |
| AC-015.1 | PC-026 |
| AC-015.2 | PC-027 |
| AC-015.3 | PC-028 |
| AC-016.1 | PC-015 |
| AC-016.2 | PC-016 |
| AC-016.3 | PC-025 |

All 36 ACs covered. ✓

---

## TDD Order (5 Phases)

### Phase 1 — Domain Types (`element_types.rs`)

**Layers:** Entity

**Files touched:** `crates/ecc-domain/src/cartography/element_types.rs`, `crates/ecc-domain/src/cartography/mod.rs`

**What to build:**
- `ElementType` enum: universal variants (`Module`, `Interface`, `Config`, `Unknown`) + ECC overlay (`Command`, `Agent`, `Skill`, `Hook`, `Rule`, `Crate`, `Port`, `Adapter`, `DomainEntity`)
- `ElementEntry` struct: `slug: String`, `element_type: ElementType`, `purpose: String`, `uses: Vec<String>`, `used_by: Vec<String>`, `participating_flows: Vec<String>`, `participating_journeys: Vec<String>`, `sources: Vec<String>`, `last_updated: String`
- `infer_element_type_from_crate(crate_name: &str) -> ElementType` — pure function, no I/O
- `infer_element_type_from_path(source_path: &str) -> ElementType` — path prefix matching
- Derives: `Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize` on both types
- `ElementType::Unknown` serialises as `"unknown"` (verified via `serde_json::to_string`)
- `mod.rs` extended: `pub mod element_types; pub mod element_validation; pub mod cross_reference;`

**TDD cycle:**
1. Write tests PC-001 through PC-008 (RED) → commit `test: add element_types domain tests`
2. Implement types and inference functions (GREEN) → commit `feat: add ElementEntry, ElementType, inference fns`
3. Refactor: extract crate-name constants, ensure no clippy warnings → commit `refactor: extract crate type map constants`

**Boy Scout Delta:** Scan `crates/ecc-domain/src/cartography/types.rs` — if `SessionDelta` has a magic constant, extract it.

---

### Phase 2 — Domain Validation and Cross-Reference (`element_validation.rs`, `cross_reference.rs`)

**Layers:** Entity

**Files touched:** `crates/ecc-domain/src/cartography/element_validation.rs`, `crates/ecc-domain/src/cartography/cross_reference.rs`

**What to build:**

`element_validation.rs`:
- `ElementValidationError` enum: `MissingSections(Vec<String>)`
- `validate_element(content: &str) -> Result<(), ElementValidationError>`
  - Required sections: `## Overview`, `## Relationships`, `## Participating Flows`, `## Participating Journeys`
  - Order-independent (scan all lines, collect present headings, diff against required set)

`cross_reference.rs`:
- `CrossReferenceMatrix` struct: Markdown table with element rows, journey+flow columns
- `build_cross_reference_matrix(element_files: &[ElementEntry], journey_slugs: &[&str], flow_slugs: &[&str]) -> String`
  - Renders journey columns first, then flow columns
  - Cell = `Y` when element's `participating_journeys` / `participating_flows` contains the slug, else blank
  - Returns full Markdown table string (no I/O)

**TDD cycle:**
1. Write tests PC-009 through PC-014 (RED) → commit `test: add element_validation and cross_reference tests`
2. Implement (GREEN) → commit `feat: implement validate_element and build_cross_reference_matrix`
3. Refactor: extract `REQUIRED_ELEMENT_SECTIONS` constant → commit `refactor: extract required sections constant`

**Boy Scout Delta:** Scan `element_validation.rs` neighbours (e.g., `validation.rs` from Sub-Spec A) for a TODO; resolve one.

---

### Phase 3 — Handler Extensions (`cartography.rs`)

**Layers:** UseCase

**Files touched:** `crates/ecc-app/src/hook/handlers/tier3_session/cartography.rs`

**What to build:**

Add to the existing `start_cartography` handler:
- `scaffold_elements_dir(base: &Path, ports: &HookPorts) -> Result<(), String>`
  - Creates `docs/cartography/elements/` + `README.md` stub when absent
  - Idempotent: checks `fs.exists` before creating

Extend the post-loop section of the existing `end_cartography` / session-end-cartography handler:
- After journey + flow generators complete, check delta for element targets
- If element targets present: dispatch `cartography-element-generator` agent via `ShellExecutor`
  - On failure: `git reset HEAD docs/cartography/`, return failure result (same path as existing flow failure)
  - On success: `git add docs/cartography/` (existing scoped-add reused)
- After elements: regenerate `docs/cartography/elements/INDEX.md` by calling `build_cross_reference_matrix`; write via `fs.write`
  - Full replacement (no delta-merge)

**Ordering contract:** MockExecutor call log must show: `journey_gen` call index < `flow_gen` call index < `element_gen` call index < `index_write`.

**TDD cycle:**
1. Write tests PC-015 through PC-021b (RED) → commit `test: add cartography handler element/INDEX tests`
2. Implement scaffold + post-loop dispatch (GREEN) → commit `feat: extend cartography handler for elements and INDEX`
3. Refactor: extract `dispatch_element_generator` helper fn → commit `refactor: extract dispatch_element_generator helper`

**Boy Scout Delta:** If `cartography.rs` exceeds 800 lines after changes, extract the scaffolding helpers into a `cartography_helpers.rs` sub-module.

---

### Phase 4 — Validate CLI Extensions (`validate_cartography.rs`)

**Layers:** UseCase, Adapter

**Files touched:** `crates/ecc-app/src/validate_cartography.rs`

**What to build:**

Extend existing `validate_cartography` function:
1. **Elements scan:** Walk `docs/cartography/elements/` (if present). For each `.md` file that is not `INDEX.md` or `README.md`: call `validate_element(content)`. Collect errors; print and exit 1 on any failure.
2. **INDEX.md presence:** Check `docs/cartography/elements/INDEX.md`. If absent: emit warning (not error) to stderr; continue.
3. **INDEX.md staleness:** Parse INDEX.md table header slugs. Compare against known element slugs from scanned files. If any element slug is absent from INDEX.md: warn with the missing list.
4. **Staleness (elements):** Reuse `check_staleness` from Sub-Spec A. Include element files with `CARTOGRAPHY-META` marker in the staleness scan.
5. **Coverage:** Pass element-referenced `sources` into `calculate_coverage`. When coverage < 50%, include element sources in gap output.
6. **Missing `elements/` dir:** If directory absent, continue cleanly (no error, no warning needed per AC-016.3).

**TDD cycle:**
1. Write tests PC-022 through PC-028 (RED) → commit `test: add validate_cartography element/INDEX/staleness/coverage tests`
2. Implement (GREEN) → commit `feat: extend validate_cartography for elements, INDEX, staleness, coverage`
3. Refactor: extract `validate_elements_dir` helper → commit `refactor: extract validate_elements_dir helper`

**Boy Scout Delta:** Rename any vague variable `data` or `result` in `validate_cartography.rs` to a domain name (e.g., `element_errors`, `coverage_result`).

---

### Phase 5 — Agents, Config, Gates

**Layers:** Framework

**Files touched:** `agents/cartography-element-generator.md`, `agents/cartographer.md`

**What to build:**

`agents/cartography-element-generator.md` — new agent:
- Frontmatter: `name`, `description`, `tools`, `model` (required per ECC conventions)
- `tools` must include `Read`, `Write`, `Edit` (for creating/updating element files)
- `model: claude-sonnet-4-6` (implementation work per performance rules)
- Instructions:
  1. Receive delta (element source paths)
  2. For each source: infer element type via path prefix (agents/ → Agent, etc.)
  3. Check if `docs/cartography/elements/<slug>.md` exists
     - If yes: delta-merge using `merge_section()` (preserves manual content)
     - If no: create from skeleton template with all four required sections
  4. For unknown type: add `GAP` marker in file
  5. Create `elements/` dir if absent (AC-012.8)
  6. Relationships section: `uses` and `used_by` as relative Markdown links
  7. Participating Flows / Journeys: relative links to flow/journey files

`agents/cartographer.md` — modify existing:
- After existing journey + flow dispatch steps, add:
  - Step: "Dispatch cartography-element-generator with element targets from delta"
  - Step: "After element generator succeeds, regenerate INDEX.md at `docs/cartography/elements/INDEX.md`"
- Failure path: same as journey/flow failure (git reset, no archive)

**TDD cycle:**
1. Verify agent schema passes `ecc validate agents` (PC-029, PC-030) — this is RED if frontmatter is missing
2. Create agent files with correct frontmatter (GREEN) → commit `feat: add cartography-element-generator agent`; `feat: extend cartographer agent for element dispatch and INDEX`
3. No refactor step needed for markdown-only changes

**PC-038, PC-039, PC-040** run here as final gates.

---

## E2E Assessment

- **Touches user-facing flows?** Yes — `ecc validate cartography` CLI command gains new output (element errors, INDEX warnings). Session-end hook gains new file writes.
- **Crosses 3+ modules end-to-end?** Yes — delta → handler → agent → element files → validate CLI → INDEX.md
- **New E2E tests needed?** No — agent-side generation is non-deterministic prose. E2E coverage is provided by:
  - Phase 3 handler tests (MockExecutor verifies dispatch order and git staging)
  - Phase 4 validate tests (InMemoryFileSystem verifies correct CLI exit codes and warnings)
  - PC-038 full workspace test suite as final gate
- Existing E2E suite will be run as a gate after all phases.

---

## Risks and Mitigations

| Risk | Severity | Mitigation |
|------|----------|------------|
| Sub-Spec A not yet implemented when Phase 1 starts | High | Gate Phase 1 on Sub-Spec A completing; use `cargo check` PC-008 to surface missing symbols |
| `cartography.rs` handler exceeds 800-line limit after Phase 3 | Medium | Boy Scout Delta: extract helpers module proactively if file approaches limit |
| `build_cross_reference_matrix` produces unstable column order | Medium | Sort journey slugs alphabetically, then flow slugs alphabetically before rendering |
| Agent markdown content drift from schema | Low | `ecc validate agents` (PC-029, PC-030) run in Phase 5 gate |
| `elements/` scan picks up INDEX.md or README.md as element files | Low | Filter: skip files whose name is `INDEX.md` or `README.md` in the scan loop |

---

## Success Criteria

- [ ] All 41 PCs pass (PC-001 through PC-040, plus PC-021b)
- [ ] All 36 ACs covered (coverage matrix above)
- [ ] `cargo clippy --workspace -- -D warnings` clean
- [ ] `cargo build --release` succeeds
- [ ] `crates/ecc-domain/src/cartography/` contains zero `std::fs`/`std::process`/`std::net` imports
- [ ] `agents/cartography-element-generator.md` passes `ecc validate agents`
- [ ] `agents/cartographer.md` passes `ecc validate agents`
- [ ] Handler file `cartography.rs` stays under 800 lines
