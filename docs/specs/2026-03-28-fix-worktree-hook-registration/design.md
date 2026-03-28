# Design: BL-085 — Fix WorktreeCreate/WorktreeRemove Hook Registration

## Overview

Remove `WorktreeCreate`/`WorktreeRemove` delegation hooks from `hooks.json`, add `PostToolUse` entries for `EnterWorktree`/`ExitWorktree`, repurpose existing handler code for PostToolUse stdin format, and extend legacy detection to auto-clean old entries on `ecc install`.

## File Changes

| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | `hooks/hooks.json` | Modify | Remove `WorktreeCreate`/`WorktreeRemove` top-level keys; add two `PostToolUse` entries with `matcher: "EnterWorktree"` and `matcher: "ExitWorktree"` | AC-001.1, AC-002.3 |
| 2 | `crates/ecc-domain/src/config/merge.rs` | Modify | Extend `is_legacy_ecc_hook()` and `is_legacy_ecc_hook_typed()` to detect `"worktree:create:init"` and `"stop:worktree-cleanup-reminder"` command strings as legacy | AC-001.2, AC-001.3, AC-001.4 |
| 3 | `crates/ecc-app/src/hook/handlers/tier3_session/worktree.rs` | Modify | Rename `worktree_create_init` to `post_enter_worktree_session_log`; parse `$.tool_input.worktree_path` -> `$.tool_input.name` -> `"unknown"` from PostToolUse stdin | AC-002.1, AC-002.4 |
| 4 | `crates/ecc-app/src/hook/handlers/tier3_session/mod.rs` | Modify | Update re-export: `worktree_create_init` -> `post_enter_worktree_session_log` | AC-002.1 |
| 5 | `crates/ecc-app/src/hook/handlers/tier1_simple/dev_hooks.rs` | Modify | Rename `worktree_cleanup_reminder` to `post_exit_worktree_cleanup_reminder`; parse `$.tool_input.worktree_path` -> `$.tool_input.name` -> `"unknown"` from PostToolUse stdin; update warn message to include `"Worktree removed"` and path | AC-002.2 |
| 6 | `crates/ecc-app/src/hook/mod.rs` | Modify | Replace dispatch entries: `"worktree:create:init"` -> `"post:enter-worktree:session-log"` calling `post_enter_worktree_session_log`; `"stop:worktree-cleanup-reminder"` -> `"post:exit-worktree:cleanup-reminder"` calling `post_exit_worktree_cleanup_reminder` | AC-002.1, AC-002.2 |
| 7 | `crates/ecc-integration-tests/tests/hook_dispatch.rs` | Modify | Update integration tests: change hook IDs and stdin payloads to PostToolUse format | AC-002.1, AC-002.2 |

