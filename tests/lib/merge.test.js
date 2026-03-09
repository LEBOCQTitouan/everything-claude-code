/**
 * Tests for src/lib/merge.ts
 *
 * Run with: npx tsx tests/lib/merge.test.js
 */

const assert = require('assert');
const fs = require('fs');
const path = require('path');
const os = require('os');

const { mergeDirectory, mergeSkills, mergeRules, mergeHooks, isLegacyEccHook, combineMergeReports, defaultMergeOptions } = require('../../src/lib/merge');

const { createManifest } = require('../../src/lib/manifest');
const { test, describe } = require('../harness');

function makeTempDir() {
  return fs.mkdtempSync(path.join(os.tmpdir(), 'ecc-merge-test-'));
}

function cleanup(dir) {
  fs.rmSync(dir, { recursive: true, force: true });
}

function sampleArtifacts() {
  return {
    agents: ['planner.md', 'architect.md'],
    commands: ['tdd.md'],
    skills: ['tdd-workflow'],
    rules: { common: ['coding-style.md'] },
    hookDescriptions: []
  };
}

async function runTests() {
  describe('Testing merge.ts');

  const tmpDir = makeTempDir();

  try {
    const nonInteractiveOpts = { ...defaultMergeOptions(), interactive: false };

    // --- mergeDirectory ---
    describe('mergeDirectory');

    await test('adds new files to empty destination', async () => {
      const srcDir = path.join(tmpDir, 'src-agents');
      const destDir = path.join(tmpDir, 'dest-agents-1');
      fs.mkdirSync(srcDir, { recursive: true });
      fs.writeFileSync(path.join(srcDir, 'planner.md'), '# Planner');
      fs.writeFileSync(path.join(srcDir, 'architect.md'), '# Architect');

      const report = await mergeDirectory(srcDir, destDir, null, 'agents', nonInteractiveOpts);
      assert.strictEqual(report.added.length, 2);
      assert.strictEqual(report.updated.length, 0);
      assert.ok(fs.existsSync(path.join(destDir, 'planner.md')));
      assert.ok(fs.existsSync(path.join(destDir, 'architect.md')));
    });

    await test('updates ECC-managed files', async () => {
      const srcDir = path.join(tmpDir, 'src-agents');
      const destDir = path.join(tmpDir, 'dest-agents-2');
      fs.mkdirSync(destDir, { recursive: true });
      fs.writeFileSync(path.join(destDir, 'planner.md'), '# Old Planner');

      const manifest = createManifest('1.0.0', ['ts'], sampleArtifacts());
      const report = await mergeDirectory(srcDir, destDir, manifest, 'agents', nonInteractiveOpts);

      assert.ok(report.updated.includes('planner.md'));
      const content = fs.readFileSync(path.join(destDir, 'planner.md'), 'utf8');
      assert.strictEqual(content, '# Planner');
    });

    await test('updates changed files in non-interactive mode', async () => {
      const srcDir = path.join(tmpDir, 'src-agents');
      const destDir = path.join(tmpDir, 'dest-agents-3');
      fs.mkdirSync(destDir, { recursive: true });
      fs.writeFileSync(path.join(destDir, 'planner.md'), '# Custom Planner');

      // Non-interactive mode accepts all changed files (no manifest distinction)
      const report = await mergeDirectory(srcDir, destDir, null, 'agents', nonInteractiveOpts);

      assert.ok(report.updated.includes('planner.md'));
      const content = fs.readFileSync(path.join(destDir, 'planner.md'), 'utf8');
      assert.strictEqual(content, '# Planner');
    });

    await test('reports unchanged files separately', async () => {
      const srcDir = path.join(tmpDir, 'src-agents');
      const destDir = path.join(tmpDir, 'dest-agents-unchanged');
      fs.mkdirSync(destDir, { recursive: true });
      // Write identical content to dest
      fs.writeFileSync(path.join(destDir, 'planner.md'), '# Planner');
      fs.writeFileSync(path.join(destDir, 'architect.md'), '# Architect');

      const report = await mergeDirectory(srcDir, destDir, null, 'agents', nonInteractiveOpts);

      assert.strictEqual(report.unchanged.length, 2);
      assert.ok(report.unchanged.includes('planner.md'));
      assert.ok(report.unchanged.includes('architect.md'));
      assert.strictEqual(report.added.length, 0);
      assert.strictEqual(report.updated.length, 0);
    });

    await test('applyAll accept updates all remaining files', async () => {
      const srcDir = path.join(tmpDir, 'src-agents');
      const destDir = path.join(tmpDir, 'dest-agents-applyall');
      fs.mkdirSync(destDir, { recursive: true });
      fs.writeFileSync(path.join(destDir, 'planner.md'), '# Old Planner');
      fs.writeFileSync(path.join(destDir, 'architect.md'), '# Old Architect');

      const opts = { ...nonInteractiveOpts, applyAll: 'accept' };
      const report = await mergeDirectory(srcDir, destDir, null, 'agents', opts);

      assert.strictEqual(report.updated.length, 2);
      assert.strictEqual(report.skipped.length, 0);
    });

    await test('applyAll keep skips all remaining files', async () => {
      const srcDir = path.join(tmpDir, 'src-agents');
      const destDir = path.join(tmpDir, 'dest-agents-keepall');
      fs.mkdirSync(destDir, { recursive: true });
      fs.writeFileSync(path.join(destDir, 'planner.md'), '# Old Planner');
      fs.writeFileSync(path.join(destDir, 'architect.md'), '# Old Architect');

      const opts = { ...nonInteractiveOpts, applyAll: 'keep' };
      const report = await mergeDirectory(srcDir, destDir, null, 'agents', opts);

      assert.strictEqual(report.skipped.length, 2);
      assert.strictEqual(report.updated.length, 0);
    });

    await test('force mode overwrites everything', async () => {
      const srcDir = path.join(tmpDir, 'src-agents');
      const destDir = path.join(tmpDir, 'dest-agents-4');
      fs.mkdirSync(destDir, { recursive: true });
      fs.writeFileSync(path.join(destDir, 'planner.md'), '# Custom Planner');

      const forceOpts = { ...nonInteractiveOpts, force: true };
      const report = await mergeDirectory(srcDir, destDir, null, 'agents', forceOpts);

      assert.ok(report.updated.includes('planner.md'));
      const content = fs.readFileSync(path.join(destDir, 'planner.md'), 'utf8');
      assert.strictEqual(content, '# Planner');
    });

    await test('dry-run does not write files', async () => {
      const srcDir = path.join(tmpDir, 'src-agents');
      const destDir = path.join(tmpDir, 'dest-agents-5-dryrun');

      const dryRunOpts = { ...nonInteractiveOpts, dryRun: true };
      const report = await mergeDirectory(srcDir, destDir, null, 'agents', dryRunOpts);

      assert.strictEqual(report.added.length, 2);
      assert.ok(!fs.existsSync(destDir));
    });

    await test('handles non-existent source dir', async () => {
      const report = await mergeDirectory(path.join(tmpDir, 'nonexistent-src'), path.join(tmpDir, 'dest-x'), null, 'agents', nonInteractiveOpts);
      assert.strictEqual(report.added.length, 0);
    });

    // --- mergeSkills ---
    describe('mergeSkills');

    await test('adds new skills', async () => {
      const srcDir = path.join(tmpDir, 'src-skills');
      const destDir = path.join(tmpDir, 'dest-skills-1');
      fs.mkdirSync(path.join(srcDir, 'tdd-workflow'), { recursive: true });
      fs.writeFileSync(path.join(srcDir, 'tdd-workflow', 'SKILL.md'), '# TDD');
      fs.writeFileSync(path.join(srcDir, 'tdd-workflow', 'extra.md'), '# Extra');

      const report = await mergeSkills(srcDir, destDir, null, nonInteractiveOpts);
      assert.strictEqual(report.added.length, 1);
      assert.ok(fs.existsSync(path.join(destDir, 'tdd-workflow', 'SKILL.md')));
      assert.ok(fs.existsSync(path.join(destDir, 'tdd-workflow', 'extra.md')));
    });

    await test('updates ECC-managed skills atomically', async () => {
      const srcDir = path.join(tmpDir, 'src-skills');
      const destDir = path.join(tmpDir, 'dest-skills-2');
      fs.mkdirSync(path.join(destDir, 'tdd-workflow'), { recursive: true });
      fs.writeFileSync(path.join(destDir, 'tdd-workflow', 'SKILL.md'), '# Old TDD');

      const manifest = createManifest('1.0.0', ['ts'], sampleArtifacts());
      const report = await mergeSkills(srcDir, destDir, manifest, nonInteractiveOpts);

      assert.ok(report.updated.includes('tdd-workflow'));
      const content = fs.readFileSync(path.join(destDir, 'tdd-workflow', 'SKILL.md'), 'utf8');
      assert.strictEqual(content, '# TDD');
    });

    // --- mergeRules ---
    describe('mergeRules');

    await test('adds rules by group', async () => {
      const srcDir = path.join(tmpDir, 'src-rules');
      const destDir = path.join(tmpDir, 'dest-rules-1');
      fs.mkdirSync(path.join(srcDir, 'common'), { recursive: true });
      fs.writeFileSync(path.join(srcDir, 'common', 'coding-style.md'), '# Style');
      fs.mkdirSync(path.join(srcDir, 'typescript'), { recursive: true });
      fs.writeFileSync(path.join(srcDir, 'typescript', 'ts-rules.md'), '# TS');

      const report = await mergeRules(srcDir, destDir, null, ['common', 'typescript'], nonInteractiveOpts);
      assert.strictEqual(report.added.length, 2);
      assert.ok(report.added.includes('common/coding-style.md'));
      assert.ok(report.added.includes('typescript/ts-rules.md'));
    });

    // --- mergeHooks ---
    describe('mergeHooks');

    await test('adds hooks to empty settings', () => {
      const hooksDir = path.join(tmpDir, 'hooks-src');
      const settingsDir = path.join(tmpDir, 'hooks-dest-1');
      fs.mkdirSync(hooksDir, { recursive: true });
      fs.mkdirSync(settingsDir, { recursive: true });
      fs.writeFileSync(
        path.join(hooksDir, 'hooks.json'),
        JSON.stringify({
          hooks: {
            PreToolUse: [{ matcher: 'Bash', hooks: [{ type: 'command', command: 'echo test' }], description: 'Test' }]
          }
        })
      );

      const result = mergeHooks(path.join(hooksDir, 'hooks.json'), path.join(settingsDir, 'settings.json'), '/plugin/root');
      assert.strictEqual(result.added, 1);
      assert.strictEqual(result.existing, 0);

      const settings = JSON.parse(fs.readFileSync(path.join(settingsDir, 'settings.json'), 'utf8'));
      assert.strictEqual(settings.hooks.PreToolUse.length, 1);
    });

    await test('deduplicates hooks', () => {
      const hooksDir = path.join(tmpDir, 'hooks-src');
      const settingsDir = path.join(tmpDir, 'hooks-dest-2');
      fs.mkdirSync(settingsDir, { recursive: true });

      // Pre-populate with the same hook
      fs.writeFileSync(
        path.join(settingsDir, 'settings.json'),
        JSON.stringify({
          hooks: {
            PreToolUse: [{ matcher: 'Bash', hooks: [{ type: 'command', command: 'echo test' }], description: 'Test' }]
          }
        })
      );

      const result = mergeHooks(path.join(hooksDir, 'hooks.json'), path.join(settingsDir, 'settings.json'), '/plugin/root');
      assert.strictEqual(result.added, 0);
      assert.strictEqual(result.existing, 1);

      const settings = JSON.parse(fs.readFileSync(path.join(settingsDir, 'settings.json'), 'utf8'));
      assert.strictEqual(settings.hooks.PreToolUse.length, 1);
    });

    await test('preserves non-hook keys in settings', () => {
      const hooksDir = path.join(tmpDir, 'hooks-src');
      const settingsDir = path.join(tmpDir, 'hooks-dest-3');
      fs.mkdirSync(settingsDir, { recursive: true });

      fs.writeFileSync(
        path.join(settingsDir, 'settings.json'),
        JSON.stringify({
          customKey: 'preserved',
          allowedTools: ['Read', 'Write'],
          hooks: {}
        })
      );

      mergeHooks(path.join(hooksDir, 'hooks.json'), path.join(settingsDir, 'settings.json'), '/plugin/root');

      const settings = JSON.parse(fs.readFileSync(path.join(settingsDir, 'settings.json'), 'utf8'));
      assert.strictEqual(settings.customKey, 'preserved');
      assert.deepStrictEqual(settings.allowedTools, ['Read', 'Write']);
    });

    await test('replaces ECC_ROOT placeholder', () => {
      const hooksDir = path.join(tmpDir, 'hooks-placeholder');
      const settingsDir = path.join(tmpDir, 'hooks-dest-4');
      fs.mkdirSync(hooksDir, { recursive: true });
      fs.mkdirSync(settingsDir, { recursive: true });

      fs.writeFileSync(
        path.join(hooksDir, 'hooks.json'),
        JSON.stringify({
          hooks: {
            Stop: [{ matcher: '*', hooks: [{ type: 'command', command: 'node "${ECC_ROOT}/dist/test.js"' }], description: 'Test' }]
          }
        })
      );

      mergeHooks(path.join(hooksDir, 'hooks.json'), path.join(settingsDir, 'settings.json'), '/my/ecc');

      const settings = JSON.parse(fs.readFileSync(path.join(settingsDir, 'settings.json'), 'utf8'));
      assert.ok(settings.hooks.Stop[0].hooks[0].command.includes('/my/ecc'));
      assert.ok(!settings.hooks.Stop[0].hooks[0].command.includes('ECC_ROOT'));
    });

    // --- combineMergeReports ---
    describe('combineMergeReports');

    await test('combines multiple reports', () => {
      const r1 = { added: ['a'], updated: ['b'], unchanged: [], skipped: [], smartMerged: [], errors: [] };
      const r2 = { added: ['c'], updated: [], unchanged: [], skipped: ['d'], smartMerged: ['e'], errors: [] };
      const combined = combineMergeReports(r1, r2);
      assert.deepStrictEqual(combined.added, ['a', 'c']);
      assert.deepStrictEqual(combined.updated, ['b']);
      assert.deepStrictEqual(combined.skipped, ['d']);
      assert.deepStrictEqual(combined.smartMerged, ['e']);
    });

    // --- isLegacyEccHook ---
    describe('isLegacyEccHook');

    await test('detects scripts/hooks/ legacy path', () => {
      assert.strictEqual(
        isLegacyEccHook({
          matcher: '*',
          hooks: [{ type: 'command', command: 'node "/opt/homebrew/lib/node_modules/@lebocqtitouan/ecc/scripts/hooks/check-console-log.js"' }]
        }),
        true
      );
    });

    await test('detects inline node -e legacy hooks', () => {
      assert.strictEqual(
        isLegacyEccHook({
          matcher: 'Bash',
          hooks: [{ type: 'command', command: 'node -e "const cmd = ...; if (/dev-server/.test(cmd)) ..."' }]
        }),
        true
      );
    });

    await test('does not flag current run-with-flags hooks', () => {
      assert.strictEqual(
        isLegacyEccHook({
          matcher: 'Bash',
          hooks: [{ type: 'command', command: 'node "/plugin/dist/hooks/run-with-flags.js" "pre:bash:dev-server-block" "dist/hooks/pre-bash-dev-server-block.js" "standard,strict"' }]
        }),
        false
      );
    });

    await test('does not flag user-custom hooks', () => {
      assert.strictEqual(
        isLegacyEccHook({
          matcher: 'Bash',
          hooks: [{ type: 'command', command: 'node my-custom-hook.js' }]
        }),
        false
      );
    });

    await test('does not flag run-with-flags-shell.sh hooks', () => {
      assert.strictEqual(
        isLegacyEccHook({
          matcher: '*',
          hooks: [{ type: 'command', command: 'bash "/plugin/scripts/hooks/run-with-flags-shell.sh" "pre:observe" "skills/continuous-learning-v2/hooks/observe.sh"' }]
        }),
        false
      );
    });

    // --- mergeHooks legacy cleanup ---
    describe('mergeHooks legacy cleanup');

    await test('removes legacy hooks during merge', () => {
      const hooksDir = path.join(tmpDir, 'hooks-src');
      const settingsDir = path.join(tmpDir, 'hooks-dest-legacy');
      fs.mkdirSync(settingsDir, { recursive: true });

      // Pre-populate with a legacy hook and a user-custom hook
      fs.writeFileSync(
        path.join(settingsDir, 'settings.json'),
        JSON.stringify({
          hooks: {
            Stop: [
              { matcher: '*', hooks: [{ type: 'command', command: 'node "/ecc/scripts/hooks/check-console-log.js"' }], description: 'Legacy' },
              { matcher: '*', hooks: [{ type: 'command', command: 'node my-custom-stop-hook.js' }], description: 'User custom' }
            ]
          }
        })
      );

      const result = mergeHooks(path.join(hooksDir, 'hooks.json'), path.join(settingsDir, 'settings.json'), '/plugin/root');

      assert.strictEqual(result.legacyRemoved, 1);

      const settings = JSON.parse(fs.readFileSync(path.join(settingsDir, 'settings.json'), 'utf8'));
      // Legacy hook removed, user hook preserved, new hook added
      const stopHooks = settings.hooks.Stop;
      assert.ok(
        stopHooks.some(h => h.description === 'User custom'),
        'User hook preserved'
      );
      assert.ok(!stopHooks.some(h => h.description === 'Legacy'), 'Legacy hook removed');
    });

    await test('removes all 10 legacy hooks from realistic settings.json', () => {
      const hooksDir = path.join(tmpDir, 'hooks-src');
      const settingsDir = path.join(tmpDir, 'hooks-dest-realistic');
      fs.mkdirSync(settingsDir, { recursive: true });

      // Realistic legacy settings.json matching the actual user's ~/.claude/settings.json
      const legacySettings = {
        hooks: {
          PreToolUse: [
            { matcher: 'Write', hooks: [{ type: 'command', command: 'node "/opt/homebrew/lib/node_modules/@lebocqtitouan/ecc/scripts/hooks/doc-file-warning.js"' }] },
            { matcher: 'Edit|Write', hooks: [{ type: 'command', command: 'node "/opt/homebrew/lib/node_modules/@lebocqtitouan/ecc/scripts/hooks/suggest-compact.js"' }] }
          ],
          PreCompact: [{ matcher: '*', hooks: [{ type: 'command', command: 'node "/opt/homebrew/lib/node_modules/@lebocqtitouan/ecc/scripts/hooks/pre-compact.js"' }] }],
          SessionStart: [{ matcher: '*', hooks: [{ type: 'command', command: 'node "/opt/homebrew/lib/node_modules/@lebocqtitouan/ecc/scripts/hooks/session-start.js"' }] }],
          PostToolUse: [
            { matcher: 'Edit', hooks: [{ type: 'command', command: 'node "/opt/homebrew/lib/node_modules/@lebocqtitouan/ecc/scripts/hooks/post-edit-format.js"' }] },
            { matcher: 'Edit', hooks: [{ type: 'command', command: 'node "/opt/homebrew/lib/node_modules/@lebocqtitouan/ecc/scripts/hooks/post-edit-typecheck.js"' }] },
            { matcher: 'Edit', hooks: [{ type: 'command', command: 'node "/opt/homebrew/lib/node_modules/@lebocqtitouan/ecc/scripts/hooks/post-edit-console-warn.js"' }] },
            // User-custom hook that must be preserved
            { matcher: 'Bash', hooks: [{ type: 'command', command: 'python3 my-custom-hook.py' }], description: 'My custom hook' }
          ],
          Stop: [
            { matcher: '*', hooks: [{ type: 'command', command: 'node "/opt/homebrew/lib/node_modules/@lebocqtitouan/ecc/scripts/hooks/check-console-log.js"' }] },
            { matcher: '*', hooks: [{ type: 'command', command: 'node "/opt/homebrew/lib/node_modules/@lebocqtitouan/ecc/scripts/hooks/session-end.js"' }] },
            { matcher: '*', hooks: [{ type: 'command', command: 'node "/opt/homebrew/lib/node_modules/@lebocqtitouan/ecc/scripts/hooks/evaluate-session.js"' }] }
          ]
        },
        // Non-hook keys must be preserved
        allowedTools: ['Read', 'Write'],
        customSetting: true
      };

      fs.writeFileSync(path.join(settingsDir, 'settings.json'), JSON.stringify(legacySettings));

      const result = mergeHooks(path.join(hooksDir, 'hooks.json'), path.join(settingsDir, 'settings.json'), '/plugin/root');

      // All 10 legacy entries removed
      assert.strictEqual(result.legacyRemoved, 10);
      // New hook from hooks-src added
      assert.strictEqual(result.added, 1);

      const settings = JSON.parse(fs.readFileSync(path.join(settingsDir, 'settings.json'), 'utf8'));

      // User-custom hook preserved
      const postHooks = settings.hooks.PostToolUse || [];
      assert.ok(
        postHooks.some(h => h.description === 'My custom hook'),
        'User custom hook preserved'
      );

      // No legacy scripts/hooks/ references remain
      const allCommands = JSON.stringify(settings.hooks);
      assert.ok(!allCommands.includes('scripts/hooks/'), 'No legacy paths remain');

      // Non-hook settings preserved
      assert.deepStrictEqual(settings.allowedTools, ['Read', 'Write']);
      assert.strictEqual(settings.customSetting, true);
    });

    // --- defaultMergeOptions ---
    describe('defaultMergeOptions');

    await test('returns correct defaults', () => {
      const opts = defaultMergeOptions();
      assert.strictEqual(opts.dryRun, false);
      assert.strictEqual(opts.force, false);
      assert.strictEqual(opts.interactive, true);
      assert.strictEqual(opts.applyAll, null);
    });
  } finally {
    cleanup(tmpDir);
  }
}

module.exports = { runTests };

if (require.main === module) {
  const { getResults, resetCounters } = require('../harness');
  resetCounters();
  runTests().then(() => {
    const r = getResults();
    console.log('\nPassed: ' + r.passed);
    console.log('Failed: ' + r.failed);
    console.log('Total:  ' + (r.passed + r.failed));
    if (r.failed > 0) process.exit(1);
  });
}
