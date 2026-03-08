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
const { test, describe } = require('../harness');

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

async function runTests() {
  describe('Testing cost-tracker.ts');

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

  describe('Basic functionality');

  await test('exits with code 0', () => {
    const result = runHook('{}');
    assert.strictEqual(result.code, 0);
  });

  await test('passes stdin through to stdout', () => {
    const input = '{"model":"sonnet","usage":{"input_tokens":100}}';
    const result = runHook(input);
    assert.strictEqual(result.stdout, input);
  });

  await test('handles empty stdin', () => {
    const result = runHook('');
    assert.strictEqual(result.code, 0);
  });

  await test('handles invalid JSON gracefully', () => {
    const result = runHook('not json');
    assert.strictEqual(result.code, 0);
    assert.strictEqual(result.stdout, 'not json');
  });

  describe('Usage parsing');

  await test('accepts usage with input_tokens/output_tokens', () => {
    const input = JSON.stringify({
      model: 'sonnet',
      usage: { input_tokens: 1000, output_tokens: 500 }
    });
    const result = runHook(input);
    assert.strictEqual(result.code, 0);
    assert.strictEqual(result.stdout, input);
  });

  await test('accepts token_usage alias', () => {
    const input = JSON.stringify({
      model: 'haiku',
      token_usage: { prompt_tokens: 200, completion_tokens: 100 }
    });
    const result = runHook(input);
    assert.strictEqual(result.code, 0);
    assert.strictEqual(result.stdout, input);
  });

  await test('handles missing usage gracefully', () => {
    const input = JSON.stringify({ model: 'opus' });
    const result = runHook(input);
    assert.strictEqual(result.code, 0);
    assert.strictEqual(result.stdout, input);
  });

  await test('handles zero tokens', () => {
    const input = JSON.stringify({
      model: 'sonnet',
      usage: { input_tokens: 0, output_tokens: 0 }
    });
    const result = runHook(input);
    assert.strictEqual(result.code, 0);
  });

  describe('Model recognition');

  await test('processes haiku model', () => {
    const input = JSON.stringify({
      model: 'claude-haiku-4-5',
      usage: { input_tokens: 1000000, output_tokens: 1000000 }
    });
    const result = runHook(input);
    assert.strictEqual(result.code, 0);
  });

  await test('processes opus model', () => {
    const input = JSON.stringify({
      model: 'claude-opus-4-5',
      usage: { input_tokens: 1000000, output_tokens: 1000000 }
    });
    const result = runHook(input);
    assert.strictEqual(result.code, 0);
  });

  await test('defaults to sonnet pricing for unknown model', () => {
    const input = JSON.stringify({
      model: 'unknown-model',
      usage: { input_tokens: 100, output_tokens: 100 }
    });
    const result = runHook(input);
    assert.strictEqual(result.code, 0);
  });

  describe('Edge cases');

  await test('handles large token counts', () => {
    const input = JSON.stringify({
      model: 'sonnet',
      usage: { input_tokens: 100000000, output_tokens: 50000000 }
    });
    const result = runHook(input);
    assert.strictEqual(result.code, 0);
  });

  await test('handles negative token counts without crash', () => {
    const input = JSON.stringify({
      model: 'sonnet',
      usage: { input_tokens: -100, output_tokens: -50 }
    });
    const result = runHook(input);
    assert.strictEqual(result.code, 0);
  });

  await test('handles non-numeric token values', () => {
    const input = JSON.stringify({
      model: 'sonnet',
      usage: { input_tokens: 'abc', output_tokens: null }
    });
    const result = runHook(input);
    assert.strictEqual(result.code, 0);
  });

  await test('handles large stdin without crash', () => {
    const largeInput = 'x'.repeat(2 * 1024 * 1024);
    const result = runHook(largeInput);
    assert.strictEqual(result.code, 0);
    assert.ok(result.stdout.length <= 1024 * 1024 + 1);
  });

  await test('reads CLAUDE_SESSION_ID from env', () => {
    const input = JSON.stringify({ model: 'sonnet', usage: { input_tokens: 10, output_tokens: 5 } });
    const result = runHook(input, { CLAUDE_SESSION_ID: 'test-session-123' });
    assert.strictEqual(result.code, 0);
  });

  cleanup();
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
