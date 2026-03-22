#!/usr/bin/env bash
# Test suite for skills/design-an-interface/SKILL.md
# Run: bash tests/hooks/test-interface-designer.sh
# Run single test: bash tests/hooks/test-interface-designer.sh test_name
set -uo pipefail

PASS_COUNT=0
FAIL_COUNT=0
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
SKILL_FILE="$PROJECT_ROOT/skills/design-an-interface/SKILL.md"
AGENT_FILE="$PROJECT_ROOT/agents/interface-designer.md"
COMMAND_FILE="$PROJECT_ROOT/commands/design.md"

# --- Helpers ---

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

# --- Tests ---

test_skill_exists() {
  echo "--- test_skill_exists ---"
  if test -f "$SKILL_FILE"; then
    echo "PASS  skill file exists"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  skill file exists ($SKILL_FILE not found)"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
}

test_skill_frontmatter() {
  echo "--- test_skill_frontmatter ---"
  assert_file_contains "frontmatter has name field" "$SKILL_FILE" "^name: design-an-interface"
  assert_file_contains "frontmatter has description field" "$SKILL_FILE" "^description:"
  assert_file_contains "frontmatter has origin field" "$SKILL_FILE" "^origin: ECC"
}

test_skill_word_count() {
  echo "--- test_skill_word_count ---"
  local wc
  wc="$(wc -w < "$SKILL_FILE")"
  if [ "$wc" -lt 500 ]; then
    echo "PASS  skill word count under 500 (got $wc)"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  skill word count under 500 (got $wc)"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
}

test_skill_triggers() {
  echo "--- test_skill_triggers ---"
  assert_file_contains "trigger: design an interface" "$SKILL_FILE" "design an interface"
  assert_file_contains "trigger: design it twice" "$SKILL_FILE" "design it twice"
  assert_file_contains "trigger: explore interface options" "$SKILL_FILE" "explore interface options"
  assert_file_contains "trigger: compare API shapes" "$SKILL_FILE" "compare API shapes"
  assert_file_contains "trigger: what should the port look like" "$SKILL_FILE" "what should the port look like"
}

test_skill_constraints() {
  echo "--- test_skill_constraints ---"
  local count=0
  grep -q "minimize method count" "$SKILL_FILE" && count=$((count + 1))
  grep -q "maximize flexibility" "$SKILL_FILE" && count=$((count + 1))
  grep -qi "optimize.*common case" "$SKILL_FILE" && count=$((count + 1))
  grep -qi "named paradigm" "$SKILL_FILE" && count=$((count + 1))
  if [ "$count" -ge 4 ]; then
    echo "PASS  skill has >= 4 constraints (found $count)"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  skill has >= 4 constraints (found $count, expected >= 4)"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
}

test_skill_dimensions() {
  echo "--- test_skill_dimensions ---"
  local count=0
  grep -qi "simplicity" "$SKILL_FILE" && count=$((count + 1))
  grep -qiE "general.purpose.*specialized" "$SKILL_FILE" && count=$((count + 1))
  grep -qi "implementation efficiency" "$SKILL_FILE" && count=$((count + 1))
  grep -qi "depth" "$SKILL_FILE" && count=$((count + 1))
  grep -qi "ease of correct use" "$SKILL_FILE" && count=$((count + 1))
  if [ "$count" -ge 5 ]; then
    echo "PASS  skill has >= 5 dimensions (found $count)"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  skill has >= 5 dimensions (found $count, expected >= 5)"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
}

test_skill_anti_patterns() {
  echo "--- test_skill_anti_patterns ---"
  local count=0
  grep -q "DO NOT.*similar" "$SKILL_FILE" && count=$((count + 1))
  grep -q "DO NOT.*skip" "$SKILL_FILE" && count=$((count + 1))
  grep -q "DO NOT.*implement" "$SKILL_FILE" && count=$((count + 1))
  if [ "$count" -ge 3 ]; then
    echo "PASS  skill has >= 3 anti-patterns (found $count)"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  skill has >= 3 anti-patterns (found $count, expected >= 3)"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
}

test_skill_agent_reference() {
  echo "--- test_skill_agent_reference ---"
  assert_file_contains "skill references interface-designer agent" "$SKILL_FILE" "interface-designer"
}

# --- Run tests ---

run_tests() {
  echo "=== Interface Designer Skill Tests ==="
  echo ""

  if [ -n "${1:-}" ]; then
    # Run a single named test
    "$1"
  fi
  if [ -z "${1:-}" ]; then
    test_skill_exists
    test_skill_frontmatter
    test_skill_word_count
    test_skill_triggers
    test_skill_constraints
    test_skill_dimensions
    test_skill_anti_patterns
    test_skill_agent_reference
  fi

  echo ""
  echo "=== Summary: $PASS_COUNT passed, $FAIL_COUNT failed ==="

  if [ "$FAIL_COUNT" -gt 0 ]; then
    exit 1
  fi
  exit 0
}

run_tests "${1:-}"
