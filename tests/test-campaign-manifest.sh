#!/usr/bin/env bash
# Test suite for BL-035: Campaign Manifest for Amnesiac Agents
set -uo pipefail

PASS_COUNT=0
FAIL_COUNT=0
CURRENT_TEST=""

pass() { PASS_COUNT=$((PASS_COUNT + 1)); echo "  PASS: $CURRENT_TEST"; }
fail() { FAIL_COUNT=$((FAIL_COUNT + 1)); echo "  FAIL: $CURRENT_TEST — $1"; }

assert_file_contains() {
  local file="$1" pattern="$2"
  if grep -qi "$pattern" "$file" 2>/dev/null; then return 0; else return 1; fi
}

assert_file_not_contains() {
  local file="$1" pattern="$2"
  if grep -qi "$pattern" "$file" 2>/dev/null; then return 1; else return 0; fi
}

# === Group 1: Foundation ===

test_artifact_schemas_skill() {
  CURRENT_TEST="artifact-schemas skill frontmatter"
  grep -q "^name: artifact-schemas" skills/artifact-schemas/SKILL.md && \
  grep -q "^origin: ECC" skills/artifact-schemas/SKILL.md && \
  grep -q "spec.md" skills/artifact-schemas/SKILL.md && \
  grep -q "design.md" skills/artifact-schemas/SKILL.md && \
  grep -q "tasks.md" skills/artifact-schemas/SKILL.md && \
  grep -q "state.json" skills/artifact-schemas/SKILL.md && \
  grep -q "campaign.md" skills/artifact-schemas/SKILL.md && pass || fail "missing frontmatter or schemas"
}

test_campaign_manifest_skill() {
  CURRENT_TEST="campaign-manifest skill schema sections"
  grep -q "^name: campaign-manifest" skills/campaign-manifest/SKILL.md && \
  grep -q "^origin: ECC" skills/campaign-manifest/SKILL.md && \
  grep -q "Status:" skills/campaign-manifest/SKILL.md && \
  grep -q "Artifacts" skills/campaign-manifest/SKILL.md && \
  grep -q "Grill-Me Decisions" skills/campaign-manifest/SKILL.md && \
  grep -q "Adversary History" skills/campaign-manifest/SKILL.md && \
  grep -q "Agent Outputs" skills/campaign-manifest/SKILL.md && \
  grep -q "Commit Trail" skills/campaign-manifest/SKILL.md && pass || fail "missing sections"
}

test_campaign_manifest_lifecycle() {
  CURRENT_TEST="campaign-manifest lifecycle rules"
  assert_file_contains skills/campaign-manifest/SKILL.md "Phase 0" && \
  assert_file_contains skills/campaign-manifest/SKILL.md "Migration" && \
  assert_file_contains skills/campaign-manifest/SKILL.md "Malformed" && pass || fail "missing lifecycle rules"
}

test_workflow_init_toolchain() {
  CURRENT_TEST="workflow-init.sh has toolchain field"
  assert_file_contains .claude/hooks/workflow-init.sh "toolchain" && pass || fail "no toolchain"
}

test_workflow_init_campaign_path() {
  CURRENT_TEST="workflow-init.sh has campaign_path"
  assert_file_contains .claude/hooks/workflow-init.sh "campaign_path" && pass || fail "no campaign_path"
}

test_toolchain_persist_exists() {
  CURRENT_TEST="toolchain-persist.sh exists with jq --arg"
  test -f .claude/hooks/toolchain-persist.sh && \
  assert_file_contains .claude/hooks/toolchain-persist.sh "jq --arg" && pass || fail "missing or no jq --arg"
}

test_toolchain_persist_jq_fallback() {
  CURRENT_TEST="toolchain-persist.sh has jq unavailable warning"
  assert_file_contains .claude/hooks/toolchain-persist.sh "jq unavailable" && pass || fail "no jq fallback"
}

test_phase_transition_campaign() {
  CURRENT_TEST="phase-transition.sh supports campaign artifact"
  assert_file_contains .claude/hooks/phase-transition.sh "campaign" && pass || fail "no campaign support"
}

test_phase_transition_jq_arg() {
  CURRENT_TEST="phase-transition.sh uses jq --arg"
  assert_file_contains .claude/hooks/phase-transition.sh "jq.*--arg\|--arg.*target" && pass || fail "no jq --arg"
}

test_scope_check_design_path() {
  CURRENT_TEST="scope-check.sh reads design_path"
  assert_file_contains .claude/hooks/scope-check.sh "design_path" && pass || fail "no design_path"
}

test_scope_check_legacy_fallback() {
  CURRENT_TEST="scope-check.sh has legacy solution.md fallback"
  assert_file_contains .claude/hooks/scope-check.sh "solution.md" && pass || fail "no fallback"
}

