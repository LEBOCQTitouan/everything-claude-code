/**
 * Tests for src/lib/ansi.ts
 *
 * Run with: npx tsx tests/lib/ansi.test.js
 */

const assert = require('assert');

// Force color output for testing (override NO_COLOR / non-TTY detection)
// We test both with and without colors by importing the module source directly.

function test(name, fn) {
  try {
    fn();
    console.log(`  ✓ ${name}`);
    return true;
  } catch (err) {
    console.log(`  ✗ ${name}`);
    console.log(`    Error: ${err.message}`);
    return false;
  }
}

function runTests() {
  console.log('\n=== Testing ansi.ts ===\n');
  let passed = 0;
  let failed = 0;

  // We can't easily toggle NO_COLOR mid-process, so we test stripAnsi
  // and the structural behavior of the color functions.

  const { stripAnsi, bold, dim, red, green, yellow, cyan, white, magenta, gray, bgCyan } = require('../../src/lib/ansi');

  // --- stripAnsi ---
  console.log('stripAnsi:');

  if (
    test('removes ANSI escape sequences', () => {
      assert.strictEqual(stripAnsi('\x1b[31mhello\x1b[0m'), 'hello');
    })
  )
    passed++;
  else failed++;

  if (
    test('handles nested ANSI codes', () => {
      assert.strictEqual(stripAnsi('\x1b[1m\x1b[31mbold red\x1b[0m\x1b[0m'), 'bold red');
    })
  )
    passed++;
  else failed++;

  if (
    test('returns plain string unchanged', () => {
      assert.strictEqual(stripAnsi('no codes here'), 'no codes here');
    })
  )
    passed++;
  else failed++;

  if (
    test('handles empty string', () => {
      assert.strictEqual(stripAnsi(''), '');
    })
  )
    passed++;
  else failed++;

  if (
    test('strips multi-param codes like \\x1b[38;5;200m', () => {
      assert.strictEqual(stripAnsi('\x1b[38;5;200mtext\x1b[0m'), 'text');
    })
  )
    passed++;
  else failed++;

  // --- Color functions ---
  console.log('\nColor functions:');

  if (
    test('all color functions are callable', () => {
      const fns = [bold, dim, red, green, yellow, cyan, white, magenta, gray, bgCyan];
      for (const fn of fns) {
        assert.strictEqual(typeof fn, 'function');
        const result = fn('test');
        assert.strictEqual(typeof result, 'string');
        // The result must contain 'test' somewhere
        assert.ok(result.includes('test'), `Expected result to contain 'test', got: ${result}`);
      }
    })
  )
    passed++;
  else failed++;

  if (
    test('color functions return string containing original text', () => {
      assert.ok(red('error').includes('error'));
      assert.ok(green('success').includes('success'));
      assert.ok(bold('title').includes('title'));
    })
  )
    passed++;
  else failed++;

  if (
    test('stripAnsi removes output from color functions', () => {
      assert.strictEqual(stripAnsi(red('hello')), 'hello');
      assert.strictEqual(stripAnsi(bold('world')), 'world');
      assert.strictEqual(stripAnsi(green('ok')), 'ok');
      assert.strictEqual(stripAnsi(dim('faded')), 'faded');
    })
  )
    passed++;
  else failed++;

  if (
    test('color functions handle empty string', () => {
      const result = red('');
      assert.strictEqual(stripAnsi(result), '');
    })
  )
    passed++;
  else failed++;

  console.log(`\nPassed: ${passed}`);
  console.log(`Failed: ${failed}`);
  console.log(`Total:  ${passed + failed}\n`);
  if (failed > 0) process.exit(1);
}

runTests();
