#!/usr/bin/env bash
# Shared Bats helpers for statusline tests

TEST_REPO=""

setup() {
  TEST_REPO="$BATS_TEST_TMPDIR/test-repo-$$"
  mkdir -p "$TEST_REPO"
  cd "$TEST_REPO"
  git init -q
  git config user.email "test@example.com"
  git config user.name "Test User"
  git checkout -q -b main 2>/dev/null || true
  touch README.md
  git add README.md
  git commit -q -m "initial commit"
  export CACHE_DIR="$BATS_TEST_TMPDIR"
}

teardown() {
  rm -rf "$TEST_REPO" 2>/dev/null || true
}

create_worktree() {
  local name="$1"
  local branch="$2"
  local wt_path="$BATS_TEST_TMPDIR/${name}"
  git -C "$TEST_REPO" branch "$branch" 2>/dev/null || true
  git -C "$TEST_REPO" worktree add "$wt_path" "$branch"
}

create_bare_repo() {
  local bare_dir="$BATS_TEST_TMPDIR/bare-repo-$$"
  git init -q --bare "$bare_dir"
  printf '%s' "$bare_dir"
}

create_stale_worktree() {
  local stale_dir="$BATS_TEST_TMPDIR/stale-wt-$$"
  git -C "$TEST_REPO" branch stale-branch 2>/dev/null || true
  git -C "$TEST_REPO" worktree add "$stale_dir" stale-branch
  rm -rf "$stale_dir"
}

run_statusline() {
  local cwd="${1:-$TEST_REPO}"
  local script_dir
  script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
  local script_path="${script_dir}/../../statusline/statusline-command.sh"
  (
    cd "$cwd" && \
    echo '{"model":{"display_name":"Test"},"context_window":{"used_percentage":42},"cost":{"total_cost_usd":0,"total_duration_ms":0,"total_lines_added":0,"total_lines_removed":0}}' \
    | COLUMNS=200 CACHE_DIR="$BATS_TEST_TMPDIR" bash "$script_path"
  )
}

strip_ansi() {
  printf '%s' "$1" | sed 's/\x1b\[[0-9;]*m//g'
}