## Pass Conditions

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | lint | hooks.json has no WorktreeCreate or WorktreeRemove keys | AC-001.1 | `! grep -q '"WorktreeCreate"\|"WorktreeRemove"' hooks/hooks.json` | exit 0 |
| PC-002 | unit | is_legacy_ecc_hook detects worktree:create:init | AC-001.2 | `cargo test -p ecc-domain is_legacy_ecc_hook_worktree_create_init -- --exact` | pass |
| PC-003 | unit | is_legacy_ecc_hook detects stop:worktree-cleanup-reminder | AC-001.3 | `cargo test -p ecc-domain is_legacy_ecc_hook_worktree_cleanup_reminder -- --exact` | pass |
| PC-004 | unit | remove_legacy_hooks strips old worktree entries from settings | AC-001.4 | `cargo test -p ecc-domain remove_legacy_hooks_removes_worktree_delegation -- --exact` | pass |
| PC-005 | unit | VALID_HOOK_EVENTS contains WorktreeCreate and WorktreeRemove | AC-001.5 | `cargo test -p ecc-domain all_21_hook_events_accepted -- --exact` | pass (already passing, regression guard) |
| PC-006 | unit | post_enter_worktree_session_log parses tool_input.worktree_path | AC-002.1 | `cargo test -p ecc-app post_enter_worktree_session_log_logs_from_tool_input -- --exact` | pass |
| PC-007 | unit | post_enter_worktree_session_log falls back to tool_input.name | AC-002.1 | `cargo test -p ecc-app post_enter_worktree_session_log_fallback_name -- --exact` | pass |
| PC-008 | unit | post_enter_worktree_session_log falls back to unknown | AC-002.1 | `cargo test -p ecc-app post_enter_worktree_session_log_fallback_unknown -- --exact` | pass |
| PC-009 | unit | post_exit_worktree_cleanup_reminder emits warn with path | AC-002.2 | `cargo test -p ecc-app post_exit_worktree_cleanup_with_path -- --exact` | pass |
| PC-010 | unit | post_exit_worktree_cleanup_reminder emits warn without path | AC-002.2 | `cargo test -p ecc-app post_exit_worktree_cleanup_without_path -- --exact` | pass |
| PC-011 | lint | hooks.json has PostToolUse entries for EnterWorktree and ExitWorktree | AC-002.3 | `grep -c '"EnterWorktree"\|"ExitWorktree"' hooks/hooks.json` | 2 |
| PC-012 | unit | post_enter_worktree_session_log passthrough when no session file | AC-002.4 | `cargo test -p ecc-app post_enter_worktree_session_log_no_session_passthrough -- --exact` | pass |
| PC-013 | integration | post:enter-worktree:session-log dispatches via CLI | AC-002.1 | `cargo test -p ecc-integration-tests enter_worktree_session_log_exits_zero -- --exact` | pass |
| PC-014 | integration | post:exit-worktree:cleanup-reminder dispatches via CLI | AC-002.2 | `cargo test -p ecc-integration-tests exit_worktree_cleanup_reminder_exits_zero -- --exact` | pass |
| PC-015 | unit | is_legacy_ecc_hook_typed detects worktree:create:init | AC-001.2 | `cargo test -p ecc-domain is_legacy_ecc_hook_typed_worktree_create_init -- --exact` | pass |
| PC-016 | unit | is_legacy_ecc_hook_typed detects stop:worktree-cleanup-reminder | AC-001.3 | `cargo test -p ecc-domain is_legacy_ecc_hook_typed_worktree_cleanup_reminder -- --exact` | pass |
| PC-017 | unit | remove_legacy_hooks with real-world command string including profiles | AC-001.4 | `cargo test -p ecc-domain remove_legacy_hooks_real_world_worktree_command -- --exact` | pass |
| PC-018 | unit | New hook commands NOT flagged as legacy (negative test) | AC-001.2, AC-001.3 | `cargo test -p ecc-domain is_legacy_ecc_hook_new_worktree_hooks_not_legacy -- --exact` | pass |
| PC-019 | build | Full workspace builds | All | `cargo build --workspace` | exit 0 |
| PC-020 | lint | Clippy clean | All | `cargo clippy -- -D warnings` | exit 0 |

## TDD Implementation Order

### Phase 1: Legacy Detection (Domain Layer)

**Layers: [Entity]**

Extend `is_legacy_ecc_hook()` and `is_legacy_ecc_hook_typed()` to detect old worktree hook command strings.

