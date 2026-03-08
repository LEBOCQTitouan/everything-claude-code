/**
 * Merge strategies for ECC installation.
 * Handles directory merges, settings merges, and conflict resolution.
 */

import fs from 'fs';
import path from 'path';
import readline from 'readline';
import { isEccManaged, isEccManagedRule } from './manifest';
import { generateDiff, smartMerge, isClaudeAvailable } from './smart-merge';
import type { EccManifest } from './manifest';

export type ConflictChoice = 'overwrite' | 'keep' | 'diff' | 'smart-merge';
export type ConflictApplyAll = ConflictChoice | null;

export interface MergeReport {
  added: string[];
  updated: string[];
  unchanged: string[];
  skipped: string[];
  smartMerged: string[];
  errors: string[];
}

export interface MergeOptions {
  dryRun: boolean;
  force: boolean;
  interactive: boolean; // prompt user on conflicts
  applyAll: ConflictApplyAll; // cached choice for "apply to all"
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

/**
 * Prompt user for conflict resolution choice.
 * Returns the choice and whether to apply to all.
 */
export async function promptConflict(filename: string, existingPath: string, incomingPath: string): Promise<{ choice: ConflictChoice; applyAll: boolean }> {
  const rl = readline.createInterface({
    input: process.stdin,
    output: process.stderr, // stderr so it's visible to user
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
            // Show diff then re-ask
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

/**
 * Handle a single file conflict resolution.
 */
async function resolveConflict(filename: string, srcPath: string, destPath: string, options: MergeOptions, report: MergeReport): Promise<MergeOptions> {
  // Non-interactive: skip
  if (!options.interactive) {
    report.skipped.push(filename);
    return options;
  }

  // Apply-all cached choice
  if (options.applyAll !== null) {
    return applyChoice(options.applyAll, filename, srcPath, destPath, options, report);
  }

  // Interactive prompt
  const { choice, applyAll } = await promptConflict(filename, destPath, srcPath);
  const newOptions = applyAll ? { ...options, applyAll: choice } : options;

  return applyChoice(choice, filename, srcPath, destPath, newOptions, report);
}

/**
 * Apply a conflict resolution choice.
 */
function applyChoice(choice: ConflictChoice, filename: string, srcPath: string, destPath: string, options: MergeOptions, report: MergeReport): MergeOptions {
  switch (choice) {
    case 'overwrite':
      if (!options.dryRun) {
        fs.copyFileSync(srcPath, destPath);
      }
      report.updated.push(filename);
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

    case 'diff':
      // Diff is handled in promptConflict — shouldn't reach here
      report.skipped.push(filename);
      break;
  }
  return options;
}

/**
 * Merge a directory of files from source to destination.
 * Uses manifest to determine ownership.
 */
export async function mergeDirectory(srcDir: string, destDir: string, manifest: EccManifest | null, artifactType: 'agents' | 'commands', options: MergeOptions): Promise<MergeReport> {
  const report: MergeReport = { added: [], updated: [], unchanged: [], skipped: [], smartMerged: [], errors: [] };

  if (!fs.existsSync(srcDir)) return report;

  const srcFiles = fs.readdirSync(srcDir).filter(f => f.endsWith('.md'));
  if (!options.dryRun) {
    fs.mkdirSync(destDir, { recursive: true });
  }

  let currentOptions = { ...options };

  for (const filename of srcFiles) {
    const srcPath = path.join(srcDir, filename);
    const destPath = path.join(destDir, filename);

    if (!fs.existsSync(destPath)) {
      // New file — add it
      if (!currentOptions.dryRun) {
        fs.copyFileSync(srcPath, destPath);
      }
      report.added.push(filename);
    } else if (isEccManaged(manifest, artifactType, filename) || currentOptions.force) {
      // ECC-managed or force mode — update it
      if (!currentOptions.dryRun) {
        fs.copyFileSync(srcPath, destPath);
      }
      report.updated.push(filename);
    } else {
      // Conflict: file exists but not in manifest
      currentOptions = await resolveConflict(filename, srcPath, destPath, currentOptions, report);
    }
  }

  return report;
}

/**
 * Merge skills directory (skill-level granularity, not file-level).
 * Each skill is a directory that is merged atomically.
 */
export async function mergeSkills(srcDir: string, destDir: string, manifest: EccManifest | null, options: MergeOptions): Promise<MergeReport> {
  const report: MergeReport = { added: [], updated: [], unchanged: [], skipped: [], smartMerged: [], errors: [] };

  if (!fs.existsSync(srcDir)) return report;

  const srcSkills = fs
    .readdirSync(srcDir, { withFileTypes: true })
    .filter(e => e.isDirectory())
    .map(e => e.name);

  if (!options.dryRun) {
    fs.mkdirSync(destDir, { recursive: true });
  }

  let currentOptions = { ...options };

  for (const skillName of srcSkills) {
    const srcSkillDir = path.join(srcDir, skillName);
    const destSkillDir = path.join(destDir, skillName);

    if (!fs.existsSync(destSkillDir)) {
      // New skill — copy entire directory
      if (!currentOptions.dryRun) {
        copyDirRecursive(srcSkillDir, destSkillDir);
      }
      report.added.push(skillName);
    } else if (isEccManaged(manifest, 'skills', skillName) || currentOptions.force) {
      // ECC-managed skill — replace entire directory
      if (!currentOptions.dryRun) {
        copyDirRecursive(srcSkillDir, destSkillDir);
      }
      report.updated.push(skillName);
    } else {
      // Conflict: skill exists but not in manifest
      // For skills, we can't smart-merge a whole directory, so just overwrite/keep
      currentOptions = await resolveConflict(
        skillName + '/',
        path.join(srcSkillDir, 'SKILL.md'),
        fs.existsSync(path.join(destSkillDir, 'SKILL.md')) ? path.join(destSkillDir, 'SKILL.md') : destSkillDir,
        currentOptions,
        report
      );

      // If overwrite was chosen for the skill, copy the whole directory
      if (report.updated.includes(skillName + '/')) {
        if (!currentOptions.dryRun) {
          copyDirRecursive(srcSkillDir, destSkillDir);
        }
      }
    }
  }

  return report;
}

/**
 * Merge rules directory, grouped by subdirectory.
 */
export async function mergeRules(srcDir: string, destDir: string, manifest: EccManifest | null, groups: string[], options: MergeOptions): Promise<MergeReport> {
  const report: MergeReport = { added: [], updated: [], unchanged: [], skipped: [], smartMerged: [], errors: [] };

  let currentOptions = { ...options };

  for (const group of groups) {
    const srcGroupDir = path.join(srcDir, group);
    const destGroupDir = path.join(destDir, group);

    if (!fs.existsSync(srcGroupDir)) continue;

    const srcFiles = fs.readdirSync(srcGroupDir).filter(f => f.endsWith('.md'));
    if (!currentOptions.dryRun) {
      fs.mkdirSync(destGroupDir, { recursive: true });
    }

    for (const filename of srcFiles) {
      const srcPath = path.join(srcGroupDir, filename);
      const destPath = path.join(destGroupDir, filename);
      const label = `${group}/${filename}`;

      if (!fs.existsSync(destPath)) {
        if (!currentOptions.dryRun) {
          fs.copyFileSync(srcPath, destPath);
        }
        report.added.push(label);
      } else if (isEccManagedRule(manifest, group, filename) || currentOptions.force) {
        if (!currentOptions.dryRun) {
          fs.copyFileSync(srcPath, destPath);
        }
        report.updated.push(label);
      } else {
        currentOptions = await resolveConflict(label, srcPath, destPath, currentOptions, report);
      }
    }
  }

  return report;
}

/**
 * Check if a hook entry is a legacy ECC hook that should be removed.
 * Legacy hooks reference `scripts/hooks/` (a path that no longer exists)
 * or use inline `node -e` one-liners from older ECC versions.
 */
export function isLegacyEccHook(entry: Record<string, unknown>): boolean {
  const hooks = entry.hooks;
  if (!Array.isArray(hooks)) return false;

  for (const hook of hooks) {
    const cmd = (hook as Record<string, unknown>).command;
    if (typeof cmd !== 'string') continue;

    // Legacy path: scripts/hooks/ (was never correct in npm package)
    if (cmd.includes('scripts/hooks/') && !cmd.includes('run-with-flags-shell.sh')) {
      return true;
    }

    // Legacy inline one-liners from old ECC versions
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

/**
 * Remove legacy ECC hook entries from a hooks object.
 * Returns the number of entries removed.
 */
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
 * Preserves all existing keys, removes legacy ECC hooks, and deduplicates.
 */
export function mergeHooks(hooksJsonPath: string, settingsJsonPath: string, pluginRoot: string): { added: number; existing: number; legacyRemoved: number } {
  const existing = fs.existsSync(settingsJsonPath) ? JSON.parse(fs.readFileSync(settingsJsonPath, 'utf8')) : {};

  const raw = fs.readFileSync(hooksJsonPath, 'utf8').replaceAll('${CLAUDE_PLUGIN_ROOT}', pluginRoot);
  const source = JSON.parse(raw);

  const merged = { ...existing };
  merged.hooks = merged.hooks || {};

  // Clean up legacy ECC hooks before merging new ones
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

/**
 * Recursively copy a directory.
 */
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
 * Print a merge report to stderr (visible to user).
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
