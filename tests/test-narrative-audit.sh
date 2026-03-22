#!/usr/bin/env bash
# Test suite for BL-051 explanatory narrative audit
# Run: bash tests/test-narrative-audit.sh
# Run single test: bash tests/test-narrative-audit.sh test_name
set -uo pipefail

PASS_COUNT=0
FAIL_COUNT=0
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

SKILL_FILE="$PROJECT_ROOT/skills/narrative-conventions/SKILL.md"
SPEC_DEV="$PROJECT_ROOT/commands/spec-dev.md"
SPEC_FIX="$PROJECT_ROOT/commands/spec-fix.md"
SPEC_REFACTOR="$PROJECT_ROOT/commands/spec-refactor.md"
DESIGN="$PROJECT_ROOT/commands/design.md"
IMPLEMENT="$PROJECT_ROOT/commands/implement.md"
AUDIT_FULL="$PROJECT_ROOT/commands/audit-full.md"
VERIFY="$PROJECT_ROOT/commands/verify.md"
BUILD_FIX="$PROJECT_ROOT/commands/build-fix.md"
REVIEW="$PROJECT_ROOT/commands/review.md"
CATCHUP="$PROJECT_ROOT/commands/catchup.md"
BACKLOG="$PROJECT_ROOT/commands/backlog.md"
SPEC_ROUTER="$PROJECT_ROOT/commands/spec.md"
ECC_TEST="$PROJECT_ROOT/commands/ecc-test-mode.md"
ADR_FILE="$PROJECT_ROOT/docs/adr/0011-command-narrative-convention.md"
CHANGELOG="$PROJECT_ROOT/CHANGELOG.md"
AUDIT_DOC="$PROJECT_ROOT/docs/narrative-audit.md"

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

test_skill_frontmatter() {
  echo "--- test_skill_frontmatter ---"
  assert_file_contains "skill has name field" "$SKILL_FILE" "^name: narrative-conventions"
  assert_file_contains "skill has description field" "$SKILL_FILE" "^description:"
  assert_file_contains "skill has origin field" "$SKILL_FILE" "^origin: ECC"
  assert_file_not_contains "skill has no model field" "$SKILL_FILE" "^model:"
  assert_file_not_contains "skill has no tools field" "$SKILL_FILE" "^tools:"
}

test_skill_content() {
  echo "--- test_skill_content ---"
  assert_file_contains_ci "skill has agent delegation" "$SKILL_FILE" "agent delegation"
  assert_file_contains_ci "skill has gate failure" "$SKILL_FILE" "gate failure"
  assert_file_contains_ci "skill has progress" "$SKILL_FILE" "progress"
  assert_file_contains_ci "skill has active voice" "$SKILL_FILE" "active voice"
  assert_file_contains_ci "skill has before the action" "$SKILL_FILE" "before the action"
  local word_count
  word_count="$(wc -w < "$SKILL_FILE" | tr -d ' ')"
  if [ "$word_count" -lt 500 ]; then
    echo "PASS  skill is under 500 words ($word_count)"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  skill exceeds 500 words ($word_count)"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
}

test_specdev_narrative() {
  echo "--- test_specdev_narrative ---"
  assert_file_contains "spec-dev references narrative skill" "$SPEC_DEV" "narrative-conventions"
  assert_file_contains_ci "spec-dev has tell the user which agent" "$SPEC_DEV" "tell the user which agent"
  assert_file_contains_ci "spec-dev has remediation" "$SPEC_DEV" "remediation"
  if grep -qi "searching\|queries" "$SPEC_DEV"; then
    echo "PASS  spec-dev has searching or queries (AC-002.2)"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  spec-dev missing searching or queries (AC-002.2)"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
  if grep -qi "translate\|plain language" "$SPEC_DEV"; then
    echo "PASS  spec-dev has translate or plain language (AC-002.3)"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  spec-dev missing translate or plain language (AC-002.3)"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
}

test_specfix_narrative() {
  echo "--- test_specfix_narrative ---"
  assert_file_contains "spec-fix references narrative skill" "$SPEC_FIX" "narrative-conventions"
  assert_file_contains_ci "spec-fix has tell the user" "$SPEC_FIX" "tell the user"
  assert_file_contains_ci "spec-fix has remediation" "$SPEC_FIX" "remediation"
}

test_specrefactor_narrative() {
  echo "--- test_specrefactor_narrative ---"
  assert_file_contains "spec-refactor references narrative skill" "$SPEC_REFACTOR" "narrative-conventions"
  assert_file_contains_ci "spec-refactor has tell the user" "$SPEC_REFACTOR" "tell the user"
  assert_file_contains_ci "spec-refactor has remediation" "$SPEC_REFACTOR" "remediation"
}

