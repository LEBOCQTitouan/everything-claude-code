#!/usr/bin/env bash
# Test suite for .claude/hooks/phase-gate.sh
# Run: bash tests/hooks/test-phase-gate.sh
set -uo pipefail

PASS_COUNT=0
FAIL_COUNT=0
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
HOOK="$PROJECT_ROOT/.claude/hooks/phase-gate.sh"

# --- Helpers ---

setup() {
  TEMP_DIR=$(mktemp -d)
  mkdir -p "$TEMP_DIR/.claude/workflow"
}

teardown() {
  rm -rf "$TEMP_DIR"
}

write_state() {
  local phase="$1"
  cat > "$TEMP_DIR/.claude/workflow/state.json" <<EOF
{"phase": "$phase", "started_at": "2026-03-21T00:00:00Z"}
EOF
}

run_hook() {
  local tool_name="$1"
  local file_path="${2:-}"
  local json
  json=$(printf '{"tool_name": "%s", "tool_input": {"file_path": "%s"}}' "$tool_name" "$file_path")
  echo "$json" | CLAUDE_PROJECT_DIR="$TEMP_DIR" bash "$HOOK" 2>/dev/null
}

assert_exit() {
  local test_name="$1"
  local expected_exit="$2"
  local tool_name="$3"
  local file_path="${4:-}"

  run_hook "$tool_name" "$file_path"
  local actual_exit=$?

  if [ "$actual_exit" -eq "$expected_exit" ]; then
    echo "PASS  $test_name"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  $test_name (expected exit $expected_exit, got $actual_exit)"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
}

assert_stderr_contains() {
  local test_name="$1"
  local pattern="$2"
  local tool_name="$3"
  local file_path="${4:-}"

  local json
  json=$(printf '{"tool_name": "%s", "tool_input": {"file_path": "%s"}}' "$tool_name" "$file_path")
  local stderr_output
  stderr_output=$(echo "$json" | CLAUDE_PROJECT_DIR="$TEMP_DIR" bash "$HOOK" 2>&1 >/dev/null || true)

  if echo "$stderr_output" | grep -q "$pattern"; then
    echo "PASS  $test_name"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  $test_name (stderr did not contain '$pattern')"
    echo "  stderr was: $stderr_output"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
}

# --- Tests ---

# Test: Existing .claude/workflow/* path allowed (plan phase)
test_workflow_relative() {
  setup
  write_state "plan"
  assert_exit "workflow_relative" 0 "Write" ".claude/workflow/state.json"
  teardown
}

# Test: docs/specs/* allowed during plan phase
test_specs_plan() {
  setup
  write_state "plan"
  assert_exit "specs_plan" 0 "Write" "docs/specs/2026-03-21-feature/spec.md"
  teardown
}

# Test: .claude/plans/* allowed during solution phase
test_plans_solution() {
  setup
  write_state "solution"
  assert_exit "plans_solution" 0 "Edit" ".claude/plans/my-plan.md"
  teardown
}

# Test: docs/plans/* allowed during plan phase
test_docs_plans() {
  setup
  write_state "plan"
  assert_exit "docs_plans" 0 "Write" "docs/plans/draft.md"
  teardown
}

# Test: docs/designs/* allowed during solution phase
test_docs_designs() {
  setup
  write_state "solution"
  assert_exit "docs_designs" 0 "Write" "docs/designs/design.md"
  teardown
}

# Test: docs/adr/* allowed during plan phase
test_docs_adr() {
  setup
  write_state "plan"
  assert_exit "docs_adr" 0 "Write" "docs/adr/0007-foo.md"
  teardown
}

# Test: Blocked path exits 2 during plan phase
test_blocked_src() {
  setup
  write_state "plan"
  assert_exit "blocked_src" 2 "Write" "crates/ecc-domain/src/lib.rs"
  teardown
}

# Test: Empty FILE_PATH exits 2 during plan phase
test_empty_path() {
  setup
  write_state "plan"
  assert_exit "empty_path" 2 "Write" ""
  teardown
}

