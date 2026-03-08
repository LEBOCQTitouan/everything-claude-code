/**
 * Tests for src/lib/smart-merge.ts
 *
 * Run with: npx tsx tests/lib/smart-merge.test.js
 */

const assert = require('assert');
const fs = require('fs');
const path = require('path');
const os = require('os');

const { generateDiff, readFileForMerge, isClaudeAvailable, computeLineDiff, formatSideBySideDiff } = require('../../src/lib/smart-merge');

const { stripAnsi } = require('../../src/lib/ansi');

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

  if (
    test('returns a boolean', () => {
      const result = isClaudeAvailable();
      assert.strictEqual(typeof result, 'boolean');
    })
  )
    passed++;
  else failed++;

  // --- computeLineDiff ---
  console.log('\ncomputeLineDiff:');

  if (
    test('identical lines produce all "same" entries', () => {
      const result = computeLineDiff(['a', 'b', 'c'], ['a', 'b', 'c']);
      assert.strictEqual(result.length, 3);
      assert.ok(result.every(d => d.type === 'same'));
      assert.deepStrictEqual(
        result.map(d => d.text),
        ['a', 'b', 'c']
      );
    })
  )
    passed++;
  else failed++;

  if (
    test('added lines produce "added" entries', () => {
      const result = computeLineDiff(['a'], ['a', 'b']);
      const added = result.filter(d => d.type === 'added');
      assert.strictEqual(added.length, 1);
      assert.strictEqual(added[0].text, 'b');
    })
  )
    passed++;
  else failed++;

  if (
    test('removed lines produce "removed" entries', () => {
      const result = computeLineDiff(['a', 'b'], ['a']);
      const removed = result.filter(d => d.type === 'removed');
      assert.strictEqual(removed.length, 1);
      assert.strictEqual(removed[0].text, 'b');
    })
  )
    passed++;
  else failed++;

  if (
    test('changed lines produce removed+added pair', () => {
      const result = computeLineDiff(['old'], ['new']);
      assert.strictEqual(result.length, 2);
      assert.strictEqual(result[0].type, 'removed');
      assert.strictEqual(result[0].text, 'old');
      assert.strictEqual(result[1].type, 'added');
      assert.strictEqual(result[1].text, 'new');
    })
  )
    passed++;
  else failed++;

  if (
    test('empty to non-empty produces all added', () => {
      const result = computeLineDiff([], ['a', 'b']);
      assert.strictEqual(result.length, 2);
      assert.ok(result.every(d => d.type === 'added'));
    })
  )
    passed++;
  else failed++;

  if (
    test('non-empty to empty produces all removed', () => {
      const result = computeLineDiff(['a', 'b'], []);
      assert.strictEqual(result.length, 2);
      assert.ok(result.every(d => d.type === 'removed'));
    })
  )
    passed++;
  else failed++;

  if (
    test('handles interleaved changes', () => {
      const result = computeLineDiff(['a', 'b', 'c', 'd'], ['a', 'x', 'c', 'y']);
      // a=same, b→x (removed+added), c=same, d→y (removed+added)
      const types = result.map(d => d.type);
      assert.ok(types.includes('same'));
      assert.ok(types.includes('removed'));
      assert.ok(types.includes('added'));
      // 'a' and 'c' should be same
      const sameTexts = result.filter(d => d.type === 'same').map(d => d.text);
      assert.ok(sameTexts.includes('a'));
      assert.ok(sameTexts.includes('c'));
    })
  )
    passed++;
  else failed++;

  if (
    test('falls back to simpleDiff for large files', () => {
      // 1001 * 1001 > 1_000_000 — triggers simpleDiff
      const large1 = Array.from({ length: 1001 }, (_, i) => `line${i}`);
      const large2 = Array.from({ length: 1001 }, (_, i) => `line${i}`);
      large2[500] = 'changed';
      const result = computeLineDiff(large1, large2);
      assert.ok(result.length > 0);
      const changed = result.filter(d => d.type !== 'same');
      assert.ok(changed.length > 0);
    })
  )
    passed++;
  else failed++;

  // --- formatSideBySideDiff ---
  console.log('\nformatSideBySideDiff:');

  if (
    test('includes file headers', () => {
      const diff = [{ type: 'same', text: 'hello' }];
      const output = formatSideBySideDiff(diff, 'test.md');
      const plain = stripAnsi(output);
      assert.ok(plain.includes('--- existing/test.md'));
      assert.ok(plain.includes('+++ incoming/test.md'));
    })
  )
    passed++;
  else failed++;

  if (
    test('produces multi-line output', () => {
      const diff = [
        { type: 'same', text: 'unchanged' },
        { type: 'removed', text: 'old line' },
        { type: 'added', text: 'new line' }
      ];
      const output = formatSideBySideDiff(diff, 'file.md');
      const lines = output.split('\n');
      // Header + separator + 2 content lines (same + paired removed/added) + bottom separator
      assert.ok(lines.length >= 4, `Expected at least 4 lines, got ${lines.length}`);
    })
  )
    passed++;
  else failed++;

  // --- generateDiff (integration) ---
  console.log('\ngenerateDiff:');

  if (
    test('generates diff for identical content', () => {
      const diff = generateDiff('line1\nline2', 'line1\nline2', 'test.md');
      const plain = stripAnsi(diff);
      assert.ok(plain.includes('line1'));
      assert.ok(plain.includes('line2'));
      assert.ok(plain.includes('--- existing/test.md'));
    })
  )
    passed++;
  else failed++;

  if (
    test('generates diff showing additions', () => {
      const diff = generateDiff('line1', 'line1\nline2', 'test.md');
      const plain = stripAnsi(diff);
      assert.ok(plain.includes('line2'));
    })
  )
    passed++;
  else failed++;

  if (
    test('generates diff showing removals', () => {
      const diff = generateDiff('line1\nline2', 'line1', 'test.md');
      const plain = stripAnsi(diff);
      assert.ok(plain.includes('line2'));
    })
  )
    passed++;
  else failed++;

  if (
    test('generates diff showing changes', () => {
      const diff = generateDiff('old content', 'new content', 'test.md');
      const plain = stripAnsi(diff);
      assert.ok(plain.includes('old content'));
      assert.ok(plain.includes('new content'));
    })
  )
    passed++;
  else failed++;

  if (
    test('includes file header in diff', () => {
      const diff = generateDiff('a', 'b', 'agent.md');
      const plain = stripAnsi(diff);
      assert.ok(plain.includes('--- existing/agent.md'));
      assert.ok(plain.includes('+++ incoming/agent.md'));
    })
  )
    passed++;
  else failed++;

  // --- readFileForMerge ---
  console.log('\nreadFileForMerge:');

  const tmpDir = makeTempDir();
  try {
    if (
      test('returns file content', () => {
        const filePath = path.join(tmpDir, 'test.md');
        fs.writeFileSync(filePath, 'hello world');
        assert.strictEqual(readFileForMerge(filePath), 'hello world');
      })
    )
      passed++;
    else failed++;

    if (
      test('returns null for non-existent file', () => {
        assert.strictEqual(readFileForMerge(path.join(tmpDir, 'missing.md')), null);
      })
    )
      passed++;
    else failed++;
  } finally {
    cleanup(tmpDir);
  }

  console.log(`\nPassed: ${passed}`);
  console.log(`Failed: ${failed}`);
  console.log(`Total:  ${passed + failed}\n`);
  if (failed > 0) process.exit(1);
}

runTests();
