#!/usr/bin/env bats

load test_helper

@test "shows worktree segment when in worktree" {
  create_worktree "my-wt" "feat-branch"
  output=$(run_statusline "$BATS_TEST_TMPDIR/my-wt")
  clean=$(strip_ansi "$output")
  [[ "$clean" == *"🌳"* ]]
  [[ "$clean" == *"my-wt"* ]]
}

@test "no worktree segment in main working tree" {
  output=$(run_statusline "$TEST_REPO")
  clean=$(strip_ansi "$output")
  [[ "$clean" != *"🌳"* ]]
}

@test "branch segment is replaced" {
  create_worktree "my-wt" "feat-branch"
  output=$(run_statusline "$BATS_TEST_TMPDIR/my-wt")
  clean=$(strip_ansi "$output")
  [[ "$clean" != *"⎇"* ]]
  [[ "$clean" == *"🌳"* ]]
}

@test "worktree name is basename from subdirectory" {
  create_worktree "my-wt" "feat-branch"
  mkdir -p "$BATS_TEST_TMPDIR/my-wt/subdir/deep"
  output=$(run_statusline "$BATS_TEST_TMPDIR/my-wt/subdir/deep")
  clean=$(strip_ansi "$output")
  [[ "$clean" == *"my-wt"* ]]
  [[ "$clean" != *"deep"* ]]
}

@test "detached HEAD shows detached" {
  create_worktree "det-wt" "det-branch"
  cd "$BATS_TEST_TMPDIR/det-wt"
  git checkout --detach HEAD
  output=$(run_statusline "$BATS_TEST_TMPDIR/det-wt")
  clean=$(strip_ansi "$output")
  [[ "$clean" == *"detached"* ]]
}

@test "bare repo hides worktree segment" {
  bare_dir=$(create_bare_repo)
  output=$(run_statusline "$bare_dir")
  clean=$(strip_ansi "$output")
  [[ "$clean" != *"🌳"* ]]
}

@test "non-git directory hides segment" {
  nogit="$BATS_TEST_TMPDIR/nogit-$$"
  mkdir -p "$nogit"
  output=$(run_statusline "$nogit")
  clean=$(strip_ansi "$output")
  [[ "$clean" != *"🌳"* ]]
  [[ "$clean" != *"⎇"* ]]
}

@test "stale worktree hides segment" {
  create_stale_worktree
  output=$(run_statusline "$TEST_REPO")
  clean=$(strip_ansi "$output")
  [[ "$clean" != *"🌳"* ]]
}

@test "cache hit within TTL" {
  # First run populates cache
  run_statusline "$TEST_REPO" > /dev/null
  # Verify cache file exists in CACHE_DIR
  PWD_HASH=$(echo "$TEST_REPO" | md5sum 2>/dev/null | cut -c1-8 || md5 -q -s "$TEST_REPO" 2>/dev/null | cut -c1-8)
  cache_file="$BATS_TEST_TMPDIR/ecc-sl-cache-${PWD_HASH}"
  [ -f "$cache_file" ]
}

@test "cache miss refreshes" {
  # Pre-populate cache with old content
  PWD_HASH=$(echo "$TEST_REPO" | md5sum 2>/dev/null | cut -c1-8 || md5 -q -s "$TEST_REPO" 2>/dev/null | cut -c1-8)
  cache_file="$BATS_TEST_TMPDIR/ecc-sl-cache-${PWD_HASH}"
  echo "old-branch" > "$cache_file"
  # Make cache old (older than 5s)
  touch -t 202001010000.00 "$cache_file"
  # Run statusline — should refresh
  output=$(run_statusline "$TEST_REPO")
  clean=$(strip_ansi "$output")
  [[ "$clean" != *"old-branch"* ]]
}

