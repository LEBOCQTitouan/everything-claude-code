/**
 * Tests for src/lib/smart-merge.ts
 *
 * Run with: npx tsx tests/lib/smart-merge.test.js
 */

const assert = require('assert');
const fs = require('fs');
const path = require('path');
const os = require('os');

const {
  generateDiff,
  readFileForMerge,
  isClaudeAvailable,
} = require('../../src/lib/smart-merge');

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

function makeTempDir() {
  return fs.mkdtempSync(path.join(os.tmpdir(), 'ecc-smart-merge-test-'));
}

function cleanup(dir) {
  fs.rmSync(dir, { recursive: true, force: true });
}

function runTests() {
  console.log('\n=== Testing smart-merge.ts ===\n');
  let passed = 0;
  let failed = 0;

  // --- isClaudeAvailable ---
  console.log('isClaudeAvailable:');

  if (test('returns a boolean', () => {
    const result = isClaudeAvailable();
    assert.strictEqual(typeof result, 'boolean');
  })) passed++; else failed++;

  // --- generateDiff ---
  console.log('\ngenerateDiff:');

  if (test('shows identical lines with space prefix', () => {
    const diff = generateDiff('line1\nline2', 'line1\nline2', 'test.md');
    assert.ok(diff.includes(' line1'));
    assert.ok(diff.includes(' line2'));
    assert.ok(!diff.includes('-line'));
    assert.ok(!diff.includes('+line'));
  })) passed++; else failed++;

  if (test('shows added lines with + prefix', () => {
    const diff = generateDiff('line1', 'line1\nline2', 'test.md');
    assert.ok(diff.includes('+line2'));
  })) passed++; else failed++;

  if (test('shows removed lines with - prefix', () => {
    const diff = generateDiff('line1\nline2', 'line1', 'test.md');
    assert.ok(diff.includes('-line2'));
  })) passed++; else failed++;

  if (test('shows changed lines with both - and +', () => {
    const diff = generateDiff('old content', 'new content', 'test.md');
    assert.ok(diff.includes('-old content'));
    assert.ok(diff.includes('+new content'));
  })) passed++; else failed++;

  if (test('includes file header', () => {
    const diff = generateDiff('a', 'b', 'agent.md');
    assert.ok(diff.includes('--- existing/agent.md'));
    assert.ok(diff.includes('+++ incoming/agent.md'));
  })) passed++; else failed++;

  // --- readFileForMerge ---
  console.log('\nreadFileForMerge:');

  const tmpDir = makeTempDir();
  try {
    if (test('returns file content', () => {
      const filePath = path.join(tmpDir, 'test.md');
      fs.writeFileSync(filePath, 'hello world');
      assert.strictEqual(readFileForMerge(filePath), 'hello world');
    })) passed++; else failed++;

    if (test('returns null for non-existent file', () => {
      assert.strictEqual(readFileForMerge(path.join(tmpDir, 'missing.md')), null);
    })) passed++; else failed++;
  } finally {
    cleanup(tmpDir);
  }

  console.log(`\n${passed} passed, ${failed} failed\n`);
  if (failed > 0) process.exit(1);
}

runTests();
