/**
 * ECC configuration audit module.
 *
 * Compares installed ECC artifacts against the source of truth and produces
 * structured diffs. Replaces the fragile legacy-pattern detection with a
 * source-of-truth comparison approach.
 */

import fs from 'fs';
import path from 'path';
import { contentsDiffer } from './smart-merge';
import { bold, dim, red, green, yellow } from './ansi';

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/** A hook entry identified as stale or missing, with its event type. */
export interface HookDiffEntry {
  event: string;
  entry: Record<string, unknown>;
}

/** Result of comparing hooks in settings.json against hooks.json source. */
export interface HooksDiff {
  /** ECC hooks in settings that are NOT in the source (should be removed). */
  stale: HookDiffEntry[];
  /** Source hooks that are NOT in settings (should be added). */
  missing: HookDiffEntry[];
  /** Hooks that match between settings and source. */
  matching: HookDiffEntry[];
  /** Non-ECC user hooks (left untouched). */
  userHooks: HookDiffEntry[];
}

/** Audit result for a file-based artifact category (agents, commands). */
export interface ArtifactAudit {
  matching: string[];
  outdated: string[];
  missing: string[];
}

/** Full config audit result. */
export interface ConfigAudit {
  agents: ArtifactAudit;
  commands: ArtifactAudit;
  hooks: HooksDiff;
  hasDifferences: boolean;
}

// ---------------------------------------------------------------------------
// ECC package identification
// ---------------------------------------------------------------------------

/** Known ECC package identifiers in npm paths. */
const ECC_PACKAGE_IDENTIFIERS = [
  '@lebocqtitouan/ecc/',
  'everything-claude-code/'
];

// ---------------------------------------------------------------------------
// isEccManagedHook
// ---------------------------------------------------------------------------

/**
 * Determine if a hook entry is managed by ECC.
 *
 * A hook is ECC-managed if any of its commands:
 * 1. Start with `ecc-hook` or `ecc-shell-hook` (current format)
 * 2. Contain a known ECC package identifier in an absolute path
 * 3. Match any entry in the provided source hooks
 * 4. Match legacy patterns (inline node -e, placeholders, etc.)
 */
export function isEccManagedHook(
  entry: Record<string, unknown>,
  sourceHooks: Record<string, Array<Record<string, unknown>>>
): boolean {
  const hooks = entry.hooks;
  if (!Array.isArray(hooks) || hooks.length === 0) return false;

  for (const hook of hooks) {
    const cmd = (hook as Record<string, unknown>).command;
    if (typeof cmd !== 'string') continue;

    // 1. Current ecc-hook / ecc-shell-hook wrappers
    if (cmd.startsWith('ecc-hook ') || cmd.startsWith('ecc-shell-hook ')) {
      return true;
    }

    // 2. Absolute path containing ECC package identifier
    for (const identifier of ECC_PACKAGE_IDENTIFIERS) {
      if (cmd.includes(identifier)) {
        return true;
      }
    }

    // 3. Check if this hook matches any entry in the source hooks.json
    if (matchesSourceHook(entry, sourceHooks)) {
      return true;
    }

    // 4. Legacy patterns (fallback for edge cases)
    if (isLegacyPattern(cmd)) {
      return true;
    }
  }

  return false;
}

/**
 * Check if an entry matches any hook in the source hooks.json (by command content).
 */
function matchesSourceHook(
  entry: Record<string, unknown>,
  sourceHooks: Record<string, Array<Record<string, unknown>>>
): boolean {
  const entryKey = JSON.stringify(entry.hooks);
  for (const entries of Object.values(sourceHooks)) {
    if (!Array.isArray(entries)) continue;
    for (const sourceEntry of entries) {
      if (JSON.stringify(sourceEntry.hooks) === entryKey) {
        return true;
      }
    }
  }
  return false;
}

/**
 * Legacy pattern detection — kept as fallback for hooks that predate
 * both the current wrapper format and the npm package path format.
 */