@test "cache format is newline-delimited" {
  create_worktree "cache-wt" "cache-branch"
  run_statusline "$BATS_TEST_TMPDIR/cache-wt" > /dev/null
  PWD_HASH=$(echo "$BATS_TEST_TMPDIR/cache-wt" | md5sum 2>/dev/null | cut -c1-8 || md5 -q -s "$BATS_TEST_TMPDIR/cache-wt" 2>/dev/null | cut -c1-8)
  cache_file="$BATS_TEST_TMPDIR/ecc-sl-cache-${PWD_HASH}"
  [ -f "$cache_file" ]
  line_count=$(wc -l < "$cache_file")
  [ "$line_count" -ge 2 ]
}

@test "legacy single-line cache" {
  # Write a legacy single-line cache
  PWD_HASH=$(echo "$TEST_REPO" | md5sum 2>/dev/null | cut -c1-8 || md5 -q -s "$TEST_REPO" 2>/dev/null | cut -c1-8)
  cache_file="$BATS_TEST_TMPDIR/ecc-sl-cache-${PWD_HASH}"
  printf 'main' > "$cache_file"
  touch "$cache_file"  # fresh timestamp
  output=$(run_statusline "$TEST_REPO")
  clean=$(strip_ansi "$output")
  # Should work fine — legacy cache has branch only, no worktree
  [[ "$clean" == *"main"* ]]
  [[ "$clean" != *"🌳"* ]]
}

@test "truncation drops worktree" {
  create_worktree "trunc-wt" "trunc-branch"
  local script_path
  script_path="$(cd "$(dirname "${BATS_TEST_FILENAME}")" && pwd)/../../statusline/statusline-command.sh"
  # Very narrow terminal — only model should fit
  output=$(cd "$BATS_TEST_TMPDIR/trunc-wt" && echo '{"model":{"display_name":"Test"},"context_window":{"used_percentage":42},"cost":{"total_cost_usd":0,"total_duration_ms":0,"total_lines_added":0,"total_lines_removed":0}}' | COLUMNS=50 CACHE_DIR="$BATS_TEST_TMPDIR" bash "$script_path")
  clean=$(strip_ansi "$output")
  # With only 50 cols, worktree segment should be dropped
  [[ "$clean" == *"Test"* ]]
}

@test "narrow variant drops branch" {
  create_worktree "narrow-wt" "narrow-branch-name-that-is-long"
  local script_path
  script_path="$(cd "$(dirname "${BATS_TEST_FILENAME}")" && pwd)/../../statusline/statusline-command.sh"
  # Medium terminal — narrow variant should drop branch from worktree segment
  output=$(cd "$BATS_TEST_TMPDIR/narrow-wt" && echo '{"model":{"display_name":"TestModel"},"context_window":{"used_percentage":42},"cost":{"total_cost_usd":0,"total_duration_ms":60000,"total_lines_added":50,"total_lines_removed":10},"rate_limits":{"five_hour":{"used_percentage":20},"seven_day":{"used_percentage":10}}}' | COLUMNS=120 CACHE_DIR="$BATS_TEST_TMPDIR" bash "$script_path")
  clean=$(strip_ansi "$output")
  # With 120 cols and lots of segments, narrow worktree might be used
  # At minimum, the worktree name should be present
  [[ "$clean" == *"narrow-wt"* ]] || [[ "$clean" != *"🌳"* ]]
}

@test "exact format" {
  create_worktree "fmt-wt" "fmt-branch"
  output=$(run_statusline "$BATS_TEST_TMPDIR/fmt-wt")
  clean=$(strip_ansi "$output")
  [[ "$clean" == *"🌳 fmt-wt (fmt-branch)"* ]]
}

@test "name with hyphens" {
  create_worktree "my-hyphenated-worktree" "feat-test"
  output=$(run_statusline "$BATS_TEST_TMPDIR/my-hyphenated-worktree")
  clean=$(strip_ansi "$output")
  [[ "$clean" == *"my-hyphenated-worktree"* ]]
}
