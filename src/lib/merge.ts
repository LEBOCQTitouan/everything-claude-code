/**
 * Merge strategies for ECC installation.
 * Handles directory merges, settings merges, and interactive diff review.
 */

import fs from 'fs';
import path from 'path';
import readline from 'readline';
import { generateDiff, smartMerge, isClaudeAvailable, contentsDiffer } from './smart-merge';
import { bold, dim, green, yellow, cyan } from './ansi';
import type { EccManifest } from './manifest';

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/** User action for resolving a file conflict (legacy flow). */
export type ConflictChoice = 'overwrite' | 'keep' | 'diff' | 'smart-merge';
/** Cached conflict choice to apply to all remaining files, or null for per-file prompting. */
export type ConflictApplyAll = ConflictChoice | null;

/** User action for interactive file review. */
export type ReviewChoice = 'accept' | 'keep' | 'smart-merge';
/** Cached review choice to apply to all remaining files, or null for per-file prompting. */
export type ReviewApplyAll = ReviewChoice | null;

/** Report of files added, updated, unchanged, skipped, smart-merged, and errored during a merge. */
export interface MergeReport {
  added: string[];
  updated: string[];
  unchanged: string[];
  skipped: string[];
  smartMerged: string[];
  errors: string[];
}

/** Options controlling merge behavior — dry run, force, interactive mode, and cached choice. */
export interface MergeOptions {
  dryRun: boolean;
  force: boolean;
  interactive: boolean;
  applyAll: ReviewApplyAll;
}

/**
 * Create default merge options.
 */
export function defaultMergeOptions(): MergeOptions {
  return {
    dryRun: false,
    force: false,
    interactive: true,
    applyAll: null
  };
}

// ---------------------------------------------------------------------------
// Empty report helper
// ---------------------------------------------------------------------------

function emptyReport(): MergeReport {
  return { added: [], updated: [], unchanged: [], skipped: [], smartMerged: [], errors: [] };
}

// ---------------------------------------------------------------------------
// Interactive file review prompt
// ---------------------------------------------------------------------------

/**
 * Count lines added/removed between two strings for a compact summary.
 */
function diffStats(existingContent: string, incomingContent: string): { added: number; removed: number } {
  const existingLines = existingContent.split('\n');
  const incomingLines = incomingContent.split('\n');
  const { computeLineDiff } = require('./smart-merge');
  const diffLines = computeLineDiff(existingLines, incomingLines);
  let added = 0;
  let removed = 0;
  for (const line of diffLines) {
    if (line.type === 'added') added++;
    if (line.type === 'removed') removed++;
  }
  return { added, removed };
}

/**
 * Prompt user for interactive file review.
 * Shows diff for existing files or preview for new files, then asks for action.
 */
export async function promptFileReview(
  filename: string,
  srcPath: string,
  destPath: string | null,
  context: {
    isNew: boolean;
    progress: { current: number; total: number };
  }
): Promise<{ choice: ReviewChoice; applyAll: boolean }> {
  const rl = readline.createInterface({
    input: process.stdin,
    output: process.stderr,
    terminal: true
  });

  return new Promise(resolve => {
    const { current, total } = context.progress;
    const prefix = dim(`[${current}/${total}]`);

    if (context.isNew) {
      // New file: show preview
      const preview = fs.readFileSync(srcPath, 'utf8').split('\n').slice(0, 10);
      console.error(`\n${prefix} ${green('NEW')}: ${bold(filename)}`);
      console.error(dim('  Preview:'));
      for (const line of preview) {
        console.error(dim(`    ${line}`));
      }
      if (fs.readFileSync(srcPath, 'utf8').split('\n').length > 10) {
        console.error(dim('    ...'));
      }
      console.error('');
      console.error(`  ${green('[a]')} Accept  ${dim('[s]')} Skip  ${dim('[p]')} Preview full  ${cyan('[A/S]')} All remaining`);
    } else {
      // Changed file: show diff summary + diff
      const existing = fs.readFileSync(destPath!, 'utf8');
      const incoming = fs.readFileSync(srcPath, 'utf8');
      const stats = diffStats(existing, incoming);
      const statsStr = `${green(`+${stats.added}`)} ${dim('/')} ${yellow(`-${stats.removed}`)} lines`;

      console.error(`\n${prefix} ${yellow('CHANGED')}: ${bold(filename)} (${statsStr})`);
      console.error(generateDiff(existing, incoming, filename));
      console.error('');

      const mergeHint = isClaudeAvailable() ? `  ${dim('[m]')} Smart merge` : '';
      const mergeAllHint = isClaudeAvailable() ? '/M' : '';
      console.error(`  ${green('[a]')} Accept  ${dim('[k]')} Keep existing${mergeHint}  ${cyan(`[A/K${mergeAllHint}]`)} All remaining`);
    }

    function ask(): void {
      rl.question('  Choice [a]: ', answer => {
        const trimmed = answer.trim();

        switch (trimmed) {
          case 'a':
          case '':
            rl.close();
            return resolve({ choice: 'accept', applyAll: false });
          case 'A':
            rl.close();
            return resolve({ choice: 'accept', applyAll: true });
          case 's':
          case 'k':
            rl.close();
            return resolve({ choice: 'keep', applyAll: false });
          case 'S':
          case 'K':
            rl.close();
            return resolve({ choice: 'keep', applyAll: true });
          case 'p':
            if (context.isNew) {
              console.error('\n' + fs.readFileSync(srcPath, 'utf8') + '\n');
            } else {
              const existing = fs.readFileSync(destPath!, 'utf8');
              const incoming = fs.readFileSync(srcPath, 'utf8');
              console.error('\n' + generateDiff(existing, incoming, filename) + '\n');
            }
            return ask();
          case 'm':
          case 'M':
            if (!context.isNew && isClaudeAvailable()) {
              rl.close();
              return resolve({ choice: 'smart-merge', applyAll: trimmed === 'M' });
            }
            if (context.isNew) {
              console.error('  Smart merge not available for new files.');
            } else {
              console.error('  Claude CLI not available.');
            }
            return ask();
          default:
            if (context.isNew) {
              console.error('  Invalid choice. Try a/s/p or A/S.');
            } else {
              console.error('  Invalid choice. Try a/k/m/p or A/K/M.');
            }
            return ask();
        }
      });
    }

    ask();
  });
}