function isLegacyPattern(cmd: string): boolean {
  // Old-style scripts/hooks/ direct paths
  if (cmd.includes('scripts/hooks/') && !cmd.includes('run-with-flags-shell.sh')) {
    return true;
  }

  // Unresolved placeholder commands
  if (cmd.includes('${ECC_ROOT}') || cmd.includes('${CLAUDE_PLUGIN_ROOT}')) {
    return true;
  }

  // Resolved absolute-path run-with-flags.js (not via ecc-hook wrapper)
  if (cmd.includes('/dist/hooks/run-with-flags.js') && !cmd.startsWith('ecc-hook')) {
    return true;
  }

  // Resolved absolute-path shell hook commands
  if (cmd.includes('/scripts/hooks/run-with-flags-shell.sh') && !cmd.startsWith('ecc-shell-hook')) {
    return true;
  }

  // Inline node -e one-liners from pre-hook-runner era
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

  return false;
}

// ---------------------------------------------------------------------------
// diffHooks
// ---------------------------------------------------------------------------

/**
 * Compare hooks in a settings.json against the source hooks.json.
 * Returns a structured diff showing stale, missing, matching, and user hooks.
 */
export function diffHooks(
  settingsJsonPath: string,
  hooksJsonPath: string
): HooksDiff {
  const settingsHooks = readHooksFromSettings(settingsJsonPath);
  const sourceHooks = readHooksFromSource(hooksJsonPath);

  const stale: HookDiffEntry[] = [];
  const matching: HookDiffEntry[] = [];
  const userHooks: HookDiffEntry[] = [];

  // Check each hook in settings: is it ECC-managed? Does it match source?
  for (const [event, entries] of Object.entries(settingsHooks)) {
    if (!Array.isArray(entries)) continue;
    for (const entry of entries) {
      if (isEccManagedHook(entry, sourceHooks)) {
        if (existsInSource(event, entry, sourceHooks)) {
          matching.push({ event, entry });
        } else {
          stale.push({ event, entry });
        }
      } else {
        userHooks.push({ event, entry });
      }
    }
  }

  // Check source for hooks missing from settings
  const missing: HookDiffEntry[] = [];
  for (const [event, entries] of Object.entries(sourceHooks)) {
    if (!Array.isArray(entries)) continue;
    for (const entry of entries) {
      if (!existsInSettings(event, entry, settingsHooks)) {
        missing.push({ event, entry });
      }
    }
  }

  return { stale, missing, matching, userHooks };
}

/**
 * Check if a source hook entry exists in the settings hooks.
 */
function existsInSettings(
  event: string,
  sourceEntry: Record<string, unknown>,
  settingsHooks: Record<string, Array<Record<string, unknown>>>
): boolean {
  const entries = settingsHooks[event];
  if (!Array.isArray(entries)) return false;
  const key = JSON.stringify(sourceEntry.hooks);
  return entries.some(e => JSON.stringify(e.hooks) === key);
}

/**
 * Check if a settings hook entry exists in the source hooks.
 */
function existsInSource(
  event: string,
  settingsEntry: Record<string, unknown>,
  sourceHooks: Record<string, Array<Record<string, unknown>>>
): boolean {
  const entries = sourceHooks[event];
  if (!Array.isArray(entries)) return false;
  const key = JSON.stringify(settingsEntry.hooks);
  return entries.some(e => JSON.stringify(e.hooks) === key);
}

// ---------------------------------------------------------------------------
// auditEccConfig
// ---------------------------------------------------------------------------

/**
 * Compare all installed ECC artifacts against the source of truth.
 * Returns a structured audit report.
 */
export function auditEccConfig(eccRoot: string, claudeDir: string): ConfigAudit {
  const agents = auditArtifactDir(
    path.join(eccRoot, 'agents'),
    path.join(claudeDir, 'agents'),
    '.md'
  );

  const commands = auditArtifactDir(
    path.join(eccRoot, 'commands'),
    path.join(claudeDir, 'commands'),
    '.md'
  );

  const hooksJsonPath = path.join(eccRoot, 'hooks', 'hooks.json');
  const settingsJsonPath = path.join(claudeDir, 'settings.json');
  const hooks = diffHooks(settingsJsonPath, hooksJsonPath);

  const hasDifferences =
    agents.outdated.length > 0 ||
    agents.missing.length > 0 ||
    commands.outdated.length > 0 ||
    commands.missing.length > 0 ||
    hooks.stale.length > 0 ||
    hooks.missing.length > 0;

  return { agents, commands, hooks, hasDifferences };
}

