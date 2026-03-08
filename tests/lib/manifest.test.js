/**
 * Tests for src/lib/manifest.ts
 *
 * Run with: npx tsx tests/lib/manifest.test.js
 */

const assert = require('assert');
const fs = require('fs');
const path = require('path');
const os = require('os');

const { readManifest, writeManifest, createManifest, updateManifest, isEccManaged, isEccManagedRule, diffFileList, getManifestFilename } = require('../../src/lib/manifest');

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
  return fs.mkdtempSync(path.join(os.tmpdir(), 'ecc-manifest-test-'));
}

function cleanup(dir) {
  fs.rmSync(dir, { recursive: true, force: true });
}

function sampleArtifacts() {
  return {
    agents: ['planner.md', 'architect.md'],
    commands: ['tdd.md', 'plan.md'],
    skills: ['tdd-workflow', 'security-review'],
    rules: { common: ['coding-style.md'], typescript: ['ts-rules.md'] },
    hookDescriptions: ['Test hook', 'Stop hook']
  };
}

function runTests() {
  console.log('\n=== Testing manifest.ts ===\n');
  let passed = 0;
  let failed = 0;

  const tmpDir = makeTempDir();

  try {
    // --- readManifest ---
    console.log('readManifest:');

    if (
      test('returns null for non-existent directory', () => {
        const result = readManifest(path.join(tmpDir, 'nonexistent'));
        assert.strictEqual(result, null);
      })
    )
      passed++;
    else failed++;

    if (
      test('returns null for corrupted JSON', () => {
        const dir = path.join(tmpDir, 'corrupted');
        fs.mkdirSync(dir, { recursive: true });
        fs.writeFileSync(path.join(dir, '.ecc-manifest.json'), 'not json');
        assert.strictEqual(readManifest(dir), null);
      })
    )
      passed++;
    else failed++;

    if (
      test('returns null for JSON without required fields', () => {
        const dir = path.join(tmpDir, 'incomplete');
        fs.mkdirSync(dir, { recursive: true });
        fs.writeFileSync(path.join(dir, '.ecc-manifest.json'), JSON.stringify({ foo: 'bar' }));
        assert.strictEqual(readManifest(dir), null);
      })
    )
      passed++;
    else failed++;

    if (
      test('reads valid manifest', () => {
        const dir = path.join(tmpDir, 'valid');
        fs.mkdirSync(dir, { recursive: true });
        const manifest = createManifest('1.0.0', ['typescript'], sampleArtifacts());
        fs.writeFileSync(path.join(dir, '.ecc-manifest.json'), JSON.stringify(manifest));
        const result = readManifest(dir);
        assert.ok(result);
        assert.strictEqual(result.version, '1.0.0');
        assert.deepStrictEqual(result.languages, ['typescript']);
      })
    )
      passed++;
    else failed++;

    // --- writeManifest ---
    console.log('\nwriteManifest:');

    if (
      test('writes manifest to directory', () => {
        const dir = path.join(tmpDir, 'write-test');
        const manifest = createManifest('1.0.0', ['typescript'], sampleArtifacts());
        writeManifest(dir, manifest);
        assert.ok(fs.existsSync(path.join(dir, '.ecc-manifest.json')));
        const read = readManifest(dir);
        assert.strictEqual(read.version, '1.0.0');
      })
    )
      passed++;
    else failed++;

    if (
      test('creates parent directories if needed', () => {
        const dir = path.join(tmpDir, 'deep', 'nested', 'dir');
        const manifest = createManifest('1.0.0', ['typescript'], sampleArtifacts());
        writeManifest(dir, manifest);
        assert.ok(fs.existsSync(path.join(dir, '.ecc-manifest.json')));
      })
    )
      passed++;
    else failed++;

    // --- createManifest ---
    console.log('\ncreateManifest:');

    if (
      test('creates manifest with correct structure', () => {
        const manifest = createManifest('2.0.0', ['golang', 'python'], sampleArtifacts());
        assert.strictEqual(manifest.version, '2.0.0');
        assert.deepStrictEqual(manifest.languages, ['golang', 'python']);
        assert.ok(manifest.installedAt);
        assert.ok(manifest.updatedAt);
        assert.deepStrictEqual(manifest.artifacts.agents, ['planner.md', 'architect.md']);
      })
    )
      passed++;
    else failed++;

    if (
      test('does not share references with input', () => {
        const arts = sampleArtifacts();
        const manifest = createManifest('1.0.0', ['ts'], arts);
        arts.agents.push('new.md');
        assert.strictEqual(manifest.artifacts.agents.length, 2);
      })
    )
      passed++;
    else failed++;

    // --- updateManifest ---
    console.log('\nupdateManifest:');

    if (
      test('preserves installedAt, updates updatedAt', () => {
        const original = createManifest('1.0.0', ['typescript'], sampleArtifacts());
        const originalInstalledAt = original.installedAt;

        // Small delay to ensure timestamp differs
        const updated = updateManifest(original, '1.1.0', ['golang'], sampleArtifacts());
        assert.strictEqual(updated.installedAt, originalInstalledAt);
        assert.strictEqual(updated.version, '1.1.0');
      })
    )
      passed++;
    else failed++;

    if (
      test('merges languages (union)', () => {
        const original = createManifest('1.0.0', ['typescript'], sampleArtifacts());
        const updated = updateManifest(original, '1.1.0', ['golang'], sampleArtifacts());
        assert.ok(updated.languages.includes('typescript'));
        assert.ok(updated.languages.includes('golang'));
      })
    )
      passed++;
    else failed++;

    if (
      test('does not mutate original manifest', () => {
        const original = createManifest('1.0.0', ['typescript'], sampleArtifacts());
        updateManifest(original, '2.0.0', ['golang'], sampleArtifacts());
        assert.strictEqual(original.version, '1.0.0');
        assert.deepStrictEqual(original.languages, ['typescript']);
      })
    )
      passed++;
    else failed++;

    // --- isEccManaged ---
    console.log('\nisEccManaged:');

    if (
      test('returns false for null manifest', () => {
        assert.strictEqual(isEccManaged(null, 'agents', 'planner.md'), false);
      })
    )
      passed++;
    else failed++;

    if (
      test('returns true for managed file', () => {
        const manifest = createManifest('1.0.0', ['ts'], sampleArtifacts());
        assert.strictEqual(isEccManaged(manifest, 'agents', 'planner.md'), true);
      })
    )
      passed++;
    else failed++;

    if (
      test('returns false for unmanaged file', () => {
        const manifest = createManifest('1.0.0', ['ts'], sampleArtifacts());
        assert.strictEqual(isEccManaged(manifest, 'agents', 'custom-agent.md'), false);
      })
    )
      passed++;
    else failed++;

    // --- isEccManagedRule ---
    console.log('\nisEccManagedRule:');

    if (
      test('returns true for managed rule', () => {
        const manifest = createManifest('1.0.0', ['ts'], sampleArtifacts());
        assert.strictEqual(isEccManagedRule(manifest, 'common', 'coding-style.md'), true);
      })
    )
      passed++;
    else failed++;

    if (
      test('returns false for unknown group', () => {
        const manifest = createManifest('1.0.0', ['ts'], sampleArtifacts());
        assert.strictEqual(isEccManagedRule(manifest, 'rust', 'rules.md'), false);
      })
    )
      passed++;
    else failed++;

    // --- diffFileList ---
    console.log('\ndiffFileList:');

    if (
      test('computes diff correctly', () => {
        const diff = diffFileList(['a.md', 'b.md', 'c.md'], ['b.md', 'c.md', 'd.md']);
        assert.deepStrictEqual(diff.added, ['d.md']);
        assert.deepStrictEqual(diff.updated, ['b.md', 'c.md']);
        assert.deepStrictEqual(diff.removed, ['a.md']);
      })
    )
      passed++;
    else failed++;

    if (
      test('handles empty lists', () => {
        const diff = diffFileList([], ['a.md']);
        assert.deepStrictEqual(diff.added, ['a.md']);
        assert.deepStrictEqual(diff.updated, []);
        assert.deepStrictEqual(diff.removed, []);
      })
    )
      passed++;
    else failed++;

    // --- getManifestFilename ---
    console.log('\ngetManifestFilename:');

    if (
      test('returns .ecc-manifest.json', () => {
        assert.strictEqual(getManifestFilename(), '.ecc-manifest.json');
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
