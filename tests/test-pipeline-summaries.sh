#!/usr/bin/env bash
# Test suite for BL-048 pipeline output summaries
# Run: bash tests/test-pipeline-summaries.sh
# Run single test: bash tests/test-pipeline-summaries.sh test_name
set -uo pipefail

PASS_COUNT=0
FAIL_COUNT=0
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

SKILL_FILE="$PROJECT_ROOT/skills/spec-pipeline-shared/SKILL.md"
SPEC_DEV_FILE="$PROJECT_ROOT/commands/spec-dev.md"
SPEC_FIX_FILE="$PROJECT_ROOT/commands/spec-fix.md"
SPEC_REFACTOR_FILE="$PROJECT_ROOT/commands/spec-refactor.md"
DESIGN_FILE="$PROJECT_ROOT/commands/design.md"
IMPLEMENT_FILE="$PROJECT_ROOT/commands/implement.md"
ADR_FILE="$PROJECT_ROOT/docs/adr/0009-phase-summary-convention.md"
CHANGELOG_FILE="$PROJECT_ROOT/CHANGELOG.md"
BACKLOG_DIR="$PROJECT_ROOT/docs/backlog"

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

# --- Tests ---

test_skill_frontmatter() {
  echo "--- test_skill_frontmatter ---"
  assert_file_contains "skill has name field" "$SKILL_FILE" "^name: spec-pipeline-shared"
  assert_file_contains "skill has description field" "$SKILL_FILE" "^description:"
  assert_file_contains "skill has origin field" "$SKILL_FILE" "^origin: ECC"
}

test_skill_project_detection() {
  echo "--- test_skill_project_detection ---"
  if grep -qi "Project Detection" "$SKILL_FILE"; then
    echo "PASS  skill has Project Detection section"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  skill missing Project Detection section"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
}

test_skill_grillme_rules() {
  echo "--- test_skill_grillme_rules ---"
  assert_file_contains "skill has Grill-Me" "$SKILL_FILE" "Grill-Me"
  if grep -qi "Interview\|Rules" "$SKILL_FILE"; then
    echo "PASS  skill has Interview or Rules"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  skill missing Interview or Rules"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
}

test_skill_adversarial_schema() {
  echo "--- test_skill_adversarial_schema ---"
  assert_file_contains "skill has Adversarial Review" "$SKILL_FILE" "Adversarial Review"
  if grep -qi "Spec Output Schema\|Output Schema" "$SKILL_FILE"; then
    echo "PASS  skill has Spec Output Schema or Output Schema"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  skill missing Spec Output Schema or Output Schema"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
}

test_specdev_dry_ref() {
  echo "--- test_specdev_dry_ref ---"
  assert_file_contains "spec-dev references shared skill" "$SPEC_DEV_FILE" "spec-pipeline-shared"
}

test_specfix_dry_ref() {
  echo "--- test_specfix_dry_ref ---"
  assert_file_contains "spec-fix references shared skill" "$SPEC_FIX_FILE" "spec-pipeline-shared"
}

test_specrefactor_dry_ref() {
  echo "--- test_specrefactor_dry_ref ---"
  assert_file_contains "spec-refactor references shared skill" "$SPEC_REFACTOR_FILE" "spec-pipeline-shared"
}

test_specdev_grillme_accumulator() {
  echo "--- test_specdev_grillme_accumulator ---"
  if grep -qi "accumulate" "$SPEC_DEV_FILE"; then
    echo "PASS  spec-dev has accumulate"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  spec-dev missing accumulate"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
}

test_specdev_grillme_table() {
  echo "--- test_specdev_grillme_table ---"
  if grep -q "Question" "$SPEC_DEV_FILE" && grep -q "Answer" "$SPEC_DEV_FILE" && grep -q "Source" "$SPEC_DEV_FILE"; then
    echo "PASS  spec-dev has Question/Answer/Source table headers"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  spec-dev missing Question/Answer/Source table headers"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
}

test_specdev_us_ac_tables() {
  echo "--- test_specdev_us_ac_tables ---"
  assert_file_contains "spec-dev has User Stories table" "$SPEC_DEV_FILE" "ID.*Title.*AC Count"
  assert_file_contains "spec-dev has AC table" "$SPEC_DEV_FILE" "AC ID.*Description"
}