/**
 * Compare files in a source directory against an installed directory.
 */
function auditArtifactDir(srcDir: string, destDir: string, ext: string): ArtifactAudit {
  const matching: string[] = [];
  const outdated: string[] = [];
  const missing: string[] = [];

  if (!fs.existsSync(srcDir)) return { matching, outdated, missing };

  const srcFiles = fs.readdirSync(srcDir).filter(f => f.endsWith(ext));

  for (const filename of srcFiles) {
    const srcPath = path.join(srcDir, filename);
    const destPath = path.join(destDir, filename);

    if (!fs.existsSync(destPath)) {
      missing.push(filename);
    } else if (contentsDiffer(srcPath, destPath)) {
      outdated.push(filename);
    } else {
      matching.push(filename);
    }
  }

  return { matching, outdated, missing };
}

// ---------------------------------------------------------------------------
// Display
// ---------------------------------------------------------------------------

/**
 * Print a hooks diff to stderr with colored output.
 */
export function printHooksDiff(diff: HooksDiff): void {
  if (diff.stale.length === 0 && diff.missing.length === 0) {
    console.error(dim('  Hooks: all in sync'));
    return;
  }

  console.error(bold('  Hooks diff:'));

  for (const { event, entry } of diff.stale) {
    const cmd = extractCommand(entry);
    console.error(red(`    - [${event}] ${cmd}`));
  }

  for (const { event, entry } of diff.missing) {
    const cmd = extractCommand(entry);
    console.error(green(`    + [${event}] ${cmd}`));
  }

  if (diff.matching.length > 0) {
    console.error(dim(`    = ${diff.matching.length} hook(s) in sync`));
  }

  if (diff.userHooks.length > 0) {
    console.error(dim(`    ~ ${diff.userHooks.length} user hook(s) preserved`));
  }
}

/**
 * Print a full config audit to stderr.
 */
export function printConfigAudit(audit: ConfigAudit): void {
  if (!audit.hasDifferences) {
    console.error(dim('ECC config is in sync.'));
    return;
  }

  console.error(yellow(bold('ECC config differences detected:')));
  printArtifactAudit('Agents', audit.agents);
  printArtifactAudit('Commands', audit.commands);
  printHooksDiff(audit.hooks);
}

function printArtifactAudit(label: string, audit: ArtifactAudit): void {
  if (audit.outdated.length === 0 && audit.missing.length === 0) {
    console.error(dim(`  ${label}: all in sync`));
    return;
  }

  console.error(bold(`  ${label} diff:`));
  for (const f of audit.outdated) {
    console.error(yellow(`    ~ ${f} (outdated)`));
  }
  for (const f of audit.missing) {
    console.error(green(`    + ${f} (missing)`));
  }
  if (audit.matching.length > 0) {
    console.error(dim(`    = ${audit.matching.length} file(s) in sync`));
  }
}

/**
 * Extract the command string from a hook entry for display.
 */
function extractCommand(entry: Record<string, unknown>): string {
  const hooks = entry.hooks;
  if (!Array.isArray(hooks) || hooks.length === 0) return '(unknown)';
  const cmd = (hooks[0] as Record<string, unknown>).command;
  if (typeof cmd !== 'string') return '(unknown)';
  // Truncate long commands for display
  return cmd.length > 80 ? cmd.slice(0, 77) + '...' : cmd;
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function readHooksFromSettings(settingsPath: string): Record<string, Array<Record<string, unknown>>> {
  if (!fs.existsSync(settingsPath)) return {};
  try {
    const data = JSON.parse(fs.readFileSync(settingsPath, 'utf8'));
    return data.hooks || {};
  } catch {
    return {};
  }
}

function readHooksFromSource(hooksJsonPath: string): Record<string, Array<Record<string, unknown>>> {
  try {
    const data = JSON.parse(fs.readFileSync(hooksJsonPath, 'utf8'));
    return data.hooks || {};
  } catch {
    return {};
  }
}