/**
 * Apply a review choice to a file.
 * Returns updated options (with applyAll set if user chose "all").
 */
function applyReviewChoice(choice: ReviewChoice, filename: string, srcPath: string, destPath: string, options: MergeOptions, report: MergeReport, isNew: boolean): MergeOptions {
  switch (choice) {
    case 'accept':
      if (!options.dryRun) {
        fs.mkdirSync(path.dirname(destPath), { recursive: true });
        fs.copyFileSync(srcPath, destPath);
      }
      if (isNew) {
        report.added.push(filename);
      } else {
        report.updated.push(filename);
      }
      break;

    case 'keep':
      report.skipped.push(filename);
      break;

    case 'smart-merge': {
      const existing = fs.readFileSync(destPath, 'utf8');
      const incoming = fs.readFileSync(srcPath, 'utf8');
      const result = smartMerge(existing, incoming, filename);

      if (result.success && result.merged) {
        if (!options.dryRun) {
          fs.writeFileSync(destPath, result.merged, 'utf8');
        }
        report.smartMerged.push(filename);
      } else {
        console.error(`  Smart merge failed: ${result.error}. Keeping existing.`);
        report.skipped.push(filename);
      }
      break;
    }
  }
  return options;
}

/**
 * Review a single file interactively (or apply cached/force/non-interactive logic).
 * Handles the decision tree: force → non-interactive → unchanged → applyAll → prompt.
 */
async function reviewFile(
  filename: string,
  srcPath: string,
  destPath: string,
  isNew: boolean,
  options: MergeOptions,
  report: MergeReport,
  progress: { current: number; total: number }
): Promise<MergeOptions> {
  // Force mode: always accept
  if (options.force) {
    return applyReviewChoice('accept', filename, srcPath, destPath, options, report, isNew);
  }

  // Apply-all cached choice (explicit user decision takes priority)
  if (options.applyAll !== null) {
    return applyReviewChoice(options.applyAll, filename, srcPath, destPath, options, report, isNew);
  }

  // Non-interactive or dry-run: accept all (no prompts)
  if (!options.interactive || options.dryRun) {
    return applyReviewChoice('accept', filename, srcPath, destPath, options, report, isNew);
  }

  // Interactive prompt
  const { choice, applyAll } = await promptFileReview(filename, srcPath, isNew ? null : destPath, {
    isNew,
    progress
  });
  const newOptions = applyAll ? { ...options, applyAll: choice } : options;

  return applyReviewChoice(choice, filename, srcPath, destPath, newOptions, report, isNew);
}

// ---------------------------------------------------------------------------
// Category header
// ---------------------------------------------------------------------------

/**
 * Print a category header showing how many files need review.
 */