# Test: BLOCKED error message lists all allowed paths
test_error_message() {
  setup
  write_state "plan"
  assert_stderr_contains "error_message" "docs/specs" "Write" "src/main.rs"
  teardown
}

# Test: Absolute path form also matched
test_absolute_path() {
  setup
  write_state "plan"
  assert_exit "absolute_path" 0 "Write" "/Users/dev/project/docs/specs/feature/spec.md"
  teardown
}

# Test: implement phase allows any write
test_implement_ungated() {
  setup
  write_state "implement"
  assert_exit "implement_ungated" 0 "Write" "crates/ecc-domain/src/lib.rs"
  teardown
}

# Test: done phase allows any write
test_done_ungated() {
  setup
  write_state "done"
  assert_exit "done_ungated" 0 "Write" "crates/ecc-domain/src/lib.rs"
  teardown
}

# (test_bypass removed — ECC_WORKFLOW_BYPASS=1 bypass eliminated per ADR-0056)

# Test: Missing state.json exits 0
test_no_state() {
  TEMP_DIR=$(mktemp -d)
  mkdir -p "$TEMP_DIR/.claude/workflow"
  # Do NOT write state.json
  assert_exit "no_state" 0 "Write" "crates/ecc-domain/src/lib.rs"
  rm -rf "$TEMP_DIR"
}

# Test: Malformed JSON in state.json exits 0 (fail-open)
test_malformed_json() {
  setup
  echo "this is not json" > "$TEMP_DIR/.claude/workflow/state.json"
  assert_exit "malformed_json" 0 "Write" "crates/ecc-domain/src/lib.rs"
  teardown
}

# Test: Existing docs/audits/* path still allowed
test_audits_existing() {
  setup
  write_state "plan"
  assert_exit "audits_existing" 0 "Write" "docs/audits/report.md"
  teardown
}

# Test: Existing docs/backlog/* path still allowed
test_backlog_existing() {
  setup
  write_state "plan"
  assert_exit "backlog_existing" 0 "Write" "docs/backlog/BL-001.md"
  teardown
}

# Test: Integration — workflow-init → phase-gate cycle
test_integration_cycle() {
  setup
  # Simulate workflow-init by writing a plan-phase state
  write_state "plan"
  # Verify allowed path works
  run_hook "Write" "docs/specs/2026-03-21-test/spec.md"
  local allowed_exit=$?
  # Verify blocked path works
  run_hook "Write" "src/main.rs"
  local blocked_exit=$?
  # Transition to implement phase
  write_state "implement"
  # Verify ungated
  run_hook "Write" "src/main.rs"
  local ungated_exit=$?

  if [ "$allowed_exit" -eq 0 ] && [ "$blocked_exit" -eq 2 ] && [ "$ungated_exit" -eq 0 ]; then
    echo "PASS  integration_cycle"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  integration_cycle (allowed=$allowed_exit blocked=$blocked_exit ungated=$ungated_exit)"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
  teardown
}

# Test: Path with spaces
test_path_with_spaces() {
  setup
  write_state "plan"
  assert_exit "path_with_spaces" 0 "Write" "docs/specs/my feature/spec.md"
  teardown
}

# Test: Existing docs/user-stories/* path still allowed
test_user_stories_existing() {
  setup
  write_state "plan"
  assert_exit "user_stories_existing" 0 "Write" "docs/user-stories/US-001.md"
  teardown
}

# --- Run all tests ---

echo "=== Phase-Gate Hook Tests ==="
echo ""

test_workflow_relative
test_specs_plan
test_plans_solution
test_docs_plans
test_docs_designs
test_docs_adr
test_blocked_src
test_empty_path
test_error_message
test_absolute_path
test_implement_ungated
test_done_ungated
test_no_state
test_malformed_json
test_audits_existing
test_backlog_existing
test_integration_cycle
test_path_with_spaces
test_user_stories_existing

echo ""
echo "=== Summary: $PASS_COUNT passed, $FAIL_COUNT failed ==="

if [ "$FAIL_COUNT" -gt 0 ]; then
  exit 1
fi
exit 0
