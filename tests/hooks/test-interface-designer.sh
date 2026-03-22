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

# --- Agent Tests ---

test_agent_exists() {
  echo "--- test_agent_exists ---"
  if test -f "$AGENT_FILE"; then
    echo "PASS  agent file exists"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  agent file exists ($AGENT_FILE not found)"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
}

test_agent_frontmatter() {
  echo "--- test_agent_frontmatter ---"
  assert_file_contains "frontmatter has name field" "$AGENT_FILE" "^name: interface-designer"
  assert_file_contains "frontmatter has model field" "$AGENT_FILE" "^model: opus"
  assert_file_contains "frontmatter has description field" "$AGENT_FILE" "^description:"
  assert_file_contains "frontmatter tools contains Agent" "$AGENT_FILE" "Agent"
  assert_file_contains "frontmatter skills contains design-an-interface" "$AGENT_FILE" "design-an-interface"
}

test_agent_parallel() {
  echo "--- test_agent_parallel ---"
  if grep -qi "parallel" "$AGENT_FILE"; then
    echo "PASS  agent mentions parallel"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  agent does not mention parallel"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
}

test_agent_allowed_tools() {
  echo "--- test_agent_allowed_tools ---"
  assert_file_contains "agent specifies allowedTools" "$AGENT_FILE" "allowedTools"
}

test_agent_constraints() {
  echo "--- test_agent_constraints ---"
  local count=0
  grep -q "minimize method count" "$AGENT_FILE" && count=$((count + 1))
  grep -q "maximize flexibility" "$AGENT_FILE" && count=$((count + 1))
  grep -qi "optimize.*common case" "$AGENT_FILE" && count=$((count + 1))
  grep -qi "named paradigm" "$AGENT_FILE" && count=$((count + 1))
  if [ "$count" -ge 4 ]; then
    echo "PASS  agent has >= 4 constraints (found $count)"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  agent has >= 4 constraints (found $count, expected >= 4)"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
}

test_agent_optional_constraint() {
  echo "--- test_agent_optional_constraint ---"
  if grep -qi "optional.*constraint\|5th constraint\|additional constraint" "$AGENT_FILE"; then
    echo "PASS  agent mentions optional constraint"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  agent does not mention optional constraint"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
}

test_agent_output_format() {
  echo "--- test_agent_output_format ---"
  local count=0
  grep -qi "signature" "$AGENT_FILE" && count=$((count + 1))
  grep -qi "usage example" "$AGENT_FILE" && count=$((count + 1))
  grep -qi "hides internally" "$AGENT_FILE" && count=$((count + 1))
  grep -qi "tradeoffs" "$AGENT_FILE" && count=$((count + 1))
  if [ "$count" -ge 4 ]; then
    echo "PASS  agent has >= 4 output format items (found $count)"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  agent has >= 4 output format items (found $count, expected >= 4)"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
}

test_agent_language_detection() {
  echo "--- test_agent_language_detection ---"
  if grep -qi "Cargo.toml\|package.json\|go.mod" "$AGENT_FILE"; then
    echo "PASS  agent mentions language detection markers"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  agent does not mention language detection markers"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
}

test_agent_prompt_module() {
  echo "--- test_agent_prompt_module ---"
  if grep -qi "prompt.*module\|ask.*module\|specify.*module\|target.*module" "$AGENT_FILE"; then
    echo "PASS  agent prompts for target module"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  agent does not prompt for target module"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
}

test_agent_todowrite() {
  echo "--- test_agent_todowrite ---"
  assert_file_contains "agent uses TodoWrite" "$AGENT_FILE" "TodoWrite"
  if grep -qi "unavailable\|proceed without" "$AGENT_FILE"; then
    echo "PASS  agent has TodoWrite graceful degradation"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  agent lacks TodoWrite graceful degradation"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
}

test_agent_multi_language() {
  echo "--- test_agent_multi_language ---"
  if grep -qi "multiple.*language\|ask.*language\|which language" "$AGENT_FILE"; then
    echo "PASS  agent handles multiple languages"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  agent does not handle multiple languages"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
}

test_agent_convergence() {
  echo "--- test_agent_convergence ---"
  if grep -qi "structural pattern.*method\|method.*overlap\|convergence" "$AGENT_FILE"; then
    echo "PASS  agent checks for convergence"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  agent does not check for convergence"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
}

test_agent_respawn() {
  echo "--- test_agent_respawn ---"
  if grep -qi "re-spawn\|retry.*converg\|re.spawn" "$AGENT_FILE"; then
    echo "PASS  agent supports respawn on convergence"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  agent does not support respawn on convergence"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
}

test_agent_failure_handling() {
  echo "--- test_agent_failure_handling ---"
  if grep -qi "fail\|timeout\|proceed.*available" "$AGENT_FILE"; then
    echo "PASS  agent handles failures"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  agent does not handle failures"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
}

test_agent_minimum_designs() {
  echo "--- test_agent_minimum_designs ---"
  if grep -qE "minimum.*(2|two)" "$AGENT_FILE"; then
    echo "PASS  agent requires minimum 2 designs"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  agent does not require minimum 2 designs"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
}