function printCategoryHeader(label: string, totalFiles: number, changedFiles: number): void {
  if (changedFiles === 0) {
    console.error(`\n${dim(`--- ${label} (${totalFiles} files, all unchanged) ---`)}`);
  } else {
    console.error(`\n${bold(`--- ${label} (${changedFiles} changed out of ${totalFiles}) ---`)}`);
  }
}

// ---------------------------------------------------------------------------
// Merge functions
// ---------------------------------------------------------------------------

/**
 * Merge a directory of files from source to destination.
 * In interactive mode, shows diffs and prompts for each changed file.
 */
export async function mergeDirectory(srcDir: string, destDir: string, _manifest: EccManifest | null, artifactType: 'agents' | 'commands', options: MergeOptions): Promise<MergeReport> {
  const report = emptyReport();

  if (!fs.existsSync(srcDir)) return report;

  const srcFiles = fs.readdirSync(srcDir).filter(f => f.endsWith('.md'));
  if (!options.dryRun) {
    fs.mkdirSync(destDir, { recursive: true });
  }

  // Pre-scan: identify which files need review (new or changed)
  const filesToReview: Array<{ filename: string; srcPath: string; destPath: string; isNew: boolean }> = [];
  for (const filename of srcFiles) {
    const srcPath = path.join(srcDir, filename);
    const destPath = path.join(destDir, filename);

    if (!fs.existsSync(destPath)) {
      filesToReview.push({ filename, srcPath, destPath, isNew: true });
    } else if (contentsDiffer(srcPath, destPath)) {
      filesToReview.push({ filename, srcPath, destPath, isNew: false });
    } else {
      report.unchanged.push(filename);
    }
  }

  // Show category header in interactive mode
  if (options.interactive && !options.dryRun) {
    const label = artifactType.charAt(0).toUpperCase() + artifactType.slice(1);
    printCategoryHeader(label, srcFiles.length, filesToReview.length);
  }

  // Review each changed file
  let currentOptions = { ...options };
  for (let i = 0; i < filesToReview.length; i++) {
    const { filename, srcPath, destPath, isNew } = filesToReview[i];
    currentOptions = await reviewFile(filename, srcPath, destPath, isNew, currentOptions, report, { current: i + 1, total: filesToReview.length });
  }

  return report;
}

/**
 * Merge skills directory (skill-level granularity, not file-level).
 * Each skill is a directory that is merged atomically.
 */
export async function mergeSkills(srcDir: string, destDir: string, _manifest: EccManifest | null, options: MergeOptions): Promise<MergeReport> {
  const report = emptyReport();

  if (!fs.existsSync(srcDir)) return report;

  const srcSkills = fs
    .readdirSync(srcDir, { withFileTypes: true })
    .filter(e => e.isDirectory())
    .map(e => e.name);

  if (!options.dryRun) {
    fs.mkdirSync(destDir, { recursive: true });
  }

  // Pre-scan: identify which skills need review
  const skillsToReview: Array<{ skillName: string; srcSkillDir: string; destSkillDir: string; isNew: boolean; srcSkillMd: string; destSkillMd: string }> = [];
  for (const skillName of srcSkills) {
    const srcSkillDir = path.join(srcDir, skillName);
    const destSkillDir = path.join(destDir, skillName);
    const srcSkillMd = path.join(srcSkillDir, 'SKILL.md');
    const destSkillMd = path.join(destSkillDir, 'SKILL.md');

    if (!fs.existsSync(destSkillDir)) {
      skillsToReview.push({ skillName, srcSkillDir, destSkillDir, isNew: true, srcSkillMd, destSkillMd });
    } else if (fs.existsSync(srcSkillMd) && contentsDiffer(srcSkillMd, destSkillMd)) {
      skillsToReview.push({ skillName, srcSkillDir, destSkillDir, isNew: false, srcSkillMd, destSkillMd });
    } else {
      report.unchanged.push(skillName);
    }
  }

  // Show category header in interactive mode
  if (options.interactive && !options.dryRun) {
    printCategoryHeader('Skills', srcSkills.length, skillsToReview.length);
  }

  let currentOptions = { ...options };
  for (let i = 0; i < skillsToReview.length; i++) {
    const { skillName, srcSkillDir, destSkillDir, isNew, srcSkillMd, destSkillMd } = skillsToReview[i];

    // Use SKILL.md for diff display, but copy entire directory on accept
    const prevUpdated = report.updated.length;
    const prevAdded = report.added.length;
    const prevSmartMerged = report.smartMerged.length;

    currentOptions = await reviewFile(skillName, srcSkillMd, isNew ? path.join(destSkillDir, 'SKILL.md') : destSkillMd, isNew, currentOptions, report, {
      current: i + 1,
      total: skillsToReview.length
    });

    // If the file was accepted (added/updated/smart-merged), copy the whole skill directory
    const wasAccepted = report.updated.length > prevUpdated || report.added.length > prevAdded || report.smartMerged.length > prevSmartMerged;

    if (wasAccepted && !currentOptions.dryRun) {
      copyDirRecursive(srcSkillDir, destSkillDir);
    }
  }

  return report;
}

