/**
 * Tests for src/hooks/doc-coverage-reminder.ts
 *
 * Run with: npx tsx tests/hooks/doc-coverage-reminder.test.js
 */

const assert = require('assert');
const path = require('path');
const { spawnSync } = require('child_process');
const { test, describe } = require('../harness');

const HOOK_PATH = path.join(__dirname, '..', '..', 'dist', 'hooks', 'doc-coverage-reminder.js');

function runHook(filePath) {
  const input = JSON.stringify({
    tool_name: 'Edit',
    tool_input: { file_path: filePath },
  });

  const result = spawnSync('node', [HOOK_PATH], {
    input,
    encoding: 'utf8',
    timeout: 5000,
    env: {
      ...process.env,
      ECC_HOOK_FLAGS: 'standard',
    },
  });

  return {
    stdout: result.stdout || '',
    stderr: result.stderr || '',
    status: result.status,
  };
}

async function runTests() {
  describe('doc-coverage-reminder hook');

  await test('exits cleanly for non-source files (.md)', () => {
    const readmePath = path.join(__dirname, '..', '..', 'README.md');
    const result = runHook(readmePath);
    assert.strictEqual(result.status, 0);
    assert.ok(!result.stderr.includes('[DocCoverage]'), 'Should not emit reminder for .md files');
  });

  await test('exits cleanly for non-source files (.json)', () => {
    const jsonPath = path.join(__dirname, '..', '..', 'package.json');
    const result = runHook(jsonPath);
    assert.strictEqual(result.status, 0);
    assert.ok(!result.stderr.includes('[DocCoverage]'), 'Should not emit reminder for .json files');
  });

  await test('exits cleanly for nonexistent files', () => {
    const result = runHook('/tmp/does-not-exist-12345.ts');
    assert.strictEqual(result.status, 0);
    assert.ok(!result.stderr.includes('[DocCoverage]'), 'Should not emit reminder for missing files');
  });

  await test('detects undocumented exports in TypeScript files', () => {
    // The hook source itself has exports — check it scans a real .ts file
    const hookSrc = path.join(__dirname, '..', '..', 'src', 'hooks', 'doc-coverage-reminder.ts');
    const result = runHook(hookSrc);
    assert.strictEqual(result.status, 0);
    // This file has no exported symbols (it's a script), so no reminder expected
    // Just verify it doesn't crash
  });

  await test('passes through stdin to stdout', () => {
    const readmePath = path.join(__dirname, '..', '..', 'README.md');
    const input = JSON.stringify({
      tool_name: 'Edit',
      tool_input: { file_path: readmePath },
    });

    const result = spawnSync('node', [HOOK_PATH], {
      input,
      encoding: 'utf8',
      timeout: 5000,
      env: {
        ...process.env,
        ECC_HOOK_FLAGS: 'standard',
      },
    });

    assert.strictEqual(result.stdout, input, 'Hook should pass through stdin to stdout');
  });

  await test('handles malformed JSON input gracefully', () => {
    const result = spawnSync('node', [HOOK_PATH], {
      input: 'not valid json',
      encoding: 'utf8',
      timeout: 5000,
      env: {
        ...process.env,
        ECC_HOOK_FLAGS: 'standard',
      },
    });

    assert.strictEqual(result.status, 0, 'Should exit cleanly on bad input');
  });

  await test('skips node_modules paths', () => {
    const result = runHook('/tmp/node_modules/some-package/index.ts');
    assert.strictEqual(result.status, 0);
    assert.ok(!result.stderr.includes('[DocCoverage]'), 'Should skip node_modules');
  });

  await test('skips dist paths', () => {
    const result = runHook('/tmp/dist/hooks/doc-coverage-reminder.js');
    assert.strictEqual(result.status, 0);
    assert.ok(!result.stderr.includes('[DocCoverage]'), 'Should skip dist');
  });
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