test_design_narrative() {
  echo "--- test_design_narrative ---"
  assert_file_contains "design references narrative skill" "$DESIGN" "narrative-conventions"
  assert_file_contains_ci "design has tell the user which validation" "$DESIGN" "tell the user which validation"
  assert_file_contains_ci "design has remediation" "$DESIGN" "remediation"
  assert_file_contains_ci "design has coverage result" "$DESIGN" "coverage result"
}

test_implement_narrative() {
  echo "--- test_implement_narrative ---"
  assert_file_contains "implement references narrative skill" "$IMPLEMENT" "narrative-conventions"
  assert_file_contains_ci "implement has tell the user PC" "$IMPLEMENT" "tell the user.*PC"
  assert_file_contains_ci "implement has re-verified" "$IMPLEMENT" "re-verified"
  assert_file_contains_ci "implement has remediation" "$IMPLEMENT" "remediation"
  assert_file_contains_ci "implement has what was found" "$IMPLEMENT" "what was found"
}

test_audit_full_narrative() {
  echo "--- test_audit_full_narrative ---"
  assert_file_contains "audit-full references narrative skill" "$AUDIT_FULL" "narrative-conventions"
  assert_file_contains_ci "audit-full has tell the user which domain" "$AUDIT_FULL" "tell the user which domain"
  assert_file_contains_ci "audit-full has completion status" "$AUDIT_FULL" "completion status"
}

test_audit_domain_narrative() {
  echo "--- test_audit_domain_narrative ---"
  local audits=("audit-archi" "audit-code" "audit-security" "audit-test" "audit-convention" "audit-errors" "audit-observability" "audit-doc" "audit-evolution")
  local found_report_ref=0
  for audit in "${audits[@]}"; do
    local f="$PROJECT_ROOT/commands/${audit}.md"
    assert_file_contains "$audit references narrative skill" "$f" "narrative-conventions"
    assert_file_contains_ci "$audit has tell the user" "$f" "tell the user"
    if grep -qi "reference.*report\|how to.*spec" "$f"; then
      found_report_ref=1
    fi
  done
  if [ "$found_report_ref" -eq 1 ]; then
    echo "PASS  at least one audit has reference report or how to spec (AC-004.3)"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  no audit has reference report or how to spec (AC-004.3)"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
}

test_verify_narrative() {
  echo "--- test_verify_narrative ---"
  assert_file_contains "verify references narrative skill" "$VERIFY" "narrative-conventions"
  assert_file_contains_ci "verify has tell the user reviewer" "$VERIFY" "tell the user.*reviewer"
  assert_file_contains_ci "verify has why both" "$VERIFY" "why both"
}

test_buildfix_narrative() {
  echo "--- test_buildfix_narrative ---"
  assert_file_contains "build-fix references narrative skill" "$BUILD_FIX" "narrative-conventions"
  if grep -qi "explain the classification\|explain.*Structural" "$BUILD_FIX"; then
    echo "PASS  build-fix has explain classification or explain Structural"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  build-fix missing explain classification or explain Structural"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
}

test_review_narrative() {
  echo "--- test_review_narrative ---"
  assert_file_contains "review references narrative skill" "$REVIEW" "narrative-conventions"
  assert_file_contains_ci "review has Programmer Oath" "$REVIEW" "Programmer.*Oath"
}

test_catchup_narrative() {
  echo "--- test_catchup_narrative ---"
  assert_file_contains "catchup references narrative skill" "$CATCHUP" "narrative-conventions"
  if grep -qi "consequences.*resetting\|consequences.*reset" "$CATCHUP"; then
    echo "PASS  catchup has consequences resetting or reset"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  catchup missing consequences resetting or reset"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
}

test_utility_narrative() {
  echo "--- test_utility_narrative ---"
  assert_file_contains "backlog references narrative skill" "$BACKLOG" "narrative-conventions"
  assert_file_contains "spec router references narrative skill" "$SPEC_ROUTER" "narrative-conventions"
  assert_file_contains "ecc-test-mode references narrative skill" "$ECC_TEST" "narrative-conventions"
}

test_adr_0011() {
  echo "--- test_adr_0011 ---"
  if test -f "$ADR_FILE"; then
    echo "PASS  ADR 0011 file exists"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  ADR 0011 file does not exist ($ADR_FILE)"
    FAIL_COUNT=$((FAIL_COUNT + 1))
    return
  fi
  assert_file_contains "ADR has Status" "$ADR_FILE" "Status"
  assert_file_contains "ADR has Context" "$ADR_FILE" "Context"
  assert_file_contains "ADR has Decision" "$ADR_FILE" "Decision"
  assert_file_contains "ADR has Consequences" "$ADR_FILE" "Consequences"
}

