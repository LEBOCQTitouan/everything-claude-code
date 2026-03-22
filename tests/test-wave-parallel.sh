#!/usr/bin/env bash
# Test suite for BL-032 wave-based parallel TDD execution
# Run: bash tests/test-wave-parallel.sh
# Run single test: bash tests/test-wave-parallel.sh test_name
set -uo pipefail

PASS_COUNT=0
FAIL_COUNT=0
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

IMPLEMENT_FILE="$PROJECT_ROOT/commands/implement.md"
ADR_FILE="$PROJECT_ROOT/docs/adr/0012-wave-parallel-tdd.md"
GLOSSARY_FILE="$PROJECT_ROOT/docs/domain/glossary.md"
CHANGELOG_FILE="$PROJECT_ROOT/CHANGELOG.md"

# --- Helpers ---

assert_file_contains_ci() {
  local test_name="$1"
  local file="$2"
  local pattern="$3"

  if grep -qi "$pattern" "$file"; then
    echo "PASS  $test_name"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  $test_name ($file did not contain '$pattern' case-insensitive)"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
}

# --- Tests ---

test_wave_analysis_section() {
  echo "--- test_wave_analysis_section ---"
  assert_file_contains_ci "implement.md has Wave Analysis" "$IMPLEMENT_FILE" "wave analysis"
}

test_wave_grouping_algorithm() {
  echo "--- test_wave_grouping_algorithm ---"
  assert_file_contains_ci "implement.md has left-to-right" "$IMPLEMENT_FILE" "left-to-right"
  assert_file_contains_ci "implement.md has adjacent" "$IMPLEMENT_FILE" "adjacent"
  assert_file_contains_ci "implement.md has file overlap" "$IMPLEMENT_FILE" "file overlap"
}

test_max_concurrency_cap() {
  echo "--- test_max_concurrency_cap ---"
  if grep -qiE "max.*4|maximum.*4|cap.*4" "$IMPLEMENT_FILE"; then
    echo "PASS  implement.md has max/maximum/cap 4"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  implement.md missing max/maximum/cap 4"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
}

test_wave_plan_display() {
  echo "--- test_wave_plan_display ---"
  assert_file_contains_ci "implement.md has wave plan" "$IMPLEMENT_FILE" "wave plan"
  if grep -qiE "display|show" "$IMPLEMENT_FILE"; then
    echo "PASS  implement.md has display or show"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  implement.md missing display or show"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
}

test_degenerate_cases() {
  echo "--- test_degenerate_cases ---"
  if grep -qiE "degenerate|all.*overlap" "$IMPLEMENT_FILE"; then
    echo "PASS  implement.md has degenerate or all overlap"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  implement.md missing degenerate or all overlap"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
  if grep -qiE "single.*PC|single-PC" "$IMPLEMENT_FILE"; then
    echo "PASS  implement.md has single PC or single-PC"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  implement.md missing single PC or single-PC"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
}

test_worktree_dispatch() {
  echo "--- test_worktree_dispatch ---"
  if grep -qiE "isolation.*worktree|worktree" "$IMPLEMENT_FILE"; then
    echo "PASS  implement.md has isolation worktree or worktree"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  implement.md missing isolation worktree or worktree"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
}

test_sequential_merge() {
  echo "--- test_sequential_merge ---"
  assert_file_contains_ci "implement.md has merge" "$IMPLEMENT_FILE" "merge"
  if grep -qiE "PC-ID order|sequential" "$IMPLEMENT_FILE"; then
    echo "PASS  implement.md has PC-ID order or sequential"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  implement.md missing PC-ID order or sequential"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
}

test_single_pc_backward_compat() {
  echo "--- test_single_pc_backward_compat ---"
  if grep -qiE "single.*PC" "$IMPLEMENT_FILE"; then
    echo "PASS  implement.md has single PC"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  implement.md missing single PC"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
  if grep -qiE "backward|identical|current" "$IMPLEMENT_FILE"; then
    echo "PASS  implement.md has backward or identical or current"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  implement.md missing backward or identical or current"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
}

test_prior_results_scoping() {
  echo "--- test_prior_results_scoping ---"
  assert_file_contains_ci "implement.md has Prior PC Results" "$IMPLEMENT_FILE" "Prior PC Results"
  if grep -qiE "prior waves|completed waves" "$IMPLEMENT_FILE"; then
    echo "PASS  implement.md has prior waves or completed waves"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  implement.md missing prior waves or completed waves"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
}

test_merge_error_handling() {
  echo "--- test_merge_error_handling ---"
  assert_file_contains_ci "implement.md has merge conflict" "$IMPLEMENT_FILE" "merge conflict"
  assert_file_contains_ci "implement.md has STOP" "$IMPLEMENT_FILE" "STOP"
}