**TDD order:**
1. **RED**: Write tests PC-002, PC-003, PC-004, PC-015, PC-016, PC-017, PC-018 in `crates/ecc-domain/src/config/merge.rs` (unit tests module). PC-017 uses the exact real-world command `ecc-hook "worktree:create:init" "standard,strict"`. PC-018 asserts `ecc-hook "post:enter-worktree:session-log" "standard,strict"` is NOT legacy.
2. **GREEN**: Add legacy detection logic — in `is_legacy_ecc_hook()`, after the existing `ecc-hook`/`ecc-shell-hook` early return, add a new condition: if `cmd.starts_with("ecc-hook ")` was already handled by the `continue`, the old worktree hooks use the exact command strings `ecc-hook "worktree:create:init"` and `ecc-hook "stop:worktree-cleanup-reminder"` — these are current-format `ecc-hook` commands, not legacy per se. The correct approach is to detect them as **deprecated hook IDs** within the `ecc-hook` command pattern. Modify the `ecc-hook`/`ecc-shell-hook` branch: in addition to checking for `dist/hooks/`, also return `true` if cmd contains `"worktree:create:init"` or `"stop:worktree-cleanup-reminder"`. Apply the same change to `is_legacy_ecc_hook_typed()`.
3. **REFACTOR**: Check PC-005 still passes (regression guard for VALID_HOOK_EVENTS)

**Commit cadence:**
- `test: add legacy worktree hook detection tests (BL-085 Phase 1)`
- `feat: detect worktree delegation hooks as legacy (BL-085 Phase 1)`

**Files modified:**
- `crates/ecc-domain/src/config/merge.rs`

**Risk:** Low — extending existing pattern matching, no architectural change.

---

### Phase 2: Repurpose Handlers for PostToolUse Stdin (Application Layer)

**Layers: [UseCase]**

Rename handlers and update stdin parsing from `$.worktree_path` to `$.tool_input.worktree_path` -> `$.tool_input.name` -> `"unknown"`.

**TDD order:**
1. **RED**: Write tests PC-006, PC-007, PC-008, PC-009, PC-010, PC-012 in the respective handler test modules
2. **GREEN**:
   - In `worktree.rs`: rename `worktree_create_init` to `post_enter_worktree_session_log`, change JSON path from `$.worktree_path` to `$.tool_input.worktree_path` with fallback to `$.tool_input.name`, then `"unknown"`
   - In `dev_hooks.rs`: rename `worktree_cleanup_reminder` to `post_exit_worktree_cleanup_reminder`, change JSON path similarly, update warn message to include `"Worktree removed"` and the extracted path
   - In `tier3_session/mod.rs`: update re-export
   - In `mod.rs` dispatch: replace old hook IDs with new ones
3. **REFACTOR**: Remove old test functions that tested the previous function names/stdin format

**Commit cadence:**
- `test: add PostToolUse worktree handler tests (BL-085 Phase 2)`
- `feat: repurpose worktree handlers for PostToolUse stdin (BL-085 Phase 2)`

**Files modified:**
- `crates/ecc-app/src/hook/handlers/tier3_session/worktree.rs`
- `crates/ecc-app/src/hook/handlers/tier3_session/mod.rs`
- `crates/ecc-app/src/hook/handlers/tier1_simple/dev_hooks.rs`
- `crates/ecc-app/src/hook/mod.rs`

**Risk:** Medium — renaming functions touches re-exports and dispatch table; must update all references atomically.

---

### Phase 3: hooks.json + Integration Tests (Adapter Layer)

**Layers: [Adapter]**

Update hooks.json config and integration tests.

**TDD order:**
1. **RED**: Write integration tests PC-013, PC-014 in `hook_dispatch.rs` (these will fail because hooks.json still has old entries and integration tests reference old hook IDs)
2. **GREEN**:
   - Remove `WorktreeCreate` and `WorktreeRemove` keys from `hooks/hooks.json`
   - Add two `PostToolUse` entries: one with `matcher: "EnterWorktree"` calling `ecc-hook "post:enter-worktree:session-log"`, another with `matcher: "ExitWorktree"` calling `ecc-hook "post:exit-worktree:cleanup-reminder"`
   - Update existing integration tests `worktree_create_init_exits_zero` and `worktree_remove_dispatches_exits_zero` to use new hook IDs and PostToolUse stdin format
3. **VERIFY**: Run PC-001, PC-011, PC-017, PC-018

