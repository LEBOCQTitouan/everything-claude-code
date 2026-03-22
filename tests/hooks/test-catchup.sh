#!/usr/bin/env bash
# Test suite for commands/catchup.md
# Run: bash tests/hooks/test-catchup.sh
# Run single test: bash tests/hooks/test-catchup.sh test_name
set -uo pipefail

PASS_COUNT=0
FAIL_COUNT=0
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
COMMAND_FILE="$PROJECT_ROOT/commands/catchup.md"

# --- Helpers ---

assert_contains() {
  local test_name="$1"
  local haystack="$2"
  local needle="$3"

  if echo "$haystack" | grep -q "$needle"; then
    echo "PASS  $test_name"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  $test_name (output did not contain '$needle')"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
}

assert_not_contains() {
  local test_name="$1"
  local haystack="$2"
  local needle="$3"

  if echo "$haystack" | grep -q "$needle"; then
    echo "FAIL  $test_name (output contained '$needle' but should not)"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  else
    echo "PASS  $test_name"
    PASS_COUNT=$((PASS_COUNT + 1))
  fi
}

assert_file_contains() {
  local test_name="$1"
  local file="$2"
  local pattern="$3"

  if grep -q "$pattern" "$file"; then
    echo "PASS  $test_name"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  $test_name ($file did not contain '$pattern')"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
}

assert_file_not_contains() {
  local test_name="$1"
  local file="$2"
  local pattern="$3"

  if grep -q "$pattern" "$file"; then
    echo "FAIL  $test_name ($file contained '$pattern' but should not)"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  else
    echo "PASS  $test_name"
    PASS_COUNT=$((PASS_COUNT + 1))
  fi
}

# --- Tests ---

# (Test functions will be added in subsequent PCs)

test_git_uncommitted() {
  echo "--- test_git_uncommitted ---"
  assert_file_contains "mentions git status" "$COMMAND_FILE" "git status --short"
  assert_file_contains "reports modified files" "$COMMAND_FILE" "modified"
  assert_file_contains "reports untracked files" "$COMMAND_FILE" "untracked"
  assert_file_contains "reports staged files" "$COMMAND_FILE" "staged"
}

test_malformed_json() {
  echo "--- test_malformed_json ---"
  assert_file_contains "warns about malformed state" "$COMMAND_FILE" "state\.json is malformed"
  assert_file_contains "continues after malformed json" "$COMMAND_FILE" "continue"
}

test_spec_design_paths() {
  echo "--- test_spec_design_paths ---"
  assert_file_contains "displays spec_path" "$COMMAND_FILE" "spec_path"
  assert_file_contains "displays design_path" "$COMMAND_FILE" "design_path"
  assert_file_contains "displays artifacts.spec_path" "$COMMAND_FILE" "artifacts\.spec_path"
  assert_file_contains "displays artifacts.design_path" "$COMMAND_FILE" "artifacts\.design_path"
}

test_workflow_done() {
  echo "--- test_workflow_done ---"
  assert_file_contains "shows workflow complete" "$COMMAND_FILE" "Workflow complete"
  assert_file_contains "handles done phase" "$COMMAND_FILE" "done"
}

test_no_workflow() {
  echo "--- test_no_workflow ---"
  assert_file_contains "reports no active workflow" "$COMMAND_FILE" "No active workflow"
  assert_file_contains "continues to git and memory" "$COMMAND_FILE" "Git Status"
}

test_tasks_missing() {
  echo "--- test_tasks_missing ---"
  assert_file_contains "reports tasks.md not found" "$COMMAND_FILE" "tasks\.md not found"
  assert_file_contains "shows path when tasks missing" "$COMMAND_FILE" "tasks_path"
}

test_tasks_progress() {
  echo "--- test_tasks_progress ---"
  assert_file_contains "has Tasks Progress section" "$COMMAND_FILE" "## Tasks Progress"
  assert_file_contains "reads tasks_path from state" "$COMMAND_FILE" "tasks_path"
  assert_file_contains "reads tasks.md" "$COMMAND_FILE" "tasks\.md"
  assert_file_contains "counts completed tasks" "$COMMAND_FILE" "\[x\]"
  assert_file_contains "detects pending tasks" "$COMMAND_FILE" "pending"
  assert_file_contains "detects failed tasks" "$COMMAND_FILE" "failed"
  assert_file_contains "shows total PCs" "$COMMAND_FILE" "total"
  assert_file_contains "detects in-progress tasks" "$COMMAND_FILE" "in-progress"
}

test_workflow_active_state() {
  echo "--- test_workflow_active_state ---"
  assert_file_contains "has Workflow State section" "$COMMAND_FILE" "## Workflow State"
  assert_file_contains "reads state.json" "$COMMAND_FILE" "state\.json"
  assert_file_contains "displays phase" "$COMMAND_FILE" "phase"
  assert_file_contains "displays feature" "$COMMAND_FILE" "feature"
  assert_file_contains "displays concern" "$COMMAND_FILE" "concern"
  assert_file_contains "displays started_at" "$COMMAND_FILE" "started_at"
  assert_file_contains "checks artifact timestamps" "$COMMAND_FILE" "plan"
  assert_file_contains "checks solution artifact" "$COMMAND_FILE" "solution"
  assert_file_contains "checks implement artifact" "$COMMAND_FILE" "implement"
}

# --- Run tests ---

run_tests() {
  echo "=== Catchup Command Tests ==="
  echo ""

  if [ -n "${1:-}" ]; then
    # Run a single named test
    "$1"
  fi
  if [ -z "${1:-}" ]; then
    test_workflow_active_state
    test_tasks_progress
    test_tasks_missing
    test_no_workflow
    test_workflow_done
    test_spec_design_paths
    test_malformed_json
    test_git_uncommitted
  fi

  echo ""
  echo "=== Summary: $PASS_COUNT passed, $FAIL_COUNT failed ==="

  if [ "$FAIL_COUNT" -gt 0 ]; then
    exit 1
  fi
  exit 0
}

run_tests "${1:-}"
