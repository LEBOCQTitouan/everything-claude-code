/**
 * Install orchestrator: wires detection, manifest, merge, and gitignore
 * into a cohesive install/init flow.
 *
 * Called by install.sh or directly via: node dist/install-orchestrator.js <command> [options]
 */

import fs from 'fs';
import path from 'path';
import { detect, generateReport } from './lib/detect';
import { readManifest, writeManifest, createManifest, updateManifest } from './lib/manifest';
import { mergeDirectory, mergeSkills, mergeRules, mergeHooks, printMergeReport, combineMergeReports, defaultMergeOptions } from './lib/merge';
import { ensureGitignoreEntries, findTrackedEccFiles, gitUntrack } from './lib/gitignore';
import { auditEccConfig, printConfigAudit } from './lib/config-audit';
import type { MergeOptions } from './lib/merge';
import type { EccManifest } from './lib/manifest';

interface OrchestratorOptions {
  dryRun: boolean;
  force: boolean;
  noGitignore: boolean;
  interactive: boolean;
}

function parseArgs(args: string[]): { command: string; languages: string[]; options: OrchestratorOptions } {
  const options: OrchestratorOptions = {
    dryRun: false,
    force: false,
    noGitignore: false,
    interactive: process.stdin.isTTY === true
  };

  const positional: string[] = [];

  for (let i = 0; i < args.length; i++) {
    switch (args[i]) {
      case '--dry-run':
        options.dryRun = true;
        break;
      case '--force':
        options.force = true;
        break;
      case '--no-gitignore':
        options.noGitignore = true;
        break;
      case '--no-interactive':
        options.interactive = false;
        break;
      default:
        positional.push(args[i]);
    }
  }

  const command = positional[0] || '';
  const languages = positional.slice(1);

  return { command, languages, options };
}

/**
 * Get the ECC root directory (where install.sh / package.json lives).
 */
function getEccRoot(): string {
  // When running from dist/, go up one level
  return path.resolve(__dirname, '..');
}

/**
 * Get ECC version from package.json.
 */
function getVersion(): string {
  const pkgPath = path.join(getEccRoot(), 'package.json');
  try {
    return JSON.parse(fs.readFileSync(pkgPath, 'utf8')).version || '0.0.0';
  } catch {
    return '0.0.0';
  }
}

/**
 * Collect the list of ECC source artifacts for manifest tracking.
 */
function collectSourceArtifacts(eccRoot: string, languages: string[]): EccManifest['artifacts'] {
  const agentsDir = path.join(eccRoot, 'agents');
  const commandsDir = path.join(eccRoot, 'commands');
  const skillsDir = path.join(eccRoot, 'skills');
  const rulesDir = path.join(eccRoot, 'rules');
  const hooksFile = path.join(eccRoot, 'hooks', 'hooks.json');

  const agents = fs.existsSync(agentsDir) ? fs.readdirSync(agentsDir).filter(f => f.endsWith('.md')) : [];

  const commands = fs.existsSync(commandsDir) ? fs.readdirSync(commandsDir).filter(f => f.endsWith('.md')) : [];

  const skills = fs.existsSync(skillsDir)
    ? fs
        .readdirSync(skillsDir, { withFileTypes: true })
        .filter(e => e.isDirectory())
        .map(e => e.name)
    : [];

  const rules: Record<string, string[]> = {};
  const ruleGroups = ['common', ...languages];
  for (const group of ruleGroups) {
    const groupDir = path.join(rulesDir, group);
    if (fs.existsSync(groupDir)) {
      rules[group] = fs.readdirSync(groupDir).filter(f => f.endsWith('.md'));
    }
  }

  let hookDescriptions: string[] = [];
  try {
    const hooks = JSON.parse(fs.readFileSync(hooksFile, 'utf8'));
    for (const entries of Object.values(hooks.hooks || {})) {
      for (const entry of entries as Array<Record<string, unknown>>) {
        if (entry.description) {
          hookDescriptions.push(entry.description as string);
        }
      }
    }
  } catch {
    hookDescriptions = [];
  }

  return { agents, commands, skills, rules, hookDescriptions };
}

/**
 * Run the global install flow.
 */