test_specdev_adversary_table() {
  echo "--- test_specdev_adversary_table ---"
  assert_file_contains "spec-dev has Adversary Findings table" "$SPEC_DEV_FILE" "Dimension.*Verdict"
}

test_specdev_artifacts_table() {
  echo "--- test_specdev_artifacts_table ---"
  assert_file_contains "spec-dev has Artifacts Persisted table" "$SPEC_DEV_FILE" "File Path.*Section Written"
}

test_specdev_phase_summary() {
  echo "--- test_specdev_phase_summary ---"
  assert_file_contains "spec-dev has Phase Summary" "$SPEC_DEV_FILE" "Phase Summary"
  if grep -qi "spec\.md\|spec file" "$SPEC_DEV_FILE"; then
    echo "PASS  spec-dev Phase Summary references spec.md or spec file"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  spec-dev Phase Summary missing spec.md or spec file reference"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
}

test_specfix_variant_tables() {
  echo "--- test_specfix_variant_tables ---"
  assert_file_contains "spec-fix has Adversary table" "$SPEC_FIX_FILE" "Dimension.*Verdict"
  if grep -qi "root cause" "$SPEC_FIX_FILE"; then
    echo "PASS  spec-fix has root cause"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  spec-fix missing root cause"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
}

test_specrefactor_variant_tables() {
  echo "--- test_specrefactor_variant_tables ---"
  assert_file_contains "spec-refactor has Adversary table" "$SPEC_REFACTOR_FILE" "Dimension.*Verdict"
  if grep -qi "smells" "$SPEC_REFACTOR_FILE"; then
    echo "PASS  spec-refactor has smells"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  spec-refactor missing smells"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
}

test_design_reviews_table() {
  echo "--- test_design_reviews_table ---"
  if grep -q "Review Type.*Verdict" "$DESIGN_FILE" || (grep -q "SOLID" "$DESIGN_FILE" && grep -q "Robert" "$DESIGN_FILE" && grep -q "Security" "$DESIGN_FILE"); then
    echo "PASS  design has reviews table"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  design missing reviews table"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
}

test_design_adversary_table() {
  echo "--- test_design_adversary_table ---"
  assert_file_contains "design has Adversary table" "$DESIGN_FILE" "Dimension.*Verdict"
}

test_design_filechanges_table() {
  echo "--- test_design_filechanges_table ---"
  assert_file_contains "design has File/Action/Spec Ref table" "$DESIGN_FILE" "File.*Action.*Spec Ref"
  assert_file_contains "design has Phase Summary" "$DESIGN_FILE" "Phase Summary"
}

test_design_artifacts_table() {
  echo "--- test_design_artifacts_table ---"
  assert_file_contains "design has Artifacts Persisted table" "$DESIGN_FILE" "File Path.*Section Written"
}

test_design_phase_summary() {
  echo "--- test_design_phase_summary ---"
  assert_file_contains "design has Phase Summary" "$DESIGN_FILE" "Phase Summary"
  if grep -qi "design\.md\|design file" "$DESIGN_FILE"; then
    echo "PASS  design Phase Summary references design.md or design file"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  design Phase Summary missing design.md or design file reference"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
}

test_implement_tasks_table() {
  echo "--- test_implement_tasks_table ---"
  assert_file_contains "implement has PC ID/Description table" "$IMPLEMENT_FILE" "PC ID.*Description"
  if grep -qE "RED-GREEN|Status|Commit Count" "$IMPLEMENT_FILE"; then
    echo "PASS  implement tasks table has RED-GREEN or Status or Commit Count"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  implement tasks table missing RED-GREEN/Status/Commit Count"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
}

test_implement_commits_table() {
  echo "--- test_implement_commits_table ---"
  if grep -qE "Hash.*Message|SHA.*Message" "$IMPLEMENT_FILE"; then
    echo "PASS  implement has commits table"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  implement missing commits table (Hash/SHA.*Message)"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
}

test_implement_docs_table() {
  echo "--- test_implement_docs_table ---"
  assert_file_contains "implement has docs table" "$IMPLEMENT_FILE" "Doc File.*Level.*What Changed"
}

test_implement_artifacts_table() {
  echo "--- test_implement_artifacts_table ---"
  assert_file_contains "implement has Artifacts Persisted table" "$IMPLEMENT_FILE" "File Path.*Section Written"
}