test_changelog_bl051() {
  echo "--- test_changelog_bl051 ---"
  if grep -qi "BL-051" "$CHANGELOG"; then
    echo "PASS  CHANGELOG has BL-051"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  CHANGELOG missing BL-051"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
}

test_audit_doc() {
  echo "--- test_audit_doc ---"
  if test -f "$AUDIT_DOC"; then
    echo "PASS  narrative-audit.md exists"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  narrative-audit.md does not exist ($AUDIT_DOC)"
    FAIL_COUNT=$((FAIL_COUNT + 1))
    return
  fi
  local found=0
  for cmd in spec-dev design implement verify audit; do
    if grep -q "$cmd" "$AUDIT_DOC"; then
      found=$((found + 1))
    fi
  done
  if [ "$found" -ge 5 ]; then
    echo "PASS  narrative-audit.md contains at least 5 command names ($found)"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  narrative-audit.md contains only $found of 5 expected command names"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
}

test_line_counts() {
  echo "--- test_line_counts ---"
  local files=(
    "$SPEC_DEV" "$SPEC_FIX" "$SPEC_REFACTOR" "$DESIGN" "$IMPLEMENT"
    "$AUDIT_FULL" "$VERIFY" "$BUILD_FIX" "$REVIEW" "$CATCHUP"
    "$BACKLOG" "$SPEC_ROUTER" "$ECC_TEST"
    "$PROJECT_ROOT/commands/audit-archi.md"
    "$PROJECT_ROOT/commands/audit-code.md"
    "$PROJECT_ROOT/commands/audit-security.md"
    "$PROJECT_ROOT/commands/audit-test.md"
    "$PROJECT_ROOT/commands/audit-convention.md"
    "$PROJECT_ROOT/commands/audit-errors.md"
    "$PROJECT_ROOT/commands/audit-observability.md"
    "$PROJECT_ROOT/commands/audit-doc.md"
    "$PROJECT_ROOT/commands/audit-evolution.md"
    "$SKILL_FILE"
  )
  for f in "${files[@]}"; do
    local lines
    lines="$(wc -l < "$f")"
    local basename
    basename="$(basename "$f")"
    if [ "$lines" -lt 800 ]; then
      echo "PASS  $basename is under 800 lines ($lines)"
      PASS_COUNT=$((PASS_COUNT + 1))
    else
      echo "FAIL  $basename exceeds 800 lines ($lines)"
      FAIL_COUNT=$((FAIL_COUNT + 1))
    fi
  done
}

test_skill_ref_consistency() {
  echo "--- test_skill_ref_consistency ---"
  local files=(
    "$SPEC_DEV" "$SPEC_FIX" "$SPEC_REFACTOR" "$DESIGN" "$IMPLEMENT"
    "$AUDIT_FULL" "$VERIFY" "$BUILD_FIX" "$REVIEW" "$CATCHUP"
    "$BACKLOG" "$SPEC_ROUTER" "$ECC_TEST"
    "$PROJECT_ROOT/commands/audit-archi.md"
    "$PROJECT_ROOT/commands/audit-code.md"
    "$PROJECT_ROOT/commands/audit-security.md"
    "$PROJECT_ROOT/commands/audit-test.md"
    "$PROJECT_ROOT/commands/audit-convention.md"
    "$PROJECT_ROOT/commands/audit-errors.md"
    "$PROJECT_ROOT/commands/audit-observability.md"
    "$PROJECT_ROOT/commands/audit-doc.md"
    "$PROJECT_ROOT/commands/audit-evolution.md"
  )
  for f in "${files[@]}"; do
    local basename
    basename="$(basename "$f")"
    assert_file_contains "$basename references narrative-conventions" "$f" "narrative-conventions"
  done
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
  test_skill_content
  test_specdev_narrative
  test_specfix_narrative
  test_specrefactor_narrative
  test_design_narrative
  test_implement_narrative
  test_audit_full_narrative
  test_audit_domain_narrative
  test_verify_narrative
  test_buildfix_narrative
  test_review_narrative
  test_catchup_narrative
  test_utility_narrative
  test_adr_0011
  test_changelog_bl051
  test_audit_doc
  test_line_counts
  test_skill_ref_consistency
fi

echo ""
echo "================================"
echo "RESULTS: $PASS_COUNT passed, $FAIL_COUNT failed"
echo "================================"

[ "$FAIL_COUNT" -eq 0 ] || exit 1
