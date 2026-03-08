/**
 * Tests for src/lib/ansi.ts
 *
 * Run with: npx tsx tests/lib/ansi.test.js
 */

const assert = require('assert');
const { test, describe } = require('../harness');

// Force color output for testing (override NO_COLOR / non-TTY detection)
// We test both with and without colors by importing the module source directly.

async function runTests() {
  describe('Testing ansi.ts');

  // We can't easily toggle NO_COLOR mid-process, so we test stripAnsi
  // and the structural behavior of the color functions.

  const { stripAnsi, bold, dim, red, green, yellow, cyan, white, magenta, gray, bgCyan } = require('../../src/lib/ansi');

  // --- stripAnsi ---
  describe('stripAnsi');

  await test('removes ANSI escape sequences', () => {
    assert.strictEqual(stripAnsi('\x1b[31mhello\x1b[0m'), 'hello');
  });

  await test('handles nested ANSI codes', () => {
    assert.strictEqual(stripAnsi('\x1b[1m\x1b[31mbold red\x1b[0m\x1b[0m'), 'bold red');
  });

  await test('returns plain string unchanged', () => {
    assert.strictEqual(stripAnsi('no codes here'), 'no codes here');
  });

  await test('handles empty string', () => {
    assert.strictEqual(stripAnsi(''), '');
  });

  await test('strips multi-param codes like \\x1b[38;5;200m', () => {
    assert.strictEqual(stripAnsi('\x1b[38;5;200mtext\x1b[0m'), 'text');
  });

  // --- Color functions ---
  describe('Color functions');

  await test('all color functions are callable', () => {
    const fns = [bold, dim, red, green, yellow, cyan, white, magenta, gray, bgCyan];
    for (const fn of fns) {
      assert.strictEqual(typeof fn, 'function');
      const result = fn('test');
      assert.strictEqual(typeof result, 'string');
      // The result must contain 'test' somewhere
      assert.ok(result.includes('test'), `Expected result to contain 'test', got: ${result}`);
    }
  });

  await test('color functions return string containing original text', () => {
    assert.ok(red('error').includes('error'));
    assert.ok(green('success').includes('success'));
    assert.ok(bold('title').includes('title'));
  });

  await test('stripAnsi removes output from color functions', () => {
    assert.strictEqual(stripAnsi(red('hello')), 'hello');
    assert.strictEqual(stripAnsi(bold('world')), 'world');
    assert.strictEqual(stripAnsi(green('ok')), 'ok');
    assert.strictEqual(stripAnsi(dim('faded')), 'faded');
  });

  await test('color functions handle empty string', () => {
    const result = red('');
    assert.strictEqual(stripAnsi(result), '');
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