async function installGlobal(languages: string[], opts: OrchestratorOptions): Promise<boolean> {
  const eccRoot = getEccRoot();
  const claudeDir = process.env.CLAUDE_DIR || path.join(process.env.HOME || '', '.claude');
  const version = getVersion();

  if (opts.dryRun) {
    console.error('[DRY RUN] No files will be written.\n');
  }

  // Step 1: Detect existing setup
  const detection = detect(claudeDir);
  console.error(generateReport(detection));
  console.error('');

  // Step 2: Read existing manifest
  const existingManifest = readManifest(claudeDir);
  if (existingManifest) {
    console.error(`ECC manifest found (v${existingManifest.version}, updated ${existingManifest.updatedAt})`);
  } else {
    console.error('No ECC manifest found — first install or legacy setup.');
  }
  console.error('');

  // Step 3: Audit current config before merging
  const audit = auditEccConfig(eccRoot, claudeDir);
  printConfigAudit(audit);
  console.error('');

  // Step 4: Set up merge options
  const mergeOpts: MergeOptions = {
    ...defaultMergeOptions(),
    dryRun: opts.dryRun,
    force: opts.force,
    interactive: opts.interactive
  };

  // Step 5: Merge each artifact type
  console.error('Installing ECC artifacts:');

  const agentsReport = await mergeDirectory(path.join(eccRoot, 'agents'), path.join(claudeDir, 'agents'), existingManifest, 'agents', mergeOpts);
  printMergeReport('Agents', agentsReport);

  const commandsReport = await mergeDirectory(path.join(eccRoot, 'commands'), path.join(claudeDir, 'commands'), existingManifest, 'commands', mergeOpts);
  printMergeReport('Commands', commandsReport);

  const skillsReport = await mergeSkills(path.join(eccRoot, 'skills'), path.join(claudeDir, 'skills'), existingManifest, mergeOpts);
  printMergeReport('Skills', skillsReport);

  const ruleGroups = ['common', ...languages];
  const rulesReport = await mergeRules(path.join(eccRoot, 'rules'), path.join(claudeDir, 'rules'), existingManifest, ruleGroups, mergeOpts);
  printMergeReport('Rules', rulesReport);

  // Step 6: Merge hooks
  if (!opts.dryRun) {
    const hooksResult = mergeHooks(path.join(eccRoot, 'hooks', 'hooks.json'), path.join(claudeDir, 'settings.json'), eccRoot);
    const hookParts = [`${hooksResult.added} added`, `${hooksResult.existing} already present`];
    if (hooksResult.legacyRemoved > 0) hookParts.push(`${hooksResult.legacyRemoved} legacy removed`);
    console.error(`  Hooks: ${hookParts.join(', ')}`);
  } else {
    console.error('  Hooks: (dry-run, skipped)');
  }

  // Step 7: Write/update manifest
  const sourceArtifacts = collectSourceArtifacts(eccRoot, languages);

  if (!opts.dryRun) {
    const newManifest = existingManifest ? updateManifest(existingManifest, version, languages, sourceArtifacts) : createManifest(version, languages, sourceArtifacts);
    writeManifest(claudeDir, newManifest);
    console.error(`\nManifest ${existingManifest ? 'updated' : 'created'} at ${claudeDir}/.ecc-manifest.json`);
  }

  // Step 8: Summary
  const combined = combineMergeReports(agentsReport, commandsReport, skillsReport, rulesReport);
  console.error('\nSummary:');
  console.error(`  Added:        ${combined.added.length}`);
  console.error(`  Updated:      ${combined.updated.length}`);
  console.error(`  Unchanged:    ${combined.unchanged.length}`);
  console.error(`  Skipped:      ${combined.skipped.length}`);
  console.error(`  Smart-merged: ${combined.smartMerged.length}`);
  if (combined.errors.length > 0) {
    console.error(`  Errors:       ${combined.errors.length}`);
  }

  return combined.errors.length === 0;
}

/**
 * Run the project init flow.
 */
async function initProject(opts: OrchestratorOptions): Promise<boolean> {
  const projectDir = process.cwd();

  if (opts.dryRun) {
    console.error('[DRY RUN] No files will be written.\n');
  }

  // Gitignore management
  if (!opts.noGitignore) {
    console.error('Managing .gitignore:');
    if (!opts.dryRun) {
      const gitResult = ensureGitignoreEntries(projectDir);
      if (gitResult.skipped) {
        console.error('  Not a git repo — skipping .gitignore management.');
      } else if (gitResult.added.length === 0) {
        console.error('  All ECC entries already present in .gitignore.');
      } else {
        console.error(`  Added ${gitResult.added.length} entries to .gitignore:`);
        for (const entry of gitResult.added) {
          console.error(`    + ${entry}`);
        }
      }

      // Check for tracked files that should be ignored
      const tracked = findTrackedEccFiles(projectDir);
      if (tracked.length > 0 && opts.interactive) {
        console.error(`\n  Warning: ${tracked.length} ECC-generated file(s) are tracked by git:`);
        for (const t of tracked) {
          console.error(`    - ${t}`);
        }

        const readline = await import('readline');
        const rl = readline.createInterface({
          input: process.stdin,
          output: process.stderr,
          terminal: true
        });

        const answer = await new Promise<string>(resolve => {
          rl.question('  Untrack these files? (git rm --cached) [y/N] ', resolve);
        });
        rl.close();

        if (answer.trim().toLowerCase() === 'y') {
          for (const t of tracked) {
            gitUntrack(projectDir, t);
            console.error(`    Untracked: ${t}`);
          }
        }
      }
    } else {
      console.error('  (dry-run, skipped)');
    }
  }

  return true;
}

// ---------------------------------------------------------------------------
// CLI entry point
// ---------------------------------------------------------------------------
async function main(): Promise<void> {
  const { command, languages, options } = parseArgs(process.argv.slice(2));

  let success = false;

  switch (command) {
    case 'install':
      success = await installGlobal(languages, options);
      break;

    case 'init':
      success = await initProject(options);
      break;

    default:
      console.error(`Unknown orchestrator command: ${command}`);
      console.error('Usage: install-orchestrator.js <install|init> [options]');
      process.exit(1);
  }

  process.exit(success ? 0 : 1);
}

main().catch(err => {
  console.error(`Fatal error: ${err.message}`);
  process.exit(1);
});
