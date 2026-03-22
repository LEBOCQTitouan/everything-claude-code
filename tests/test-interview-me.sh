#!/usr/bin/env bash
# Test suite for interview-me skill (PC-005 through PC-033)
# Run: bash tests/test-interview-me.sh
# Run single test: bash tests/test-interview-me.sh test_name
set -uo pipefail

PASS_COUNT=0
FAIL_COUNT=0
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

SKILL_FILE="$PROJECT_ROOT/skills/interview-me/SKILL.md"
AGENT_FILE="$PROJECT_ROOT/agents/interviewer.md"
GRILL_ME_FILE="$PROJECT_ROOT/skills/grill-me/SKILL.md"
GLOSSARY_FILE="$PROJECT_ROOT/docs/domain/glossary.md"
CHANGELOG_FILE="$PROJECT_ROOT/CHANGELOG.md"
ADR_FILE="$PROJECT_ROOT/docs/adr/0010-skill-frontmatter-validation.md"

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

# --- Skill Tests (PC-005–011) ---

test_skill_frontmatter() {
  echo "--- test_skill_frontmatter ---"
  assert_file_contains "skill has name field" "$SKILL_FILE" "^name: interview-me"
  assert_file_contains "skill has description field" "$SKILL_FILE" "^description:"
  assert_file_contains "skill has origin field" "$SKILL_FILE" "^origin: ECC"
  assert_file_not_contains "skill has no model field" "$SKILL_FILE" "^model:"
  assert_file_not_contains "skill has no tools field" "$SKILL_FILE" "^tools:"
}

test_skill_word_count() {
  echo "--- test_skill_word_count ---"
  local wc
  wc=$(wc -w < "$SKILL_FILE")
  if [ "$wc" -lt 500 ]; then
    echo "PASS  skill is under 500 words ($wc)"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  skill exceeds 500 words ($wc)"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
}

test_skill_triggers() {
  echo "--- test_skill_triggers ---"
  assert_file_contains "skill has 'interview me' trigger" "$SKILL_FILE" "interview me"
  assert_file_contains "skill has 'help me think through' trigger" "$SKILL_FILE" "help me think through"
  assert_file_contains "skill has 'extract requirements' trigger" "$SKILL_FILE" "extract requirements"
  assert_file_contains "skill has 'what should I consider' trigger" "$SKILL_FILE" "what should I consider"
}

test_skill_stages() {
  echo "--- test_skill_stages ---"
  local stages=("current state" "desired state" "constraints" "security checkpoint" "stakeholders" "dependencies" "prior art" "failure modes")
  for stage in "${stages[@]}"; do
    if grep -qi "$stage" "$SKILL_FILE"; then
      echo "PASS  skill has stage '$stage'"
      PASS_COUNT=$((PASS_COUNT + 1))
    else
      echo "FAIL  skill missing stage '$stage'"
      FAIL_COUNT=$((FAIL_COUNT + 1))
    fi
  done
}

test_skill_output_format() {
  echo "--- test_skill_output_format ---"
  assert_file_contains "skill has docs/interviews/ path" "$SKILL_FILE" "docs/interviews/"
  assert_file_contains "skill has topic in output" "$SKILL_FILE" "topic"
  assert_file_contains "skill has date in output" "$SKILL_FILE" "date"
}

test_skill_negative_examples() {
  echo "--- test_skill_negative_examples ---"
  assert_file_contains "skill has DO NOT negative example" "$SKILL_FILE" "DO NOT"
}

test_skill_distinct_from_grill_me() {
  echo "--- test_skill_distinct_from_grill_me ---"
  # Grill-me stage names that should NOT appear as stage names in interview-me
  local grill_me_stages=("Problem" "Edge Cases" "Scope" "Rollback" "Success Criteria")
  for stage in "${grill_me_stages[@]}"; do
    # Check that the stage name does not appear as a ### Stage heading in interview-me
    if grep -q "^### Stage.*$stage" "$SKILL_FILE"; then
      echo "FAIL  interview-me should not have grill-me stage name '$stage' as a stage heading"
      FAIL_COUNT=$((FAIL_COUNT + 1))
    else
      echo "PASS  interview-me does not use grill-me stage name '$stage'"
      PASS_COUNT=$((PASS_COUNT + 1))
    fi
  done
}

# --- Agent Tests (PC-012–020) ---

test_agent_frontmatter() {
  echo "--- test_agent_frontmatter ---"
  assert_file_contains "agent has name field" "$AGENT_FILE" "^name: interviewer"
  assert_file_contains "agent has model opus" "$AGENT_FILE" "^model: opus"
  assert_file_contains "agent has interview-me skill" "$AGENT_FILE" "skills:.*interview-me"
}

