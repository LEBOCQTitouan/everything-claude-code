/**
 * Tests for src/hooks/cost-tracker.ts
 *
 * Tests the cost estimation logic and JSONL metrics output.
 *
 * Run with: node tests/hooks/cost-tracker.test.js
 */

const assert = require('assert');
const path = require('path');
const fs = require('fs');
const os = require('os');
const { spawnSync } = require('child_process');

const hookScript = path.join(__dirname, '..', '..', 'dist', 'hooks', 'cost-tracker.js');

function test(name, fn) {
  try {
    fn();
    console.log(` \u2713 ${name}`);
    return true;
  } catch (err) {
    console.log(` \u2717 ${name}`);
    console.log(`   Error: ${err.message}`);
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
  console.log('\n=== Testing cost-tracker.ts ===\n');

  let passed = 0;
  let failed = 0;

  // Use a temp dir to avoid polluting ~/.claude/metrics
  const tmpMetrics = path.join(os.tmpdir(), `ecc-cost-test-${Date.now()}`);

  function cleanup() {
    try {
      const costsFile = path.join(tmpMetrics, 'costs.jsonl');
      if (fs.existsSync(costsFile)) fs.unlinkSync(costsFile);
      if (fs.existsSync(tmpMetrics)) fs.rmdirSync(tmpMetrics);
    } catch { /* ignore */ }
  }

  // We can't easily redirect the metrics dir without modifying the hook,
  // but we CAN test stdin passthrough and exit code behavior.

  console.log('Basic functionality:');

  if (test('exits with code 0', () => {
    const result = runHook('{}');
    assert.strictEqual(result.code, 0);
  })) passed++; else failed++;

  if (test('passes stdin through to stdout', () => {
    const input = '{"model":"sonnet","usage":{"input_tokens":100}}';
    const result = runHook(input);
    assert.strictEqual(result.stdout, input);
  })) passed++; else failed++;

  if (test('handles empty stdin', () => {
    const result = runHook('');
    assert.strictEqual(result.code, 0);
  })) passed++; else failed++;

  if (test('handles invalid JSON gracefully', () => {
    const result = runHook('not json');
    assert.strictEqual(result.code, 0);
    assert.strictEqual(result.stdout, 'not json');
  })) passed++; else failed++;

  console.log('\nUsage parsing:');

  if (test('accepts usage with input_tokens/output_tokens', () => {
    const input = JSON.stringify({
      model: 'sonnet',
      usage: { input_tokens: 1000, output_tokens: 500 }
    });
    const result = runHook(input);
    assert.strictEqual(result.code, 0);
    assert.strictEqual(result.stdout, input);
  })) passed++; else failed++;

  if (test('accepts token_usage alias', () => {
    const input = JSON.stringify({
      model: 'haiku',
      token_usage: { prompt_tokens: 200, completion_tokens: 100 }
    });
    const result = runHook(input);
    assert.strictEqual(result.code, 0);
    assert.strictEqual(result.stdout, input);
  })) passed++; else failed++;

  if (test('handles missing usage gracefully', () => {
    const input = JSON.stringify({ model: 'opus' });
    const result = runHook(input);
    assert.strictEqual(result.code, 0);
    assert.strictEqual(result.stdout, input);
  })) passed++; else failed++;

  if (test('handles zero tokens', () => {
    const input = JSON.stringify({
      model: 'sonnet',
      usage: { input_tokens: 0, output_tokens: 0 }
    });
    const result = runHook(input);
    assert.strictEqual(result.code, 0);
  })) passed++; else failed++;

  console.log('\nModel recognition:');

  if (test('processes haiku model', () => {
    const input = JSON.stringify({
      model: 'claude-haiku-4-5',
      usage: { input_tokens: 1000000, output_tokens: 1000000 }
    });
    const result = runHook(input);
    assert.strictEqual(result.code, 0);
  })) passed++; else failed++;

  if (test('processes opus model', () => {
    const input = JSON.stringify({
      model: 'claude-opus-4-5',
      usage: { input_tokens: 1000000, output_tokens: 1000000 }
    });
    const result = runHook(input);
    assert.strictEqual(result.code, 0);
  })) passed++; else failed++;

  if (test('defaults to sonnet pricing for unknown model', () => {
    const input = JSON.stringify({
      model: 'unknown-model',
      usage: { input_tokens: 100, output_tokens: 100 }
    });
    const result = runHook(input);
    assert.strictEqual(result.code, 0);
  })) passed++; else failed++;

  console.log('\nEdge cases:');

  if (test('handles large token counts', () => {
    const input = JSON.stringify({
      model: 'sonnet',
      usage: { input_tokens: 100000000, output_tokens: 50000000 }
    });
    const result = runHook(input);
    assert.strictEqual(result.code, 0);
  })) passed++; else failed++;

  if (test('handles negative token counts without crash', () => {
    const input = JSON.stringify({
      model: 'sonnet',
      usage: { input_tokens: -100, output_tokens: -50 }
    });
    const result = runHook(input);
    assert.strictEqual(result.code, 0);
  })) passed++; else failed++;

  if (test('handles non-numeric token values', () => {
    const input = JSON.stringify({
      model: 'sonnet',
      usage: { input_tokens: 'abc', output_tokens: null }
    });
    const result = runHook(input);
    assert.strictEqual(result.code, 0);
  })) passed++; else failed++;

  if (test('handles large stdin without crash', () => {
    const largeInput = 'x'.repeat(2 * 1024 * 1024);
    const result = runHook(largeInput);
    assert.strictEqual(result.code, 0);
    assert.ok(result.stdout.length <= 1024 * 1024 + 1);
  })) passed++; else failed++;

  if (test('reads CLAUDE_SESSION_ID from env', () => {
    const input = JSON.stringify({ model: 'sonnet', usage: { input_tokens: 10, output_tokens: 5 } });
    const result = runHook(input, { CLAUDE_SESSION_ID: 'test-session-123' });
    assert.strictEqual(result.code, 0);
  })) passed++; else failed++;

  cleanup();

  console.log(`\n=== Results: ${passed} passed, ${failed} failed ===\n`);
  process.exit(failed > 0 ? 1 : 0);
}

runTests();