test_implement_commit_accumulator() {
  echo "--- test_implement_commit_accumulator ---"
  if grep -qi "accumulate" "$IMPLEMENT_FILE"; then
    echo "PASS  implement has accumulate"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  implement missing accumulate"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
  assert_file_contains "implement has commit" "$IMPLEMENT_FILE" "commit"
  assert_file_contains "implement has SHA" "$IMPLEMENT_FILE" "SHA"
}

test_implement_phase_summary() {
  echo "--- test_implement_phase_summary ---"
  assert_file_contains "implement has Phase Summary" "$IMPLEMENT_FILE" "Phase Summary"
  assert_file_contains "implement Phase Summary references tasks.md" "$IMPLEMENT_FILE" "tasks.md"
}

test_adr_0009() {
  echo "--- test_adr_0009 ---"
  if test -f "$ADR_FILE"; then
    echo "PASS  ADR 0009 file exists"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  ADR 0009 file does not exist ($ADR_FILE)"
    FAIL_COUNT=$((FAIL_COUNT + 1))
    return
  fi
  assert_file_contains "ADR has Status" "$ADR_FILE" "Status"
  assert_file_contains "ADR has Context" "$ADR_FILE" "Context"
  assert_file_contains "ADR has Decision" "$ADR_FILE" "Decision"
  assert_file_contains "ADR has Consequences" "$ADR_FILE" "Consequences"
}

test_changelog_bl048() {
  echo "--- test_changelog_bl048 ---"
  if grep -qi "BL-048\|pipeline.*summar" "$CHANGELOG_FILE"; then
    echo "PASS  CHANGELOG has BL-048 or pipeline summary entry"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  CHANGELOG missing BL-048 or pipeline summary entry"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
}

test_deferred_items() {
  echo "--- test_deferred_items ---"
  if find "$BACKLOG_DIR" -type f -name "*.md" -exec grep -li "coverage delta\|bounded context\|per-test-name" {} + 2>/dev/null | head -1 | grep -q .; then
    echo "PASS  backlog has deferred items"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  backlog missing deferred items (coverage delta / bounded context / per-test-name)"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
}

test_line_counts() {
  echo "--- test_line_counts ---"
  local files=("$SPEC_DEV_FILE" "$SPEC_FIX_FILE" "$SPEC_REFACTOR_FILE" "$DESIGN_FILE" "$IMPLEMENT_FILE")
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

test_idempotent_overwrite() {
  echo "--- test_idempotent_overwrite ---"
  local files=("$SPEC_DEV_FILE" "$SPEC_FIX_FILE" "$SPEC_REFACTOR_FILE" "$DESIGN_FILE" "$IMPLEMENT_FILE")
  for f in "${files[@]}"; do
    local basename
    basename="$(basename "$f")"
    if grep -q "Phase Summary" "$f"; then
      if grep -qi "overwrite\|idempotent" "$f"; then
        echo "PASS  $basename has overwrite/idempotent with Phase Summary"
        PASS_COUNT=$((PASS_COUNT + 1))
      else
        echo "FAIL  $basename has Phase Summary but missing overwrite/idempotent"
        FAIL_COUNT=$((FAIL_COUNT + 1))
      fi
    fi
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
  test_skill_project_detection
  test_skill_grillme_rules
  test_skill_adversarial_schema
  test_specdev_dry_ref
  test_specfix_dry_ref
  test_specrefactor_dry_ref
  test_specdev_grillme_accumulator
  test_specdev_grillme_table
  test_specdev_us_ac_tables
  test_specdev_adversary_table
  test_specdev_artifacts_table
  test_specdev_phase_summary
  test_specfix_variant_tables
  test_specrefactor_variant_tables
  test_design_reviews_table
  test_design_adversary_table
  test_design_filechanges_table
  test_design_artifacts_table
  test_design_phase_summary
  test_implement_tasks_table
  test_implement_commits_table
  test_implement_docs_table
  test_implement_artifacts_table
  test_implement_commit_accumulator
  test_implement_phase_summary
  test_adr_0009
  test_changelog_bl048
  test_deferred_items
  test_line_counts
  test_idempotent_overwrite
fi

echo ""
echo "================================"
echo "RESULTS: $PASS_COUNT passed, $FAIL_COUNT failed"
echo "================================"

[ "$FAIL_COUNT" -eq 0 ] || exit 1
