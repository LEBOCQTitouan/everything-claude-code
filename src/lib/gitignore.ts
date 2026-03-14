/**
 * Gitignore management: adds/updates ECC entries in .gitignore.
 * Append-only — never removes existing entries.
 */

import fs from 'fs';
import path from 'path';
import { spawnSync } from 'child_process';

const ECC_SECTION_HEADER = '# ECC (Everything Claude Code) generated files';
const ECC_SECTION_FOOTER = '# End ECC generated files';

/**
 * Default entries ECC should add to .gitignore in target projects.
 */
export const ECC_GITIGNORE_ENTRIES: ReadonlyArray<{ pattern: string; comment: string }> = [
  { pattern: '.claude/settings.local.json', comment: 'Claude Code local settings (machine-specific)' },
  { pattern: '.claude/.ecc-manifest.json', comment: 'ECC installation manifest' },
  { pattern: 'docs/CODEMAPS/', comment: 'Generated architecture docs (regeneratable via /update-codemaps)' },
  { pattern: '.claude/plans/', comment: 'Autonomous loop plans (ephemeral)' },
  { pattern: '.mcp.json', comment: 'MCP server config (may contain API keys)' },
  { pattern: 'CLAUDE.local.md', comment: 'Personal Claude Code instructions (never commit)' }
];

/** Result of ensuring gitignore entries — patterns added, already present, and skip flag. */
export interface GitignoreResult {
  added: string[];
  alreadyPresent: string[];
  skipped: boolean; // true if not a git repo
}

/**
 * Check if a directory is inside a git repository.
 */
export function isGitRepo(dir: string): boolean {
  const result = spawnSync('git', ['rev-parse', '--git-dir'], {
    cwd: dir,
    stdio: 'pipe'
  });
  return result.status === 0;
}

/**
 * Check if a file is currently tracked by git.
 */
export function isGitTracked(dir: string, filePath: string): boolean {
  const result = spawnSync('git', ['ls-files', '--error-unmatch', filePath], {
    cwd: dir,
    stdio: 'pipe'
  });
  return result.status === 0;
}

/**
 * Untrack a file from git (git rm --cached) without deleting it.
 */
export function gitUntrack(dir: string, filePath: string): boolean {
  const result = spawnSync('git', ['rm', '--cached', '-r', filePath], {
    cwd: dir,
    stdio: 'pipe'
  });
  return result.status === 0;
}

/**
 * Parse existing .gitignore content and extract all non-comment, non-empty patterns.
 */
function parseGitignorePatterns(content: string): Set<string> {
  return new Set(
    content
      .split('\n')
      .map(line => line.trim())
      .filter(line => line.length > 0 && !line.startsWith('#'))
  );
}

/**
 * Ensure ECC entries are present in .gitignore.
 * Creates .gitignore if it doesn't exist (only in git repos).
 * Returns a report of what was added.
 */
export function ensureGitignoreEntries(projectDir: string, entries: ReadonlyArray<{ pattern: string; comment: string }> = ECC_GITIGNORE_ENTRIES): GitignoreResult {
  if (!isGitRepo(projectDir)) {
    return { added: [], alreadyPresent: [], skipped: true };
  }

  const gitignorePath = path.join(projectDir, '.gitignore');
  const existingContent = fs.existsSync(gitignorePath) ? fs.readFileSync(gitignorePath, 'utf8') : '';

  const existingPatterns = parseGitignorePatterns(existingContent);
  const added: string[] = [];
  const alreadyPresent: string[] = [];

  const toAdd: Array<{ pattern: string; comment: string }> = [];

  for (const entry of entries) {
    if (existingPatterns.has(entry.pattern)) {
      alreadyPresent.push(entry.pattern);
    } else {
      toAdd.push(entry);
      added.push(entry.pattern);
    }
  }

  if (toAdd.length === 0) {
    return { added, alreadyPresent, skipped: false };
  }

  // Build the section to append
  const sectionLines: string[] = ['', ECC_SECTION_HEADER];
  for (const entry of toAdd) {
    sectionLines.push(`# ${entry.comment}`);
    sectionLines.push(entry.pattern);
  }
  sectionLines.push(ECC_SECTION_FOOTER);
  sectionLines.push('');

  const newContent = existingContent.trimEnd() + '\n' + sectionLines.join('\n');
  fs.writeFileSync(gitignorePath, newContent, 'utf8');

  return { added, alreadyPresent, skipped: false };
}

/**
 * Find ECC-generated files that are currently tracked by git.
 * Returns paths relative to projectDir.
 */
export function findTrackedEccFiles(projectDir: string): string[] {
  if (!isGitRepo(projectDir)) return [];

  const tracked: string[] = [];
  for (const entry of ECC_GITIGNORE_ENTRIES) {
    const fullPath = path.join(projectDir, entry.pattern);
    // For directories (ending with /), check if any files inside are tracked
    if (entry.pattern.endsWith('/')) {
      if (fs.existsSync(fullPath)) {
        const result = spawnSync('git', ['ls-files', entry.pattern], {
          cwd: projectDir,
          stdio: 'pipe',
          encoding: 'utf8'
        });
        if (result.status === 0 && result.stdout.trim().length > 0) {
          tracked.push(entry.pattern);
        }
      }
    } else if (isGitTracked(projectDir, entry.pattern)) {
      tracked.push(entry.pattern);
    }
  }
  return tracked;
}
