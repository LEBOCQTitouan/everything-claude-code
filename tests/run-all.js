#!/usr/bin/env node
/**
 * Single-process test runner.
 * Imports all test files in one tsx process to eliminate per-file startup overhead.
 *
 * Usage: npx tsx tests/run-all.js
 */

const path = require('path');
const fs = require('fs');
const { getResults, resetCounters } = require('./harness');

const testsDir = __dirname;
const testFiles = [
  'lib/utils.test.js',
  'lib/package-manager.test.js',
  'lib/session-manager.test.js',
  'lib/session-aliases.test.js',
  'lib/project-detect.test.js',
  'lib/hook-flags.test.js',
  'hooks/hooks.test.js',
  'hooks/evaluate-session.test.js',
  'hooks/suggest-compact.test.js',
  'hooks/stop-uncommitted-reminder.test.js',
  'hooks/cost-tracker.test.js',
  'integration/hooks.test.js',
  'ci/validators.test.js',
  'ci/validate-no-personal-paths.test.js',
  'scripts/claw.test.js',
  'scripts/setup-package-manager.test.js',
  'scripts/skill-create-output.test.js',
  'lib/detect.test.js',
  'lib/manifest.test.js',
  'lib/merge.test.js',
  'lib/gitignore.test.js',
  'lib/ansi.test.js',
  'lib/smart-merge.test.js',
  'hooks/doc-coverage-reminder.test.js',
  'ci/validate-doc-agents.test.js',
  'ci/validate-plan-tdd.test.js',
  'ci/validate-custom-diagrams.test.js',
  'scripts/security-scan.test.js',
  'ci/validate-security-scan.test.js',
  'lib/config-audit.test.js',
  'lib/clean.test.js',
  'ci/validate-audit-system.test.js',
  'lib/deny-rules.test.js',
  'lib/audit-checks.test.js'
];

const BOX_W = 58;
const boxLine = s => `\u2551${s.padEnd(BOX_W)}\u2551`;

async function main() {
  console.log('\u2554' + '\u2550'.repeat(BOX_W) + '\u2557');
  console.log(boxLine('           Everything Claude Code - Test Suite'));
  console.log('\u255A' + '\u2550'.repeat(BOX_W) + '\u255D');
  console.log();

  let grandPassed = 0;
  let grandFailed = 0;

  for (const testFile of testFiles) {
    const testPath = path.join(testsDir, testFile);

    if (!fs.existsSync(testPath)) {
      console.log(`\u26A0 Skipping ${testFile} (file not found)`);
      continue;
    }

    const fileStart = Date.now();
    console.log(`\n\u2501\u2501\u2501 Running ${testFile} \u2501\u2501\u2501`);

    // Snapshot process.env before each file
    const envSnapshot = { ...process.env };

    resetCounters();

    try {
      const mod = require(testPath);
      if (typeof mod.runTests === 'function') {
        await mod.runTests();
      }
    } catch (err) {
      console.log(`  FATAL: ${testFile} threw: ${err.message}`);
      grandFailed++;
    }

    const fileResults = getResults();
    grandPassed += fileResults.passed;
    grandFailed += fileResults.failed;

    const fileMs = Date.now() - fileStart;
    if (fileResults.passed > 0 || fileResults.failed > 0) {
      console.log(`\nPassed: ${fileResults.passed}`);
      console.log(`Failed: ${fileResults.failed}`);
      console.log(`Total:  ${fileResults.passed + fileResults.failed} (${fileMs}ms)`);
    }

    // Restore process.env
    for (const key of Object.keys(process.env)) {
      if (!(key in envSnapshot)) delete process.env[key];
    }
    for (const [key, val] of Object.entries(envSnapshot)) {
      process.env[key] = val;
    }

    // Clear require cache for this test file to prevent module-level side effects
    delete require.cache[require.resolve(testPath)];
  }

  const grandTotal = grandPassed + grandFailed;

  console.log('\n\u2554' + '\u2550'.repeat(BOX_W) + '\u2557');
  console.log(boxLine('                     Final Results'));
  console.log('\u2560' + '\u2550'.repeat(BOX_W) + '\u2563');
  console.log(boxLine(`  Total Tests: ${String(grandTotal).padStart(4)}`));
  console.log(boxLine(`  Passed:      ${String(grandPassed).padStart(4)}  \u2713`));
  console.log(boxLine(`  Failed:      ${String(grandFailed).padStart(4)}  ${grandFailed > 0 ? '\u2717' : ' '}`));
  console.log('\u255A' + '\u2550'.repeat(BOX_W) + '\u255D');

  process.exit(grandFailed > 0 ? 1 : 0);
}

main();
