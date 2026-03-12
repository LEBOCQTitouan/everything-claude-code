/**
 * Clean logic for ECC installation.
 * Removes ECC-managed files before reinstall, using manifest for surgical cleanup.
 */

import fs from 'fs';
import path from 'path';
import { isLegacyEccHook } from './merge';
import type { EccManifest } from './manifest';

/** Report of what was removed, skipped, or errored during cleanup. */
export interface CleanReport {
  readonly removed: string[];
  readonly skipped: string[];
  readonly errors: string[];
}

/** ECC artifact directories relative to claudeDir. */
const ARTIFACT_DIRS = ['agents', 'commands', 'skills', 'rules'] as const;

/**
 * Remove only files listed in the manifest (surgical cleanup).
 * Returns a new CleanReport — does not mutate any inputs.
 */
export function cleanFromManifest(claudeDir: string, manifest: EccManifest, dryRun: boolean): CleanReport {
  const removed: string[] = [];
  const skipped: string[] = [];
  const errors: string[] = [];

  // Remove agent files
  for (const agent of manifest.artifacts.agents) {
    const filePath = path.join(claudeDir, 'agents', agent);
    removeFile(filePath, `agents/${agent}`, dryRun, removed, skipped, errors);
  }

  // Remove command files
  for (const command of manifest.artifacts.commands) {
    const filePath = path.join(claudeDir, 'commands', command);
    removeFile(filePath, `commands/${command}`, dryRun, removed, skipped, errors);
  }

  // Remove skill directories
  for (const skill of manifest.artifacts.skills) {
    const dirPath = path.join(claudeDir, 'skills', skill);
    removeDirectory(dirPath, `skills/${skill}`, dryRun, removed, skipped, errors);
  }

  // Remove rule files (grouped by language)
  for (const [group, files] of Object.entries(manifest.artifacts.rules)) {
    for (const file of files) {
      const filePath = path.join(claudeDir, 'rules', group, file);
      removeFile(filePath, `rules/${group}/${file}`, dryRun, removed, skipped, errors);
    }
  }

  // Remove manifest itself
  const manifestPath = path.join(claudeDir, '.ecc-manifest.json');
  removeFile(manifestPath, '.ecc-manifest.json', dryRun, removed, skipped, errors);

  return { removed, skipped, errors };
}

/**
 * Remove entire ECC directories and clean hooks from settings.json (nuclear option).
 * Returns a new CleanReport — does not mutate any inputs.
 */
export function cleanAll(claudeDir: string, dryRun: boolean): CleanReport {
  const removed: string[] = [];
  const skipped: string[] = [];
  const errors: string[] = [];

  // Remove entire artifact directories
  for (const dir of ARTIFACT_DIRS) {
    const dirPath = path.join(claudeDir, dir);
    removeDirectory(dirPath, dir, dryRun, removed, skipped, errors);
  }

  // Remove ECC hooks from settings.json
  const settingsPath = path.join(claudeDir, 'settings.json');
  if (fs.existsSync(settingsPath)) {
    try {
      const settings = JSON.parse(fs.readFileSync(settingsPath, 'utf8'));
      if (settings.hooks) {
        const hooksBefore = countHookEntries(settings.hooks);
        removeEccHooksFromSettings(settings);
        const hooksAfter = countHookEntries(settings.hooks);
        const hooksRemoved = hooksBefore - hooksAfter;

        if (hooksRemoved > 0) {
          if (!dryRun) {
            fs.writeFileSync(settingsPath, JSON.stringify(settings, null, 2) + '\n', 'utf8');
          }
          removed.push(`settings.json (${hooksRemoved} ECC hook(s))`);
        }
      }
    } catch (err) {
      errors.push(`settings.json: ${(err as Error).message}`);
    }
  }

  // Remove manifest
  const manifestPath = path.join(claudeDir, '.ecc-manifest.json');
  removeFile(manifestPath, '.ecc-manifest.json', dryRun, removed, skipped, errors);

  return { removed, skipped, errors };
}

/**
 * Print a clean report to stderr.
 */
export function printCleanReport(report: CleanReport, dryRun: boolean): void {
  const prefix = dryRun ? '[DRY RUN] Would remove' : 'Removed';

  if (report.removed.length > 0) {
    console.error(`\n${prefix}:`);
    for (const item of report.removed) {
      console.error(`  - ${item}`);
    }
  }

  if (report.skipped.length > 0) {
    console.error(`\nSkipped (not found):`);
    for (const item of report.skipped) {
      console.error(`  - ${item}`);
    }
  }

  if (report.errors.length > 0) {
    console.error(`\nErrors:`);
    for (const item of report.errors) {
      console.error(`  - ${item}`);
    }
  }

  console.error(`\nClean summary: ${report.removed.length} removed, ${report.skipped.length} skipped, ${report.errors.length} errors`);
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

function removeFile(
  filePath: string,
  label: string,
  dryRun: boolean,
  removed: string[],
  skipped: string[],
  errors: string[]
): void {
  if (!fs.existsSync(filePath)) {
    skipped.push(label);
    return;
  }
  try {
    if (!dryRun) {
      fs.unlinkSync(filePath);
    }
    removed.push(label);
  } catch (err) {
    errors.push(`${label}: ${(err as Error).message}`);
  }
}

function removeDirectory(
  dirPath: string,
  label: string,
  dryRun: boolean,
  removed: string[],
  skipped: string[],
  errors: string[]
): void {
  if (!fs.existsSync(dirPath)) {
    skipped.push(label);
    return;
  }
  try {
    if (!dryRun) {
      fs.rmSync(dirPath, { recursive: true, force: true });
    }
    removed.push(label);
  } catch (err) {
    errors.push(`${label}: ${(err as Error).message}`);
  }
}

function countHookEntries(hooks: Record<string, unknown[]>): number {
  let count = 0;
  for (const entries of Object.values(hooks)) {
    if (Array.isArray(entries)) {
      count += entries.length;
    }
  }
  return count;
}

/**
 * Remove ECC hooks from settings, preserving user-added hooks.
 * Uses isLegacyEccHook for legacy detection, plus matches current ECC hook signatures.
 */
function removeEccHooksFromSettings(settings: Record<string, unknown>): void {
  const hooks = settings.hooks as Record<string, Array<Record<string, unknown>>>;
  for (const event of Object.keys(hooks)) {
    if (!Array.isArray(hooks[event])) continue;
    hooks[event] = hooks[event].filter(entry => {
      // Remove legacy ECC hooks
      if (isLegacyEccHook(entry)) return false;
      // Remove current ECC hooks (ecc-hook / ecc-shell-hook wrappers)
      if (isCurrentEccHook(entry)) return false;
      return true;
    });
  }
}

function isCurrentEccHook(entry: Record<string, unknown>): boolean {
  const hooks = entry.hooks;
  if (!Array.isArray(hooks)) return false;

  for (const hook of hooks) {
    const cmd = (hook as Record<string, unknown>).command;
    if (typeof cmd !== 'string') continue;
    if (cmd.startsWith('ecc-hook ') || cmd.startsWith('ecc-shell-hook ')) {
      return true;
    }
  }
  return false;
}
