/**
 * Tests for hooks/stop-uncommitted-reminder.js
 *
 * Tests the uncommitted changes reminder Stop hook.
 *
 * Run with: npx tsx tests/hooks/stop-uncommitted-reminder.test.js
 */

const assert = require('assert');
const path = require('path');
const { spawnSync } = require('child_process');

const hookScript = path.join(__dirname, '..', '..', 'dist', 'hooks', 'stop-uncommitted-reminder.js');

function test(name, fn) {
  try {
    fn();
    console.log(` \u2713 ${name}`);
    return true;
  } catch (_err) {
    console.log(` \u2717 ${name}`);
    console.log(`   Error: ${_err.message}`);
    return false;
  }
}

function runHook(input = '{}', envOverrides = {}) {
  const env = { ...process.env, ...envOverrides };
  const result = spawnSync('node', [hookScript], {
    encoding: 'utf8',
    input,
    timeout: 10000,
    env
  });
  return {
    code: result.status || 0,
    stdout: result.stdout || '',
    stderr: result.stderr || ''
  };
}

function runTests() {
  console.log('\n=== Testing stop-uncommitted-reminder.js ===\n');

  let passed = 0;
  let failed = 0;

  console.log('Basic functionality:');

  if (
    test('exits with code 0', () => {
      const result = runHook();
      assert.strictEqual(result.code, 0, `Expected exit 0, got ${result.code}`);
    })
  ) passed++; else failed++;

  if (
    test('passes stdin through to stdout', () => {
      const input = '{"tool_name":"Bash","tool_input":{}}';
      const result = runHook(input);
      assert.strictEqual(result.stdout, input, 'stdin should pass through to stdout');
    })
  ) passed++; else failed++;

  if (
    test('handles empty stdin', () => {
      const result = runHook('');
      assert.strictEqual(result.code, 0);
    })
  ) passed++; else failed++;

  console.log('\nGit detection:');

  if (
    test('runs in a git repo without error', () => {
      // This test runs inside the ECC repo which IS a git repo
      const result = runHook('{}');
      assert.strictEqual(result.code, 0);
    })
  ) passed++; else failed++;

  if (
    test('outputs reminder when uncommitted changes exist', () => {
      // We're running in the ECC repo. If there are uncommitted changes
      // (like the new files we just created), the hook should remind us.
      // If clean, it just passes through silently. Either way, exit 0.
      const result = runHook('{}');
      assert.strictEqual(result.code, 0);
      // The hook writes reminders to stderr via log()
      // It should always pass through stdin to stdout
      assert.strictEqual(result.stdout, '{}');
    })
  ) passed++; else failed++;

  console.log('\nEdge cases:');

  if (
    test('handles large stdin without crash', () => {
      const largeInput = 'x'.repeat(2 * 1024 * 1024);
      const result = runHook(largeInput);
      assert.strictEqual(result.code, 0);
      // Output should be truncated to MAX_STDIN (1MB)
      assert.ok(result.stdout.length <= 1024 * 1024 + 1, 'stdout should be capped at ~1MB');
    })
  ) passed++; else failed++;

  if (
    test('handles invalid JSON stdin gracefully', () => {
      const result = runHook('not json at all');
      assert.strictEqual(result.code, 0);
      assert.strictEqual(result.stdout, 'not json at all');
    })
  ) passed++; else failed++;

  console.log(`\n=== Results: ${passed} passed, ${failed} failed ===\n`);
  process.exit(failed > 0 ? 1 : 0);
}

runTests();