test_agent_comparison_dimensions() {
  echo "--- test_agent_comparison_dimensions ---"
  local count=0
  grep -qi "simplicity" "$AGENT_FILE" && count=$((count + 1))
  grep -qiE "general.purpose.*specialized" "$AGENT_FILE" && count=$((count + 1))
  grep -qi "implementation efficiency" "$AGENT_FILE" && count=$((count + 1))
  grep -qi "depth" "$AGENT_FILE" && count=$((count + 1))
  grep -qi "ease of correct use" "$AGENT_FILE" && count=$((count + 1))
  if [ "$count" -ge 5 ]; then
    echo "PASS  agent has >= 5 comparison dimensions (found $count)"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  agent has >= 5 comparison dimensions (found $count, expected >= 5)"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
}

test_agent_never_skip() {
  echo "--- test_agent_never_skip ---"
  if grep -qi "never skip\|DO NOT skip\|must.*compar" "$AGENT_FILE"; then
    echo "PASS  agent enforces no-skip comparison"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  agent does not enforce no-skip comparison"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
}

test_agent_ask_user() {
  echo "--- test_agent_ask_user ---"
  assert_file_contains "agent uses AskUserQuestion" "$AGENT_FILE" "AskUserQuestion"
}

test_agent_graceful_degradation() {
  echo "--- test_agent_graceful_degradation ---"
  if grep -qi "graceful\|fallback\|unavailable" "$AGENT_FILE"; then
    echo "PASS  agent has graceful degradation"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  agent lacks graceful degradation"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
}

test_agent_output_path() {
  echo "--- test_agent_output_path ---"
  assert_file_contains "agent outputs to docs/designs/" "$AGENT_FILE" "docs/designs/"
}

test_agent_create_dir() {
  echo "--- test_agent_create_dir ---"
  if grep -qi "create.*dir\|directory.*not exist\|mkdir" "$AGENT_FILE"; then
    echo "PASS  agent creates directory if needed"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  agent does not create directory if needed"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
}

test_agent_numeric_suffix() {
  echo "--- test_agent_numeric_suffix ---"
  if grep -qi "numeric.*suffix\|already exists\|-1\|-2" "$AGENT_FILE"; then
    echo "PASS  agent handles numeric suffix for existing files"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  agent does not handle numeric suffix for existing files"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
}

# --- Command & Doc Tests ---

test_design_command_reference() {
  echo "--- test_design_command_reference ---"
  assert_file_contains "design command references interface-designer" "$COMMAND_FILE" "interface-designer"
}

test_design_command_optional() {
  echo "--- test_design_command_optional ---"
  if grep -qi "optional.*interface-designer\|interface-designer.*optional" "$COMMAND_FILE"; then
    echo "PASS  design command marks interface-designer as optional"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  design command does not mark interface-designer as optional"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
}

test_agent_spec_directory_output() {
  echo "--- test_agent_spec_directory_output ---"
  assert_file_contains "agent supports spec directory output" "$AGENT_FILE" "docs/specs/"
}

test_glossary_entry() {
  echo "--- test_glossary_entry ---"
  assert_file_contains "glossary has Interface Designer entry" "$PROJECT_ROOT/docs/domain/glossary.md" "Interface Designer"
}

test_changelog_entry() {
  echo "--- test_changelog_entry ---"
  assert_file_contains "changelog has BL-014 entry" "$PROJECT_ROOT/CHANGELOG.md" "BL-014"
}

test_adr_exists() {
  echo "--- test_adr_exists ---"
  local adr_file="$PROJECT_ROOT/docs/adr/0008-designs-directory-convention.md"
  if test -f "$adr_file"; then
    echo "PASS  ADR 0008 file exists"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  ADR 0008 file does not exist ($adr_file)"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
}

test_adr_structure() {
  echo "--- test_adr_structure ---"
  local adr_file="$PROJECT_ROOT/docs/adr/0008-designs-directory-convention.md"
  local count=0
  grep -q "## Status" "$adr_file" && count=$((count + 1))
  grep -q "## Decision" "$adr_file" && count=$((count + 1))
  if [ "$count" -ge 2 ]; then
    echo "PASS  ADR 0008 has Status and Decision sections"
    PASS_COUNT=$((PASS_COUNT + 1))
  else
    echo "FAIL  ADR 0008 missing Status or Decision sections (found $count/2)"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
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
    test_agent_exists
    test_agent_frontmatter
    test_agent_parallel
    test_agent_allowed_tools
    test_agent_constraints
    test_agent_optional_constraint
    test_agent_output_format
    test_agent_language_detection
    test_agent_prompt_module
    test_agent_todowrite
    test_agent_multi_language
    test_agent_convergence
    test_agent_respawn
    test_agent_failure_handling
    test_agent_minimum_designs
    test_agent_comparison_dimensions
    test_agent_never_skip
    test_agent_ask_user
    test_agent_graceful_degradation
    test_agent_output_path
    test_agent_create_dir
    test_agent_numeric_suffix
    test_design_command_reference
    test_design_command_optional
    test_agent_spec_directory_output
    test_glossary_entry
    test_changelog_entry
    test_adr_exists
    test_adr_structure
  fi

  echo ""
  echo "=== Summary: $PASS_COUNT passed, $FAIL_COUNT failed ==="

  if [ "$FAIL_COUNT" -gt 0 ]; then
    exit 1
  fi
  exit 0
}

run_tests "${1:-}"