test_wave_regression() {
  echo "--- test_wave_regression ---"
  assert_file_contains_ci "implement.md has wave" "$IMPLEMENT_FILE" "wave"
  assert_file_contains_ci "implement.md has regression" "$IMPLEMENT_FILE" "regression"
  if grep -qiE "waves 1\.\.W|all.*PC commands" "$IMPLEMENT_FILE"; then
    echo "PASS  implement.md has waves 1..W or all PC commands"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  implement.md missing waves 1..W or all PC commands"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
}

test_failure_semantics() {
  echo "--- test_failure_semantics ---"
  if grep -qiE "wave finish|let.*finish" "$IMPLEMENT_FILE"; then
    echo "PASS  implement.md has wave finish or let finish"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  implement.md missing wave finish or let finish"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
  assert_file_contains_ci "implement.md has failed PC" "$IMPLEMENT_FILE" "failed PC"
}

test_reentry_wave_aware() {
  echo "--- test_reentry_wave_aware ---"
  assert_file_contains_ci "implement.md has re-entry" "$IMPLEMENT_FILE" "re-entry"
  assert_file_contains_ci "implement.md has wave (for re-entry)" "$IMPLEMENT_FILE" "wave"
  if grep -qiE "re-dispatch|re-derive" "$IMPLEMENT_FILE"; then
    echo "PASS  implement.md has re-dispatch or re-derive"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  implement.md missing re-dispatch or re-derive"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
}

test_git_tags() {
  echo "--- test_git_tags ---"
  if grep -qiE "wave-N-start|git tag" "$IMPLEMENT_FILE"; then
    echo "PASS  implement.md has wave-N-start or git tag"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  implement.md missing wave-N-start or git tag"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
}

test_wave_tasks_tracking() {
  echo "--- test_wave_tasks_tracking ---"
  assert_file_contains_ci "implement.md has red@" "$IMPLEMENT_FILE" "red@"
  assert_file_contains_ci "implement.md has wave (for tracking)" "$IMPLEMENT_FILE" "wave"
  assert_file_contains_ci "implement.md has done@" "$IMPLEMENT_FILE" "done@"
}

test_implement_done_wave_column() {
  echo "--- test_implement_done_wave_column ---"
  if grep -qiE "Wave column|Wave.*column" "$IMPLEMENT_FILE"; then
    echo "PASS  implement.md has Wave column"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  implement.md missing Wave column"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
}

test_line_count() {
  echo "--- test_line_count ---"
  local lines
  lines="$(wc -l < "$IMPLEMENT_FILE")"
  if [ "$lines" -lt 800 ]; then
    echo "PASS  implement.md is under 800 lines ($lines)"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  implement.md exceeds 800 lines ($lines)"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
}

test_adr_0012() {
  echo "--- test_adr_0012 ---"
  if test -f "$ADR_FILE"; then
    echo "PASS  ADR 0012 file exists"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  ADR 0012 file does not exist ($ADR_FILE)"
    FAIL_COUNT=$((FAIL_COUNT + 1))
    return
  fi
  assert_file_contains_ci "ADR has Status" "$ADR_FILE" "Status"
  assert_file_contains_ci "ADR has Context" "$ADR_FILE" "Context"
  assert_file_contains_ci "ADR has Decision" "$ADR_FILE" "Decision"
  assert_file_contains_ci "ADR has Consequences" "$ADR_FILE" "Consequences"
}

test_glossary_terms() {
  echo "--- test_glossary_terms ---"
  assert_file_contains_ci "glossary has Wave" "$GLOSSARY_FILE" "### Wave"
  assert_file_contains_ci "glossary has Wave Plan" "$GLOSSARY_FILE" "### Wave Plan"
}

test_changelog_entry() {
  echo "--- test_changelog_entry ---"
  if grep -qi "BL-032" "$CHANGELOG_FILE"; then
    echo "PASS  CHANGELOG has BL-032"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  CHANGELOG missing BL-032"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
}

# --- Run tests ---

if [ -n "${1:-}" ]; then
  if declare -f "$1" > /dev/null 2>&1; then
    "$1"
  else
    echo "Unknown test: $1"
    exit 1
  fi
else
  # Run all tests
  test_wave_analysis_section
  test_wave_grouping_algorithm
  test_max_concurrency_cap
  test_wave_plan_display
  test_degenerate_cases
  test_worktree_dispatch
  test_sequential_merge
  test_single_pc_backward_compat
  test_prior_results_scoping
  test_merge_error_handling
  test_wave_regression
  test_failure_semantics
  test_reentry_wave_aware
  test_git_tags
  test_wave_tasks_tracking
  test_implement_done_wave_column
  test_line_count
  test_adr_0012
  test_glossary_terms
  test_changelog_entry

  # PC-021: Run pipeline summaries test for backward compat
  echo ""
  echo "--- PC-021: Running test-pipeline-summaries.sh ---"
  bash "$SCRIPT_DIR/test-pipeline-summaries.sh"
fi

echo ""
echo "================================"
echo "RESULTS: $PASS_COUNT passed, $FAIL_COUNT failed"
echo "================================"

[ "$FAIL_COUNT" -eq 0 ] || exit 1