**Commit cadence:**
- `test: update integration tests for PostToolUse worktree hooks (BL-085 Phase 3)`
- `feat: replace worktree delegation hooks with PostToolUse entries (BL-085 Phase 3)`

**Files modified:**
- `hooks/hooks.json`
- `crates/ecc-integration-tests/tests/hook_dispatch.rs`

**Risk:** Low — config file change + test updates.

---

### PostToolUse stdin format reference

The handlers receive JSON like:
```json
{
  "tool_name": "EnterWorktree",
  "tool_input": {
    "worktree_path": "/tmp/wt-feature",
    "name": "feature-branch"
  }
}
```

Extraction logic: `$.tool_input.worktree_path` -> `$.tool_input.name` -> `"unknown"`.

### hooks.json PostToolUse entries (target state)

```json
{
  "matcher": "EnterWorktree",
  "hooks": [
    {
      "type": "command",
      "command": "ecc-hook \"post:enter-worktree:session-log\" \"standard,strict\"",
      "async": true,
      "timeout": 5
    }
  ],
  "description": "Log worktree creation to active session file"
},
{
  "matcher": "ExitWorktree",
  "hooks": [
    {
      "type": "command",
      "command": "ecc-hook \"post:exit-worktree:cleanup-reminder\" \"standard,strict\""
    }
  ],
  "description": "Remind about unmerged changes after worktree removal"
}
```

### Legacy detection logic (target state)

In both `is_legacy_ecc_hook()` and `is_legacy_ecc_hook_typed()`, within the `ecc-hook`/`ecc-shell-hook` branch:

```rust
if cmd.starts_with("ecc-hook ") || cmd.starts_with("ecc-shell-hook ") {
    if cmd.contains("dist/hooks/") {
        return true;
    }
    // Deprecated worktree delegation hook IDs (BL-085)
    if cmd.contains("worktree:create:init") || cmd.contains("stop:worktree-cleanup-reminder") {
        return true;
    }
    continue;
}
```

## E2E Assessment

- **Touches user-facing flows?** No — hooks are internal lifecycle, no CLI command or UI change
- **Crosses 3+ modules end-to-end?** No — domain (legacy detection) + app (handlers + dispatch) only
- **New E2E tests needed?** No — integration tests in `ecc-integration-tests` are sufficient; existing E2E suite will be run as a gate after all phases

## Risks and Mitigations

- **Risk**: Renaming functions breaks compilation if a re-export is missed
  - Mitigation: `cargo build --workspace` after Phase 2; function names are only used in 3 files (handler, mod.rs re-export, dispatch table). Note: `tier1_simple/mod.rs` uses `pub use dev_hooks::*` glob re-exports, so renaming in `dev_hooks.rs` propagates automatically — no manual change needed there.
- **Risk**: PostToolUse stdin format differs from documentation
  - Mitigation: Best-effort extraction with fallback chain; handler never errors on unexpected format
- **Risk**: Existing users have old worktree hooks in settings.json that persist
  - Mitigation: Legacy detection (Phase 1) ensures `ecc install` auto-cleans on next run

## Success Criteria

- [ ] `hooks/hooks.json` has zero `WorktreeCreate`/`WorktreeRemove` keys
- [ ] `hooks/hooks.json` has `PostToolUse` entries matching `EnterWorktree` and `ExitWorktree`
- [ ] `is_legacy_ecc_hook()` detects both old worktree command strings
- [ ] `remove_legacy_hooks()` strips old entries from settings.json
- [ ] `VALID_HOOK_EVENTS` still contains `WorktreeCreate` and `WorktreeRemove`
- [ ] New handlers parse PostToolUse stdin with 3-level fallback
- [ ] Graceful degradation when no session file exists
- [ ] New hook commands NOT flagged as legacy (negative test)
- [ ] All 20 pass conditions green
- [ ] CLAUDE.md test count updated
- [ ] `cargo clippy -- -D warnings` clean
- [ ] `cargo build --workspace` succeeds
