/**
 * Tests for src/lib/config-audit.ts
 *
 * Run with: npx tsx tests/lib/config-audit.test.js
 */

const assert = require('assert');
const fs = require('fs');
const path = require('path');
const os = require('os');

const { isEccManagedHook, diffHooks, auditEccConfig } = require('../../src/lib/config-audit');
const { test, describe } = require('../harness');

function makeTempDir() {
  return fs.mkdtempSync(path.join(os.tmpdir(), 'ecc-audit-test-'));
}

function cleanup(dir) {
  fs.rmSync(dir, { recursive: true, force: true });
}

/** Helper: build a hook entry with a single command. */
function hookEntry(matcher, command, description) {
  const entry = { matcher, hooks: [{ type: 'command', command }] };
  if (description) entry.description = description;
  return entry;
}

async function runTests() {
  describe('Testing config-audit.ts');

  const tmpDir = makeTempDir();

  try {
    // -----------------------------------------------------------------------
    // isEccManagedHook
    // -----------------------------------------------------------------------
    describe('isEccManagedHook');

    await test('detects ecc-hook commands as ECC-managed', () => {
      assert.strictEqual(isEccManagedHook(hookEntry('Bash', 'ecc-hook "pre:bash:dev-server-block" "dist/hooks/pre-bash-dev-server-block.js" "standard,strict"'), {}), true);
    });

    await test('detects ecc-shell-hook commands as ECC-managed', () => {
      assert.strictEqual(isEccManagedHook(hookEntry('*', 'ecc-shell-hook "pre:observe" "skills/continuous-learning-v2/hooks/observe.sh" "standard,strict"'), {}), true);
    });

    await test('detects absolute path with @lebocqtitouan/ecc/ as ECC-managed', () => {
      assert.strictEqual(isEccManagedHook(hookEntry('*', 'node "/opt/homebrew/lib/node_modules/@lebocqtitouan/ecc/10-scripts/hooks/check-console-log.js"'), {}), true);
    });

    await test('detects 05-skills/ absolute path as ECC-managed', () => {
      assert.strictEqual(isEccManagedHook(hookEntry('*', 'bash "/opt/homebrew/lib/node_modules/@lebocqtitouan/ecc/05-skills/hooks/observe.sh"'), {}), true);
    });

    await test('detects scripts/hooks/ legacy path as ECC-managed', () => {
      assert.strictEqual(isEccManagedHook(hookEntry('*', 'node "/opt/homebrew/lib/node_modules/@lebocqtitouan/ecc/scripts/hooks/check-console-log.js"'), {}), true);
    });

    await test('detects dist/hooks/run-with-flags.js absolute path as ECC-managed', () => {
      assert.strictEqual(isEccManagedHook(hookEntry('Bash', 'node "/plugin/dist/hooks/run-with-flags.js" "pre:bash:dev-server-block" "dist/hooks/pre-bash-dev-server-block.js"'), {}), true);
    });

    await test('detects ${ECC_ROOT} placeholder as ECC-managed', () => {
      assert.strictEqual(isEccManagedHook(hookEntry('Bash', 'node "${ECC_ROOT}/dist/hooks/run-with-flags.js" "pre:bash:test"'), {}), true);
    });

    await test('detects inline node -e legacy hook as ECC-managed', () => {
      assert.strictEqual(isEccManagedHook(hookEntry('Bash', 'node -e "const cmd = ...; if (/dev-server/.test(cmd)) ..."'), {}), true);
    });

    await test('detects hook matching source hooks.json entry as ECC-managed', () => {
      const sourceHooks = {
        Stop: [hookEntry('*', 'ecc-hook "stop:check-console-log" "dist/hooks/check-console-log.js" "standard,strict"')]
      };
      // Even if the command doesn't start with ecc-hook (hypothetical), matching source = ECC-managed
      assert.strictEqual(isEccManagedHook(hookEntry('*', 'ecc-hook "stop:check-console-log" "dist/hooks/check-console-log.js" "standard,strict"'), sourceHooks), true);
    });

    await test('does NOT flag user-custom hooks', () => {
      assert.strictEqual(isEccManagedHook(hookEntry('Bash', 'node my-custom-hook.js'), {}), false);
    });

    await test('does NOT flag user-custom python hooks', () => {
      assert.strictEqual(isEccManagedHook(hookEntry('Bash', 'python3 my-custom-hook.py', 'My custom hook'), {}), false);
    });

    await test('does NOT flag user-custom hooks with absolute paths outside ECC', () => {
      assert.strictEqual(isEccManagedHook(hookEntry('*', 'node "/usr/local/lib/other-package/hook.js"'), {}), false);
    });

    await test('handles entry with no hooks array gracefully', () => {
      assert.strictEqual(isEccManagedHook({ matcher: '*' }, {}), false);
    });

    await test('handles entry with empty hooks array', () => {
      assert.strictEqual(isEccManagedHook({ matcher: '*', hooks: [] }, {}), false);
    });

    // -----------------------------------------------------------------------
    // diffHooks
    // -----------------------------------------------------------------------
    describe('diffHooks');

    await test('returns empty diff when configs match exactly', () => {
      const settingsDir = path.join(tmpDir, 'diff-match');
      const hooksDir = path.join(tmpDir, 'diff-match-src');
      fs.mkdirSync(settingsDir, { recursive: true });
      fs.mkdirSync(hooksDir, { recursive: true });

      const hooksJson = {
        hooks: {
          Stop: [hookEntry('*', 'ecc-hook "stop:test" "dist/hooks/test.js" "standard,strict"', 'Test hook')]
        }
      };

      fs.writeFileSync(path.join(hooksDir, 'hooks.json'), JSON.stringify(hooksJson));
      fs.writeFileSync(path.join(settingsDir, 'settings.json'), JSON.stringify({ hooks: hooksJson.hooks }));

      const diff = diffHooks(path.join(settingsDir, 'settings.json'), path.join(hooksDir, 'hooks.json'));

      assert.strictEqual(diff.stale.length, 0);
      assert.strictEqual(diff.missing.length, 0);
      assert.strictEqual(diff.matching.length, 1);
    });

    await test('detects stale ECC hooks not in source', () => {
      const settingsDir = path.join(tmpDir, 'diff-stale');
      const hooksDir = path.join(tmpDir, 'diff-stale-src');
      fs.mkdirSync(settingsDir, { recursive: true });
      fs.mkdirSync(hooksDir, { recursive: true });

      // Source has no hooks
      fs.writeFileSync(path.join(hooksDir, 'hooks.json'), JSON.stringify({ hooks: {} }));
      // Settings has a stale ECC hook
      fs.writeFileSync(
        path.join(settingsDir, 'settings.json'),
        JSON.stringify({
          hooks: {
            Stop: [hookEntry('*', 'ecc-hook "stop:old-hook" "dist/hooks/old.js" "standard,strict"', 'Old hook')]
          }
        })
      );

      const diff = diffHooks(path.join(settingsDir, 'settings.json'), path.join(hooksDir, 'hooks.json'));

      assert.strictEqual(diff.stale.length, 1);
      assert.strictEqual(diff.stale[0].event, 'Stop');
      assert.strictEqual(diff.missing.length, 0);
    });

    await test('detects missing hooks not in settings', () => {
      const settingsDir = path.join(tmpDir, 'diff-missing');
      const hooksDir = path.join(tmpDir, 'diff-missing-src');
      fs.mkdirSync(settingsDir, { recursive: true });
      fs.mkdirSync(hooksDir, { recursive: true });

      // Source has a hook
      fs.writeFileSync(
        path.join(hooksDir, 'hooks.json'),
        JSON.stringify({
          hooks: {
            Stop: [hookEntry('*', 'ecc-hook "stop:new" "dist/hooks/new.js" "standard,strict"', 'New hook')]
          }
        })
      );
      // Settings is empty
      fs.writeFileSync(path.join(settingsDir, 'settings.json'), JSON.stringify({ hooks: {} }));

      const diff = diffHooks(path.join(settingsDir, 'settings.json'), path.join(hooksDir, 'hooks.json'));

      assert.strictEqual(diff.missing.length, 1);
      assert.strictEqual(diff.missing[0].event, 'Stop');
      assert.strictEqual(diff.stale.length, 0);
    });

    await test('detects stale hooks with numbered-prefix paths', () => {
      const settingsDir = path.join(tmpDir, 'diff-numbered');
      const hooksDir = path.join(tmpDir, 'diff-numbered-src');
      fs.mkdirSync(settingsDir, { recursive: true });
      fs.mkdirSync(hooksDir, { recursive: true });

      // Source has current hooks
      fs.writeFileSync(
        path.join(hooksDir, 'hooks.json'),
        JSON.stringify({
          hooks: {
            Stop: [hookEntry('*', 'ecc-hook "stop:check-console-log" "dist/hooks/check-console-log.js" "standard,strict"', 'Check console')]
          }
        })
      );
      // Settings has old numbered-prefix hook
      fs.writeFileSync(
        path.join(settingsDir, 'settings.json'),
        JSON.stringify({
          hooks: {
            Stop: [hookEntry('*', 'node "/opt/homebrew/lib/node_modules/@lebocqtitouan/ecc/10-scripts/hooks/check-console-log.js"', 'Check console')]
          }
        })
      );

      const diff = diffHooks(path.join(settingsDir, 'settings.json'), path.join(hooksDir, 'hooks.json'));

      assert.strictEqual(diff.stale.length, 1, 'Old numbered-prefix hook detected as stale');
      assert.strictEqual(diff.missing.length, 1, 'Current hook detected as missing');
    });

    await test('preserves user-custom hooks (not in diff)', () => {
      const settingsDir = path.join(tmpDir, 'diff-custom');
      const hooksDir = path.join(tmpDir, 'diff-custom-src');
      fs.mkdirSync(settingsDir, { recursive: true });
      fs.mkdirSync(hooksDir, { recursive: true });

      fs.writeFileSync(path.join(hooksDir, 'hooks.json'), JSON.stringify({ hooks: {} }));
      fs.writeFileSync(
        path.join(settingsDir, 'settings.json'),
        JSON.stringify({
          hooks: {
            Stop: [hookEntry('*', 'python3 my-custom-hook.py', 'Custom')]
          }
        })
      );

      const diff = diffHooks(path.join(settingsDir, 'settings.json'), path.join(hooksDir, 'hooks.json'));

      assert.strictEqual(diff.stale.length, 0, 'User hook not marked as stale');
      assert.strictEqual(diff.missing.length, 0);
      assert.strictEqual(diff.userHooks.length, 1, 'User hook tracked separately');
    });

    await test('handles missing settings file', () => {
      const hooksDir = path.join(tmpDir, 'diff-no-settings-src');
      fs.mkdirSync(hooksDir, { recursive: true });

      fs.writeFileSync(
        path.join(hooksDir, 'hooks.json'),
        JSON.stringify({
          hooks: {
            Stop: [hookEntry('*', 'ecc-hook "stop:test" "dist/hooks/test.js"', 'Test')]
          }
        })
      );

      const diff = diffHooks(path.join(tmpDir, 'nonexistent-settings.json'), path.join(hooksDir, 'hooks.json'));

      assert.strictEqual(diff.missing.length, 1);
      assert.strictEqual(diff.stale.length, 0);
    });

    await test('full realistic scenario: mixed stale, missing, matching, and user hooks', () => {
      const settingsDir = path.join(tmpDir, 'diff-realistic');
      const hooksDir = path.join(tmpDir, 'diff-realistic-src');
      fs.mkdirSync(settingsDir, { recursive: true });
      fs.mkdirSync(hooksDir, { recursive: true });

      // Source: 2 hooks
      fs.writeFileSync(
        path.join(hooksDir, 'hooks.json'),
        JSON.stringify({
          hooks: {
            PreToolUse: [hookEntry('Bash', 'ecc-hook "pre:bash:dev-server-block" "dist/hooks/pre-bash-dev-server-block.js" "standard,strict"', 'Block dev servers')],
            Stop: [hookEntry('*', 'ecc-hook "stop:check-console-log" "dist/hooks/check-console-log.js" "standard,strict"', 'Check console')]
          }
        })
      );

      // Settings: 1 matching, 1 stale (numbered prefix), 1 user custom, missing the Stop hook
      fs.writeFileSync(
        path.join(settingsDir, 'settings.json'),
        JSON.stringify({
          hooks: {
            PreToolUse: [
              hookEntry('Bash', 'ecc-hook "pre:bash:dev-server-block" "dist/hooks/pre-bash-dev-server-block.js" "standard,strict"', 'Block dev servers'),
              hookEntry('Bash', 'node "/opt/homebrew/lib/node_modules/@lebocqtitouan/ecc/10-scripts/hooks/tmux-reminder.js"', 'Tmux reminder')
            ],
            PostToolUse: [hookEntry('Bash', 'python3 my-linter.py', 'My linter')]
          },
          allowedTools: ['Read']
        })
      );

      const diff = diffHooks(path.join(settingsDir, 'settings.json'), path.join(hooksDir, 'hooks.json'));

      assert.strictEqual(diff.matching.length, 1, '1 hook matches');
      assert.strictEqual(diff.stale.length, 1, '1 stale hook (numbered prefix)');
      assert.strictEqual(diff.missing.length, 1, '1 missing hook (Stop)');
      assert.strictEqual(diff.userHooks.length, 1, '1 user hook preserved');
    });

    // -----------------------------------------------------------------------
    // auditEccConfig
    // -----------------------------------------------------------------------
    describe('auditEccConfig');

    await test('returns full audit comparing source and installed artifacts', () => {
      const eccRoot = path.join(tmpDir, 'audit-ecc-root');
      const claudeDir = path.join(tmpDir, 'audit-claude-dir');

      // Set up source artifacts
      fs.mkdirSync(path.join(eccRoot, 'agents'), { recursive: true });
      fs.writeFileSync(path.join(eccRoot, 'agents', 'planner.md'), '# Planner v2');
      fs.writeFileSync(path.join(eccRoot, 'agents', 'architect.md'), '# Architect');

      fs.mkdirSync(path.join(eccRoot, 'commands'), { recursive: true });
      fs.writeFileSync(path.join(eccRoot, 'commands', 'tdd.md'), '# TDD');

      fs.mkdirSync(path.join(eccRoot, 'hooks'), { recursive: true });
      fs.writeFileSync(
        path.join(eccRoot, 'hooks', 'hooks.json'),
        JSON.stringify({
          hooks: {
            Stop: [hookEntry('*', 'ecc-hook "stop:test" "dist/hooks/test.js"', 'Test')]
          }
        })
      );

      // Set up installed artifacts (outdated planner, missing architect)
      fs.mkdirSync(path.join(claudeDir, 'agents'), { recursive: true });
      fs.writeFileSync(path.join(claudeDir, 'agents', 'planner.md'), '# Planner v1');

      fs.mkdirSync(path.join(claudeDir, 'commands'), { recursive: true });
      fs.writeFileSync(path.join(claudeDir, 'commands', 'tdd.md'), '# TDD');

      fs.writeFileSync(path.join(claudeDir, 'settings.json'), JSON.stringify({ hooks: {} }));

      const audit = auditEccConfig(eccRoot, claudeDir);

      assert.ok(audit.agents.outdated.includes('planner.md'), 'planner.md is outdated');
      assert.ok(audit.agents.missing.includes('architect.md'), 'architect.md is missing');
      assert.strictEqual(audit.commands.matching.length, 1, 'tdd.md matches');
      assert.strictEqual(audit.hooks.missing.length, 1, 'Stop hook is missing');
    });

    await test('reports all matching when configs are in sync', () => {
      const eccRoot = path.join(tmpDir, 'audit-sync-root');
      const claudeDir = path.join(tmpDir, 'audit-sync-dir');

      fs.mkdirSync(path.join(eccRoot, 'agents'), { recursive: true });
      fs.writeFileSync(path.join(eccRoot, 'agents', 'planner.md'), '# Planner');

      fs.mkdirSync(path.join(eccRoot, 'hooks'), { recursive: true });
      fs.writeFileSync(path.join(eccRoot, 'hooks', 'hooks.json'), JSON.stringify({ hooks: {} }));

      fs.mkdirSync(path.join(claudeDir, 'agents'), { recursive: true });
      fs.writeFileSync(path.join(claudeDir, 'agents', 'planner.md'), '# Planner');

      fs.writeFileSync(path.join(claudeDir, 'settings.json'), JSON.stringify({ hooks: {} }));

      const audit = auditEccConfig(eccRoot, claudeDir);

      assert.strictEqual(audit.agents.matching.length, 1);
      assert.strictEqual(audit.agents.outdated.length, 0);
      assert.strictEqual(audit.agents.missing.length, 0);
      assert.strictEqual(audit.hasDifferences, false);
    });

    await test('hasDifferences is true when any artifact differs', () => {
      const eccRoot = path.join(tmpDir, 'audit-diff-root');
      const claudeDir = path.join(tmpDir, 'audit-diff-dir');

      fs.mkdirSync(path.join(eccRoot, 'agents'), { recursive: true });
      fs.writeFileSync(path.join(eccRoot, 'agents', 'planner.md'), '# Planner v2');

      fs.mkdirSync(path.join(eccRoot, 'hooks'), { recursive: true });
      fs.writeFileSync(path.join(eccRoot, 'hooks', 'hooks.json'), JSON.stringify({ hooks: {} }));

      fs.mkdirSync(path.join(claudeDir, 'agents'), { recursive: true });
      fs.writeFileSync(path.join(claudeDir, 'agents', 'planner.md'), '# Planner v1');

      fs.writeFileSync(path.join(claudeDir, 'settings.json'), JSON.stringify({ hooks: {} }));

      const audit = auditEccConfig(eccRoot, claudeDir);

      assert.strictEqual(audit.hasDifferences, true);
    });

    // -----------------------------------------------------------------------
    // mergeHooks integration with isEccManagedHook (replaces legacy detection)
    // -----------------------------------------------------------------------
    describe('mergeHooks with ECC-managed hook replacement');

    const { mergeHooks } = require('../../src/lib/merge');

    await test('replaces numbered-prefix stale hooks during merge', () => {
      const hooksDir = path.join(tmpDir, 'hooks-numbered-src');
      const settingsDir = path.join(tmpDir, 'hooks-numbered-dest');
      fs.mkdirSync(hooksDir, { recursive: true });
      fs.mkdirSync(settingsDir, { recursive: true });

      fs.writeFileSync(
        path.join(hooksDir, 'hooks.json'),
        JSON.stringify({
          hooks: {
            Stop: [hookEntry('*', 'ecc-hook "stop:check-console-log" "dist/hooks/check-console-log.js" "standard,strict"', 'Check console')]
          }
        })
      );

      fs.writeFileSync(
        path.join(settingsDir, 'settings.json'),
        JSON.stringify({
          hooks: {
            Stop: [
              hookEntry('*', 'node "/opt/homebrew/lib/node_modules/@lebocqtitouan/ecc/10-scripts/hooks/check-console-log.js"', 'Old check console'),
              hookEntry('*', 'python3 my-custom-stop.py', 'My custom')
            ]
          }
        })
      );

      const result = mergeHooks(path.join(hooksDir, 'hooks.json'), path.join(settingsDir, 'settings.json'));

      assert.ok(result.legacyRemoved >= 1, 'Numbered-prefix hook removed');

      const settings = JSON.parse(fs.readFileSync(path.join(settingsDir, 'settings.json'), 'utf8'));
      const stopHooks = settings.hooks.Stop;

      // User hook preserved
      assert.ok(
        stopHooks.some(h => h.description === 'My custom'),
        'User hook preserved'
      );
      // Old hook removed
      assert.ok(!stopHooks.some(h => h.description === 'Old check console'), 'Stale hook removed');
      // New hook added
      assert.ok(
        stopHooks.some(h => h.hooks[0].command.includes('ecc-hook "stop:check-console-log"')),
        'Current hook added'
      );
    });

    await test('replaces mixed legacy patterns including numbered prefixes', () => {
      const hooksDir = path.join(tmpDir, 'hooks-mixed-src');
      const settingsDir = path.join(tmpDir, 'hooks-mixed-dest');
      fs.mkdirSync(hooksDir, { recursive: true });
      fs.mkdirSync(settingsDir, { recursive: true });

      fs.writeFileSync(
        path.join(hooksDir, 'hooks.json'),
        JSON.stringify({
          hooks: {
            PreToolUse: [hookEntry('Bash', 'ecc-hook "pre:bash:test" "dist/hooks/test.js" "standard,strict"', 'Test')]
          }
        })
      );

      fs.writeFileSync(
        path.join(settingsDir, 'settings.json'),
        JSON.stringify({
          hooks: {
            PreToolUse: [
              // scripts/hooks/ legacy
              hookEntry('Bash', 'node "/ecc/scripts/hooks/old-pre.js"', 'Legacy scripts'),
              // numbered prefix legacy
              hookEntry('Bash', 'node "/opt/homebrew/lib/node_modules/@lebocqtitouan/ecc/10-scripts/hooks/old-pre.js"', 'Legacy numbered'),
              // user custom
              hookEntry('Bash', 'bash ./my-hook.sh', 'My hook')
            ],
            Stop: [
              // 05-skills legacy
              hookEntry('*', 'bash "/opt/homebrew/lib/node_modules/@lebocqtitouan/ecc/05-skills/hooks/observe.sh"', 'Legacy skills')
            ]
          }
        })
      );

      const result = mergeHooks(path.join(hooksDir, 'hooks.json'), path.join(settingsDir, 'settings.json'));

      assert.ok(result.legacyRemoved >= 3, `Expected >=3 legacy removed, got ${result.legacyRemoved}`);

      const settings = JSON.parse(fs.readFileSync(path.join(settingsDir, 'settings.json'), 'utf8'));

      // User hook preserved
      const preHooks = settings.hooks.PreToolUse || [];
      assert.ok(
        preHooks.some(h => h.description === 'My hook'),
        'User hook preserved'
      );

      // No legacy references remain
      const allJson = JSON.stringify(settings.hooks);
      assert.ok(!allJson.includes('10-scripts/'), 'No 10-scripts references remain');
      assert.ok(!allJson.includes('05-skills/'), 'No 05-skills references remain');
      assert.ok(!allJson.includes('scripts/hooks/old'), 'No scripts/hooks references remain');
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