/**
 * Merge rules directory, grouped by subdirectory.
 */
export async function mergeRules(srcDir: string, destDir: string, _manifest: EccManifest | null, groups: string[], options: MergeOptions): Promise<MergeReport> {
  const report = emptyReport();

  // Pre-scan all groups
  const filesToReview: Array<{ label: string; srcPath: string; destPath: string; isNew: boolean }> = [];
  for (const group of groups) {
    const srcGroupDir = path.join(srcDir, group);
    const destGroupDir = path.join(destDir, group);

    if (!fs.existsSync(srcGroupDir)) continue;

    const srcFiles = fs.readdirSync(srcGroupDir).filter(f => f.endsWith('.md'));
    if (!options.dryRun) {
      fs.mkdirSync(destGroupDir, { recursive: true });
    }

    for (const filename of srcFiles) {
      const srcPath = path.join(srcGroupDir, filename);
      const destPath = path.join(destGroupDir, filename);
      const label = `${group}/${filename}`;

      if (!fs.existsSync(destPath)) {
        filesToReview.push({ label, srcPath, destPath, isNew: true });
      } else if (contentsDiffer(srcPath, destPath)) {
        filesToReview.push({ label, srcPath, destPath, isNew: false });
      } else {
        report.unchanged.push(label);
      }
    }
  }

  // Count total rule files for header
  const totalRuleFiles = filesToReview.length + report.unchanged.length;

  if (options.interactive && !options.dryRun) {
    printCategoryHeader('Rules', totalRuleFiles, filesToReview.length);
  }

  let currentOptions = { ...options };
  for (let i = 0; i < filesToReview.length; i++) {
    const { label, srcPath, destPath, isNew } = filesToReview[i];
    currentOptions = await reviewFile(label, srcPath, destPath, isNew, currentOptions, report, { current: i + 1, total: filesToReview.length });
  }

  return report;
}

// ---------------------------------------------------------------------------
// Legacy conflict resolution (deprecated)
// ---------------------------------------------------------------------------

/**
 * @deprecated Use promptFileReview instead.
 * Prompt user for conflict resolution choice.
 */
export async function promptConflict(filename: string, existingPath: string, incomingPath: string): Promise<{ choice: ConflictChoice; applyAll: boolean }> {
  const rl = readline.createInterface({
    input: process.stdin,
    output: process.stderr,
    terminal: true
  });

  return new Promise(resolve => {
    console.error(`\nConflict: ${filename} already exists and is not ECC-managed.`);
    console.error('  [o] Overwrite with ECC version');
    console.error('  [k] Keep existing (skip)');
    console.error('  [d] Show diff');
    if (isClaudeAvailable()) {
      console.error('  [m] Smart merge with Claude');
    }
    console.error('  [O/K/M] Apply choice to all remaining conflicts');

    function ask(): void {
      rl.question('  Choice: ', answer => {
        const trimmed = answer.trim();

        switch (trimmed) {
          case 'o':
            rl.close();
            return resolve({ choice: 'overwrite', applyAll: false });
          case 'O':
            rl.close();
            return resolve({ choice: 'overwrite', applyAll: true });
          case 'k':
          case '':
            rl.close();
            return resolve({ choice: 'keep', applyAll: false });
          case 'K':
            rl.close();
            return resolve({ choice: 'keep', applyAll: true });
          case 'd': {
            const existing = fs.readFileSync(existingPath, 'utf8');
            const incoming = fs.readFileSync(incomingPath, 'utf8');
            console.error('\n' + generateDiff(existing, incoming, filename) + '\n');
            return ask();
          }
          case 'm':
          case 'M':
            if (isClaudeAvailable()) {
              rl.close();
              return resolve({ choice: 'smart-merge', applyAll: trimmed === 'M' });
            }
            console.error('  Claude CLI not available.');
            return ask();
          default:
            console.error('  Invalid choice. Try o/k/d/m or O/K/M.');
            return ask();
        }
      });
    }

    ask();
  });
}

// ---------------------------------------------------------------------------
// Hooks merge (unchanged — not interactive, deduplication-based)
// ---------------------------------------------------------------------------

/**
 * Check if a hook entry is a legacy ECC hook that should be removed.
 */