test_scope_check_path_guard() {
  CURRENT_TEST="scope-check.sh has path traversal guard"
  assert_file_contains .claude/hooks/scope-check.sh "realpath\|PROJECT_DIR" && pass || fail "no path guard"
}

# === Group 2: Spec Fixes ===

test_shared_skill_campaign_init() {
  CURRENT_TEST="spec-pipeline-shared has Campaign Init"
  assert_file_contains skills/spec-pipeline-shared/SKILL.md "Campaign Init" && pass || fail "missing"
}

test_shared_skill_grillme_disk() {
  CURRENT_TEST="spec-pipeline-shared has Grill-Me Disk Persistence"
  assert_file_contains skills/spec-pipeline-shared/SKILL.md "Grill-Me Disk Persistence" && pass || fail "missing"
}

test_shared_skill_draft_spec() {
  CURRENT_TEST="spec-pipeline-shared has Draft Spec Persistence"
  assert_file_contains skills/spec-pipeline-shared/SKILL.md "Draft Spec Persistence" && pass || fail "missing"
}

test_shared_skill_adversary_history() {
  CURRENT_TEST="spec-pipeline-shared has Adversary History Tracking"
  assert_file_contains skills/spec-pipeline-shared/SKILL.md "Adversary History Tracking" && pass || fail "missing"
}

test_shared_skill_agent_output() {
  CURRENT_TEST="spec-pipeline-shared has Agent Output Tracking"
  assert_file_contains skills/spec-pipeline-shared/SKILL.md "Agent Output Tracking" && pass || fail "missing"
}

test_shared_skill_toolchain_persist() {
  CURRENT_TEST="spec-pipeline-shared references toolchain-persist.sh"
  assert_file_contains skills/spec-pipeline-shared/SKILL.md "toolchain-persist.sh" && pass || fail "missing"
}

test_no_store_mentally() {
  CURRENT_TEST="no 'Store these commands mentally' in spec commands"
  assert_file_not_contains commands/spec-dev.md "Store these commands mentally" && \
  assert_file_not_contains commands/spec-fix.md "Store these commands mentally" && \
  assert_file_not_contains commands/spec-refactor.md "Store these commands mentally" && pass || fail "still present"
}

test_spec_commands_campaign_refs() {
  CURRENT_TEST="spec commands reference campaign via shared skill"
  assert_file_contains commands/spec-dev.md "Grill-Me Disk Persistence" && \
  assert_file_contains commands/spec-fix.md "Grill-Me Disk Persistence" && \
  assert_file_contains commands/spec-refactor.md "Grill-Me Disk Persistence" && pass || fail "missing refs"
}

test_spec_commands_toolchain_refs() {
  CURRENT_TEST="spec commands reference toolchain-persist"
  assert_file_contains commands/spec-dev.md "toolchain-persist" && \
  assert_file_contains commands/spec-fix.md "toolchain-persist" && \
  assert_file_contains commands/spec-refactor.md "toolchain-persist" && pass || fail "missing refs"
}

test_spec_commands_draft_refs() {
  CURRENT_TEST="spec commands reference draft spec persistence"
  assert_file_contains commands/spec-dev.md "Draft Spec Persistence" && \
  assert_file_contains commands/spec-fix.md "Draft Spec Persistence" && \
  assert_file_contains commands/spec-refactor.md "Draft Spec Persistence" && pass || fail "missing refs"
}

# === Group 3: Design Fixes ===

test_design_disk_fallbacks() {
  CURRENT_TEST="design.md has disk fallbacks (>=3 refs)"
  local count
  count=$(grep -c "artifacts.spec_path\|from disk\|disk fallback\|file on disk" commands/design.md)
  [ "$count" -ge 3 ] && pass || fail "only $count refs"
}

test_design_campaign_ref() {
  CURRENT_TEST="design.md references campaign"
  assert_file_contains commands/design.md "campaign" && pass || fail "no campaign ref"
}

# === Group 4: Implement Fixes ===

test_wave_analysis_skill() {
  CURRENT_TEST="wave-analysis skill extracted"
  grep -q "^name: wave-analysis" skills/wave-analysis/SKILL.md && \
  grep -q "^origin: ECC" skills/wave-analysis/SKILL.md && \
  grep -q "left-to-right" skills/wave-analysis/SKILL.md && pass || fail "missing"
}

test_wave_dispatch_skill() {
  CURRENT_TEST="wave-dispatch skill extracted"
  grep -q "^name: wave-dispatch" skills/wave-dispatch/SKILL.md && \
  grep -q "^origin: ECC" skills/wave-dispatch/SKILL.md && \
  grep -q "worktree" skills/wave-dispatch/SKILL.md && pass || fail "missing"
}

