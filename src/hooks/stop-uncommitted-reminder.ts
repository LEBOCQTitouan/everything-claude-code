#!/usr/bin/env node
/**
 * Stop Hook: Remind about uncommitted changes
 *
 * Cross-platform (Windows, macOS, Linux)
 *
 * Runs at session end. If there are uncommitted changes in a git repo,
 * emits a reminder to commit them for version history purposes.
 */

import { isGitRepo, runCommand, log } from '../lib/utils';

function checkUncommitted(stdinData: string): void {
  try {
    if (!isGitRepo()) {
      process.stdout.write(stdinData);
      process.exit(0);
    }

    const status = runCommand('git status --porcelain');
    if (!status.success) {
      process.stdout.write(stdinData);
      process.exit(0);
    }

    const lines = status.output.trim().split('\n').filter(Boolean);
    if (lines.length === 0) {
      process.stdout.write(stdinData);
      process.exit(0);
    }

    const staged = lines.filter(l => /^[MADRC]/.test(l)).length;
    const unstaged = lines.filter(l => /^.[MDRC?]/.test(l) || /^\?\?/.test(l)).length;

    log('[Hook] REMINDER: You have uncommitted changes.');
    if (staged > 0) {
      log(`[Hook]   Staged: ${staged} file(s)`);
    }
    if (unstaged > 0) {
      log(`[Hook]   Unstaged/untracked: ${unstaged} file(s)`);
    }
    log('[Hook]   Commit each logical change separately for version history.');
    log('[Hook]   See: skill atomic-commits, rule git-workflow.md');
  } catch (err) {
    log(`[Hook] stop-uncommitted-reminder error: ${(err as Error).message}`);
  }

  process.stdout.write(stdinData);
  process.exit(0);
}

const MAX_STDIN = 1024 * 1024;
let data = '';
process.stdin.setEncoding('utf8');

process.stdin.on('data', (chunk: string) => {
  if (data.length < MAX_STDIN) {
    const remaining = MAX_STDIN - data.length;
    data += chunk.substring(0, remaining);
  }
});

process.stdin.on('end', () => {
  checkUncommitted(data);
});