export function isLegacyEccHook(entry: Record<string, unknown>): boolean {
  const hooks = entry.hooks;
  if (!Array.isArray(hooks)) return false;

  for (const hook of hooks) {
    const cmd = (hook as Record<string, unknown>).command;
    if (typeof cmd !== 'string') continue;

    if (cmd.includes('scripts/hooks/') && !cmd.includes('run-with-flags-shell.sh')) {
      return true;
    }

    if (
      cmd.includes('node -e') &&
      (cmd.includes('dev-server') ||
        cmd.includes('tmux') ||
        cmd.includes('git push') ||
        cmd.includes('console.log') ||
        cmd.includes('check-console') ||
        cmd.includes('pr-created') ||
        cmd.includes('build-complete'))
    ) {
      return true;
    }
  }

  return false;
}

function removeLegacyHooks(hooks: Record<string, Array<Record<string, unknown>>>): number {
  let removed = 0;
  for (const event of Object.keys(hooks)) {
    if (!Array.isArray(hooks[event])) continue;
    const original = hooks[event].length;
    hooks[event] = hooks[event].filter(entry => !isLegacyEccHook(entry));
    removed += original - hooks[event].length;
  }
  return removed;
}

/**
 * Merge hooks from source hooks.json into destination settings.json.
 */
export function mergeHooks(hooksJsonPath: string, settingsJsonPath: string, pluginRoot: string): { added: number; existing: number; legacyRemoved: number } {
  const existing = fs.existsSync(settingsJsonPath) ? JSON.parse(fs.readFileSync(settingsJsonPath, 'utf8')) : {};

  const raw = fs.readFileSync(hooksJsonPath, 'utf8').replaceAll('${CLAUDE_PLUGIN_ROOT}', pluginRoot);
  const source = JSON.parse(raw);

  const merged = { ...existing };
  merged.hooks = merged.hooks || {};

  const legacyRemoved = removeLegacyHooks(merged.hooks);

  let added = 0;
  let alreadyPresent = 0;

  for (const [event, entries] of Object.entries(source.hooks || {})) {
    merged.hooks[event] = merged.hooks[event] || [];
    for (const entry of entries as Array<Record<string, unknown>>) {
      const key = JSON.stringify(entry.hooks);
      const exists = merged.hooks[event].some((e: Record<string, unknown>) => JSON.stringify(e.hooks) === key);
      if (!exists) {
        merged.hooks[event].push(entry);
        added++;
      } else {
        alreadyPresent++;
      }
    }
  }

  fs.mkdirSync(path.dirname(settingsJsonPath), { recursive: true });
  fs.writeFileSync(settingsJsonPath, JSON.stringify(merged, null, 2) + '\n');

  return { added, existing: alreadyPresent, legacyRemoved };
}

// ---------------------------------------------------------------------------
// Utilities
// ---------------------------------------------------------------------------

function copyDirRecursive(src: string, dest: string): void {
  fs.mkdirSync(dest, { recursive: true });
  for (const entry of fs.readdirSync(src, { withFileTypes: true })) {
    const srcPath = path.join(src, entry.name);
    const destPath = path.join(dest, entry.name);
    if (entry.isDirectory()) {
      copyDirRecursive(srcPath, destPath);
    } else {
      fs.copyFileSync(srcPath, destPath);
    }
  }
}

/**
 * Print a merge report to stderr.
 */
export function printMergeReport(label: string, report: MergeReport): void {
  const parts: string[] = [];
  if (report.added.length > 0) parts.push(`${report.added.length} added`);
  if (report.updated.length > 0) parts.push(`${report.updated.length} updated`);
  if (report.unchanged.length > 0) parts.push(`${report.unchanged.length} unchanged`);
  if (report.skipped.length > 0) parts.push(`${report.skipped.length} skipped`);
  if (report.smartMerged.length > 0) parts.push(`${report.smartMerged.length} smart-merged`);
  if (report.errors.length > 0) parts.push(`${report.errors.length} errors`);

  if (parts.length === 0) {
    console.error(`  ${label}: (no changes)`);
  } else {
    console.error(`  ${label}: ${parts.join(', ')}`);
  }
}

/**
 * Combine multiple merge reports into one.
 */
export function combineMergeReports(...reports: MergeReport[]): MergeReport {
  return {
    added: reports.flatMap(r => r.added),
    updated: reports.flatMap(r => r.updated),
    unchanged: reports.flatMap(r => r.unchanged),
    skipped: reports.flatMap(r => r.skipped),
    smartMerged: reports.flatMap(r => r.smartMerged),
    errors: reports.flatMap(r => r.errors)
  };
}