test_progress_tracking_skill() {
  CURRENT_TEST="progress-tracking skill extracted"
  grep -q "^name: progress-tracking" skills/progress-tracking/SKILL.md && \
  grep -q "^origin: ECC" skills/progress-tracking/SKILL.md && \
  grep -q "TodoWrite" skills/progress-tracking/SKILL.md && pass || fail "missing"
}

test_tasks_generation_skill() {
  CURRENT_TEST="tasks-generation skill extracted"
  grep -q "^name: tasks-generation" skills/tasks-generation/SKILL.md && \
  grep -q "^origin: ECC" skills/tasks-generation/SKILL.md && \
  grep -q "tasks.md" skills/tasks-generation/SKILL.md && pass || fail "missing"
}

test_implement_skill_refs() {
  CURRENT_TEST="implement.md references all 4 extracted skills"
  assert_file_contains commands/implement.md "wave-analysis" && \
  assert_file_contains commands/implement.md "wave-dispatch" && \
  assert_file_contains commands/implement.md "progress-tracking" && \
  assert_file_contains commands/implement.md "tasks-generation" && pass || fail "missing refs"
}

test_implement_line_count() {
  CURRENT_TEST="implement.md under 350 lines"
  local lines
  lines=$(wc -l < commands/implement.md)
  [ "$lines" -lt 350 ] && pass || fail "$lines lines"
}

test_implement_commit_trail() {
  CURRENT_TEST="implement.md campaign Commit Trail writes"
  assert_file_contains commands/implement.md "Commit Trail" && pass || fail "missing"
}

test_implement_agent_outputs() {
  CURRENT_TEST="implement.md campaign Agent Outputs writes"
  assert_file_contains commands/implement.md "Agent Outputs" && pass || fail "missing"
}

test_implement_campaign_reentry() {
  CURRENT_TEST="implement.md campaign re-entry orientation"
  assert_file_contains commands/implement.md "campaign.*re-entry\|campaign.*orientation\|campaign_path" && pass || fail "missing"
}

# === Final: Docs ===

test_adr_0013() {
  CURRENT_TEST="ADR 0013 exists with required sections"
  test -f docs/adr/0013-campaign-manifest-convention.md && \
  assert_file_contains docs/adr/0013-campaign-manifest-convention.md "Status" && \
  assert_file_contains docs/adr/0013-campaign-manifest-convention.md "Context" && \
  assert_file_contains docs/adr/0013-campaign-manifest-convention.md "Decision" && \
  assert_file_contains docs/adr/0013-campaign-manifest-convention.md "Consequences" && pass || fail "missing"
}

test_glossary_entries() {
  CURRENT_TEST="glossary has Campaign Manifest"
  assert_file_contains docs/domain/glossary.md "Campaign Manifest" && pass || fail "missing"
}

test_changelog_entry() {
  CURRENT_TEST="CHANGELOG has BL-035 entry"
  assert_file_contains CHANGELOG.md "BL-035" && pass || fail "missing"
}

# === Run ===

run_test() { "$1"; }

if [ $# -gt 0 ]; then
  run_test "$1"
else
  echo "=== Campaign Manifest Test Suite (BL-035) ==="
  echo ""
  echo "--- Group 1: Foundation ---"
  test_artifact_schemas_skill
  test_campaign_manifest_skill
  test_campaign_manifest_lifecycle
  test_workflow_init_toolchain
  test_workflow_init_campaign_path
  test_toolchain_persist_exists
  test_toolchain_persist_jq_fallback
  test_phase_transition_campaign
  test_phase_transition_jq_arg
  test_scope_check_design_path
  test_scope_check_legacy_fallback
  test_scope_check_path_guard
  echo ""
  echo "--- Group 2: Spec Fixes ---"
  test_shared_skill_campaign_init
  test_shared_skill_grillme_disk
  test_shared_skill_draft_spec
  test_shared_skill_adversary_history
  test_shared_skill_agent_output
  test_shared_skill_toolchain_persist
  test_no_store_mentally
  test_spec_commands_campaign_refs
  test_spec_commands_toolchain_refs
  test_spec_commands_draft_refs
  echo ""
  echo "--- Group 3: Design Fixes ---"
  test_design_disk_fallbacks
  test_design_campaign_ref
  echo ""
  echo "--- Group 4: Implement Fixes ---"
  test_wave_analysis_skill
  test_wave_dispatch_skill
  test_progress_tracking_skill
  test_tasks_generation_skill
  test_implement_skill_refs
  test_implement_line_count
  test_implement_commit_trail
  test_implement_agent_outputs
  test_implement_campaign_reentry
  echo ""
  echo "--- Final: Docs ---"
  test_adr_0013
  test_glossary_entries
  test_changelog_entry
  echo ""
  echo "=== Results: $PASS_COUNT passed, $FAIL_COUNT failed ==="
  [ "$FAIL_COUNT" -eq 0 ] && exit 0 || exit 1
fi
