/**
 * Tests for src/lib/merge.ts
 *
 * Run with: npx tsx tests/lib/merge.test.js
 */

const assert = require('assert');
const fs = require('fs');
const path = require('path');
const os = require('os');

const {
  mergeDirectory,
  mergeSkills,
  mergeRules,
  mergeHooks,
  isLegacyEccHook,
  combineMergeReports,
  defaultMergeOptions,
} = require('../../src/lib/merge');

const { createManifest } = require('../../src/lib/manifest');

function test(name, fn) {
  try {
    const result = fn();
    if (result && typeof result.then === 'function') {
      return result
        .then(() => { console.log(`  ✓ ${name}`); return true; })
        .catch(err => { console.log(`  ✗ ${name}\n    Error: ${err.message}`); return false; });
    }
    console.log(`  ✓ ${name}`);
    return true;
  } catch (err) {
    console.log(`  ✗ ${name}`);
    console.log(`    Error: ${err.message}`);
    return false;
  }
}

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
    hookDescriptions: [],
  };
}

async function runTests() {
  console.log('\n=== Testing merge.ts ===\n');
  let passed = 0;
  let failed = 0;

  const tmpDir = makeTempDir();

  try {
    const nonInteractiveOpts = { ...defaultMergeOptions(), interactive: false };

    // --- mergeDirectory ---
    console.log('mergeDirectory:');

    if (await test('adds new files to empty destination', async () => {
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
    })) passed++; else failed++;

    if (await test('updates ECC-managed files', async () => {
      const srcDir = path.join(tmpDir, 'src-agents');
      const destDir = path.join(tmpDir, 'dest-agents-2');
      fs.mkdirSync(destDir, { recursive: true });
      fs.writeFileSync(path.join(destDir, 'planner.md'), '# Old Planner');

      const manifest = createManifest('1.0.0', ['ts'], sampleArtifacts());
      const report = await mergeDirectory(srcDir, destDir, manifest, 'agents', nonInteractiveOpts);

      assert.ok(report.updated.includes('planner.md'));
      const content = fs.readFileSync(path.join(destDir, 'planner.md'), 'utf8');
      assert.strictEqual(content, '# Planner');
    })) passed++; else failed++;

    if (await test('skips user-custom files in non-interactive mode', async () => {
      const srcDir = path.join(tmpDir, 'src-agents');
      const destDir = path.join(tmpDir, 'dest-agents-3');
      fs.mkdirSync(destDir, { recursive: true });
      fs.writeFileSync(path.join(destDir, 'planner.md'), '# Custom Planner');

      // No manifest = all files are "unmanaged"
      const report = await mergeDirectory(srcDir, destDir, null, 'agents', nonInteractiveOpts);

      assert.ok(report.skipped.includes('planner.md'));
      const content = fs.readFileSync(path.join(destDir, 'planner.md'), 'utf8');
      assert.strictEqual(content, '# Custom Planner');
    })) passed++; else failed++;

    if (await test('force mode overwrites everything', async () => {
      const srcDir = path.join(tmpDir, 'src-agents');
      const destDir = path.join(tmpDir, 'dest-agents-4');
      fs.mkdirSync(destDir, { recursive: true });
      fs.writeFileSync(path.join(destDir, 'planner.md'), '# Custom Planner');

      const forceOpts = { ...nonInteractiveOpts, force: true };
      const report = await mergeDirectory(srcDir, destDir, null, 'agents', forceOpts);

      assert.ok(report.updated.includes('planner.md'));
      const content = fs.readFileSync(path.join(destDir, 'planner.md'), 'utf8');
      assert.strictEqual(content, '# Planner');
    })) passed++; else failed++;

    if (await test('dry-run does not write files', async () => {
      const srcDir = path.join(tmpDir, 'src-agents');
      const destDir = path.join(tmpDir, 'dest-agents-5-dryrun');

      const dryRunOpts = { ...nonInteractiveOpts, dryRun: true };
      const report = await mergeDirectory(srcDir, destDir, null, 'agents', dryRunOpts);

      assert.strictEqual(report.added.length, 2);
      assert.ok(!fs.existsSync(destDir));
    })) passed++; else failed++;

    if (await test('handles non-existent source dir', async () => {
      const report = await mergeDirectory(
        path.join(tmpDir, 'nonexistent-src'),
        path.join(tmpDir, 'dest-x'),
        null,
        'agents',
        nonInteractiveOpts,
      );
      assert.strictEqual(report.added.length, 0);
    })) passed++; else failed++;

    // --- mergeSkills ---
    console.log('\nmergeSkills:');

    if (await test('adds new skills', async () => {
      const srcDir = path.join(tmpDir, 'src-skills');
      const destDir = path.join(tmpDir, 'dest-skills-1');
      fs.mkdirSync(path.join(srcDir, 'tdd-workflow'), { recursive: true });
      fs.writeFileSync(path.join(srcDir, 'tdd-workflow', 'SKILL.md'), '# TDD');
      fs.writeFileSync(path.join(srcDir, 'tdd-workflow', 'extra.md'), '# Extra');

      const report = await mergeSkills(srcDir, destDir, null, nonInteractiveOpts);
      assert.strictEqual(report.added.length, 1);
      assert.ok(fs.existsSync(path.join(destDir, 'tdd-workflow', 'SKILL.md')));
      assert.ok(fs.existsSync(path.join(destDir, 'tdd-workflow', 'extra.md')));
    })) passed++; else failed++;

    if (await test('updates ECC-managed skills atomically', async () => {
      const srcDir = path.join(tmpDir, 'src-skills');
      const destDir = path.join(tmpDir, 'dest-skills-2');
      fs.mkdirSync(path.join(destDir, 'tdd-workflow'), { recursive: true });
      fs.writeFileSync(path.join(destDir, 'tdd-workflow', 'SKILL.md'), '# Old TDD');

      const manifest = createManifest('1.0.0', ['ts'], sampleArtifacts());
      const report = await mergeSkills(srcDir, destDir, manifest, nonInteractiveOpts);

      assert.ok(report.updated.includes('tdd-workflow'));
      const content = fs.readFileSync(path.join(destDir, 'tdd-workflow', 'SKILL.md'), 'utf8');
      assert.strictEqual(content, '# TDD');
    })) passed++; else failed++;

    // --- mergeRules ---
    console.log('\nmergeRules:');

    if (await test('adds rules by group', async () => {
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
    })) passed++; else failed++;

    // --- mergeHooks ---
    console.log('\nmergeHooks:');

    if (await test('adds hooks to empty settings', () => {
      const hooksDir = path.join(tmpDir, 'hooks-src');
      const settingsDir = path.join(tmpDir, 'hooks-dest-1');
      fs.mkdirSync(hooksDir, { recursive: true });
      fs.mkdirSync(settingsDir, { recursive: true });
      fs.writeFileSync(path.join(hooksDir, 'hooks.json'), JSON.stringify({
        hooks: {
          PreToolUse: [
            { matcher: 'Bash', hooks: [{ type: 'command', command: 'echo test' }], description: 'Test' },
          ],
        },
      }));

      const result = mergeHooks(
        path.join(hooksDir, 'hooks.json'),
        path.join(settingsDir, 'settings.json'),
        '/plugin/root',
      );
      assert.strictEqual(result.added, 1);
      assert.strictEqual(result.existing, 0);

      const settings = JSON.parse(fs.readFileSync(path.join(settingsDir, 'settings.json'), 'utf8'));
      assert.strictEqual(settings.hooks.PreToolUse.length, 1);
    })) passed++; else failed++;

    if (await test('deduplicates hooks', () => {
      const hooksDir = path.join(tmpDir, 'hooks-src');
      const settingsDir = path.join(tmpDir, 'hooks-dest-2');
      fs.mkdirSync(settingsDir, { recursive: true });

      // Pre-populate with the same hook
      fs.writeFileSync(path.join(settingsDir, 'settings.json'), JSON.stringify({
        hooks: {
          PreToolUse: [
            { matcher: 'Bash', hooks: [{ type: 'command', command: 'echo test' }], description: 'Test' },
          ],
        },
      }));

      const result = mergeHooks(
        path.join(hooksDir, 'hooks.json'),
        path.join(settingsDir, 'settings.json'),
        '/plugin/root',
      );
      assert.strictEqual(result.added, 0);
      assert.strictEqual(result.existing, 1);

      const settings = JSON.parse(fs.readFileSync(path.join(settingsDir, 'settings.json'), 'utf8'));
      assert.strictEqual(settings.hooks.PreToolUse.length, 1);
    })) passed++; else failed++;

    if (await test('preserves non-hook keys in settings', () => {
      const hooksDir = path.join(tmpDir, 'hooks-src');
      const settingsDir = path.join(tmpDir, 'hooks-dest-3');
      fs.mkdirSync(settingsDir, { recursive: true });

      fs.writeFileSync(path.join(settingsDir, 'settings.json'), JSON.stringify({
        customKey: 'preserved',
        allowedTools: ['Read', 'Write'],
        hooks: {},
      }));

      mergeHooks(
        path.join(hooksDir, 'hooks.json'),
        path.join(settingsDir, 'settings.json'),
        '/plugin/root',
      );

      const settings = JSON.parse(fs.readFileSync(path.join(settingsDir, 'settings.json'), 'utf8'));
      assert.strictEqual(settings.customKey, 'preserved');
      assert.deepStrictEqual(settings.allowedTools, ['Read', 'Write']);
    })) passed++; else failed++;

    if (await test('replaces CLAUDE_PLUGIN_ROOT placeholder', () => {
      const hooksDir = path.join(tmpDir, 'hooks-placeholder');
      const settingsDir = path.join(tmpDir, 'hooks-dest-4');
      fs.mkdirSync(hooksDir, { recursive: true });
      fs.mkdirSync(settingsDir, { recursive: true });

      fs.writeFileSync(path.join(hooksDir, 'hooks.json'), JSON.stringify({
        hooks: {
          Stop: [
            { matcher: '*', hooks: [{ type: 'command', command: 'node "${CLAUDE_PLUGIN_ROOT}/dist/test.js"' }], description: 'Test' },
          ],
        },
      }));

      mergeHooks(
        path.join(hooksDir, 'hooks.json'),
        path.join(settingsDir, 'settings.json'),
        '/my/plugin',
      );

      const settings = JSON.parse(fs.readFileSync(path.join(settingsDir, 'settings.json'), 'utf8'));
      assert.ok(settings.hooks.Stop[0].hooks[0].command.includes('/my/plugin'));
      assert.ok(!settings.hooks.Stop[0].hooks[0].command.includes('CLAUDE_PLUGIN_ROOT'));
    })) passed++; else failed++;

    // --- combineMergeReports ---
    console.log('\ncombineMergeReports:');

    if (test('combines multiple reports', () => {
      const r1 = { added: ['a'], updated: ['b'], skipped: [], smartMerged: [], errors: [] };
      const r2 = { added: ['c'], updated: [], skipped: ['d'], smartMerged: ['e'], errors: [] };
      const combined = combineMergeReports(r1, r2);
      assert.deepStrictEqual(combined.added, ['a', 'c']);
      assert.deepStrictEqual(combined.updated, ['b']);
      assert.deepStrictEqual(combined.skipped, ['d']);
      assert.deepStrictEqual(combined.smartMerged, ['e']);
    })) passed++; else failed++;

    // --- isLegacyEccHook ---
    console.log('\nisLegacyEccHook:');

    if (test('detects scripts/hooks/ legacy path', () => {
      assert.strictEqual(isLegacyEccHook({
        matcher: '*',
        hooks: [{ type: 'command', command: 'node "/opt/homebrew/lib/node_modules/@lebocqtitouan/ecc/scripts/hooks/check-console-log.js"' }],
      }), true);
    })) passed++; else failed++;

    if (test('detects inline node -e legacy hooks', () => {
      assert.strictEqual(isLegacyEccHook({
        matcher: 'Bash',
        hooks: [{ type: 'command', command: 'node -e "const cmd = ...; if (/dev-server/.test(cmd)) ..."' }],
      }), true);
    })) passed++; else failed++;

    if (test('does not flag current run-with-flags hooks', () => {
      assert.strictEqual(isLegacyEccHook({
        matcher: 'Bash',
        hooks: [{ type: 'command', command: 'node "/plugin/dist/hooks/run-with-flags.js" "pre:bash:dev-server-block" "dist/hooks/pre-bash-dev-server-block.js" "standard,strict"' }],
      }), false);
    })) passed++; else failed++;

    if (test('does not flag user-custom hooks', () => {
      assert.strictEqual(isLegacyEccHook({
        matcher: 'Bash',
        hooks: [{ type: 'command', command: 'node my-custom-hook.js' }],
      }), false);
    })) passed++; else failed++;

    if (test('does not flag run-with-flags-shell.sh hooks', () => {
      assert.strictEqual(isLegacyEccHook({
        matcher: '*',
        hooks: [{ type: 'command', command: 'bash "/plugin/scripts/hooks/run-with-flags-shell.sh" "pre:observe" "skills/continuous-learning-v2/hooks/observe.sh"' }],
      }), false);
    })) passed++; else failed++;

    // --- mergeHooks legacy cleanup ---
    console.log('\nmergeHooks legacy cleanup:');

    if (await test('removes legacy hooks during merge', () => {
      const hooksDir = path.join(tmpDir, 'hooks-src');
      const settingsDir = path.join(tmpDir, 'hooks-dest-legacy');
      fs.mkdirSync(settingsDir, { recursive: true });

      // Pre-populate with a legacy hook and a user-custom hook
      fs.writeFileSync(path.join(settingsDir, 'settings.json'), JSON.stringify({
        hooks: {
          Stop: [
            { matcher: '*', hooks: [{ type: 'command', command: 'node "/ecc/scripts/hooks/check-console-log.js"' }], description: 'Legacy' },
            { matcher: '*', hooks: [{ type: 'command', command: 'node my-custom-stop-hook.js' }], description: 'User custom' },
          ],
        },
      }));

      const result = mergeHooks(
        path.join(hooksDir, 'hooks.json'),
        path.join(settingsDir, 'settings.json'),
        '/plugin/root',
      );

      assert.strictEqual(result.legacyRemoved, 1);

      const settings = JSON.parse(fs.readFileSync(path.join(settingsDir, 'settings.json'), 'utf8'));
      // Legacy hook removed, user hook preserved, new hook added
      const stopHooks = settings.hooks.Stop;
      assert.ok(stopHooks.some(h => h.description === 'User custom'), 'User hook preserved');
      assert.ok(!stopHooks.some(h => h.description === 'Legacy'), 'Legacy hook removed');
    })) passed++; else failed++;

    // --- defaultMergeOptions ---
    console.log('\ndefaultMergeOptions:');

    if (test('returns correct defaults', () => {
      const opts = defaultMergeOptions();
      assert.strictEqual(opts.dryRun, false);
      assert.strictEqual(opts.force, false);
      assert.strictEqual(opts.interactive, true);
      assert.strictEqual(opts.applyAll, null);
    })) passed++; else failed++;

  } finally {
    cleanup(tmpDir);
  }

  console.log(`\n${passed} passed, ${failed} failed\n`);
  if (failed > 0) process.exit(1);
}

runTests();