test_agent_codebase_exploration() {
  echo "--- test_agent_codebase_exploration ---"
  assert_file_contains "agent has Phase 1" "$AGENT_FILE" "Phase 1"
  if grep -qi "codebase\|exploration\|Explore" "$AGENT_FILE"; then
    echo "PASS  agent has codebase/exploration/Explore"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  agent missing codebase/exploration/Explore"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
  if grep -qi "empty repo" "$AGENT_FILE"; then
    echo "PASS  agent mentions empty repo fallback"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  agent missing empty repo fallback"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
}

test_agent_skip_known() {
  echo "--- test_agent_skip_known ---"
  assert_file_contains "agent has skip" "$AGENT_FILE" "skip"
  if grep -qi "already known\|already evident\|already knows" "$AGENT_FILE"; then
    echo "PASS  agent has already known/evident/knows"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  agent missing already known/evident/knows"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
}

test_agent_security_gate() {
  echo "--- test_agent_security_gate ---"
  assert_file_contains "agent has security" "$AGENT_FILE" "security"
  if grep -qi "hard-block\|hard block\|refuses to proceed\|MUST NOT proceed" "$AGENT_FILE"; then
    echo "PASS  agent has hard-block/refuses to proceed/MUST NOT proceed"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  agent missing hard-block/refuses to proceed/MUST NOT proceed"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
}

test_agent_output_path() {
  echo "--- test_agent_output_path ---"
  assert_file_contains "agent has docs/interviews/ path" "$AGENT_FILE" "docs/interviews/"
}

test_agent_one_question_per_turn() {
  echo "--- test_agent_one_question_per_turn ---"
  if grep -qi "one question\|one per turn" "$AGENT_FILE"; then
    echo "PASS  agent has one question/one per turn"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  agent missing one question/one per turn"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
  assert_file_contains "agent has AskUserQuestion" "$AGENT_FILE" "AskUserQuestion"
}

test_agent_todowrite() {
  echo "--- test_agent_todowrite ---"
  assert_file_contains "agent has TodoWrite" "$AGENT_FILE" "TodoWrite"
  if grep -qi "unavailable\|graceful" "$AGENT_FILE"; then
    echo "PASS  agent has graceful degradation for TodoWrite"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  agent missing graceful degradation for TodoWrite"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
}

test_agent_early_exit() {
  echo "--- test_agent_early_exit ---"
  assert_file_contains "agent has early" "$AGENT_FILE" "early"
  if grep -qi "Stages completed\|partial" "$AGENT_FILE"; then
    echo "PASS  agent has Stages completed/partial"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  agent missing Stages completed/partial"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
}

test_agent_numeric_suffix() {
  echo "--- test_agent_numeric_suffix ---"
  if grep -qi "numeric suffix\|-2\.md" "$AGENT_FILE"; then
    echo "PASS  agent has numeric suffix/-2.md"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  agent missing numeric suffix/-2.md"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
}

# --- Doc Tests (PC-027–029) ---

test_glossary_entries() {
  echo "--- test_glossary_entries ---"
  assert_file_contains "glossary has Interview Me" "$GLOSSARY_FILE" "Interview Me"
  assert_file_contains "glossary has Interviewer" "$GLOSSARY_FILE" "Interviewer"
}

test_changelog_entry() {
  echo "--- test_changelog_entry ---"
  if grep -qi "BL-013" "$CHANGELOG_FILE"; then
    echo "PASS  CHANGELOG has BL-013"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  CHANGELOG missing BL-013"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
}

test_adr_exists() {
  echo "--- test_adr_exists ---"
  if test -f "$ADR_FILE"; then
    echo "PASS  ADR 0010 file exists"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  ADR 0010 file does not exist ($ADR_FILE)"
    FAIL_COUNT=$((FAIL_COUNT + 1))
    return
  fi
  assert_file_contains "ADR has Status" "$ADR_FILE" "Status"
  assert_file_contains "ADR has Context" "$ADR_FILE" "Context"
  assert_file_contains "ADR has Decision" "$ADR_FILE" "Decision"
  assert_file_contains "ADR has Consequences" "$ADR_FILE" "Consequences"
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
  test_skill_frontmatter
  test_skill_word_count
  test_skill_triggers
  test_skill_stages
  test_skill_output_format
  test_skill_negative_examples
  test_skill_distinct_from_grill_me
  test_agent_frontmatter
  test_agent_codebase_exploration
  test_agent_skip_known
  test_agent_security_gate
  test_agent_output_path
  test_agent_one_question_per_turn
  test_agent_todowrite
  test_agent_early_exit
  test_agent_numeric_suffix
  test_glossary_entries
  test_changelog_entry
  test_adr_exists
fi

echo ""
echo "================================"
echo "RESULTS: $PASS_COUNT passed, $FAIL_COUNT failed"
echo "================================"

[ "$FAIL_COUNT" -eq 0 ] || exit 1
