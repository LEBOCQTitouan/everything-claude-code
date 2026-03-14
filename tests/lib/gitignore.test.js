/**
 * Tests for src/lib/gitignore.ts
 *
 * Run with: npx tsx tests/lib/gitignore.test.js
 */

const assert = require('assert');
const fs = require('fs');
const path = require('path');
const os = require('os');
const { spawnSync } = require('child_process');

const { isGitRepo, ensureGitignoreEntries, ECC_GITIGNORE_ENTRIES, findTrackedEccFiles } = require('../../src/lib/gitignore');
const { test, describe } = require('../harness');

function makeTempDir() {
  return fs.mkdtempSync(path.join(os.tmpdir(), 'ecc-gitignore-test-'));
}

function cleanup(dir) {
  fs.rmSync(dir, { recursive: true, force: true });
}

/** Initialize a git repo using spawnSync (safe, no shell injection) */
function initGitRepo(dir) {
  spawnSync('git', ['init'], { cwd: dir, stdio: 'pipe' });
  spawnSync('git', ['config', 'user.email', 'test@test.com'], { cwd: dir, stdio: 'pipe' });
  spawnSync('git', ['config', 'user.name', 'Test'], { cwd: dir, stdio: 'pipe' });
}

async function runTests() {
  describe('Testing gitignore.ts');

  // --- isGitRepo ---
  describe('isGitRepo');

  const tmpDir1 = makeTempDir();
  try {
    await test('returns false for non-git directory', () => {
      assert.strictEqual(isGitRepo(tmpDir1), false);
    });
  } finally {
    cleanup(tmpDir1);
  }

  const tmpDir2 = makeTempDir();
  try {
    await test('returns true for git repo', () => {
      initGitRepo(tmpDir2);
      assert.strictEqual(isGitRepo(tmpDir2), true);
    });
  } finally {
    cleanup(tmpDir2);
  }

  // --- ECC_GITIGNORE_ENTRIES ---
  describe('ECC_GITIGNORE_ENTRIES');

  await test('contains expected entries', () => {
    const patterns = ECC_GITIGNORE_ENTRIES.map(e => e.pattern);
    assert.ok(patterns.includes('.claude/settings.local.json'));
    assert.ok(patterns.includes('.claude/.ecc-manifest.json'));
    assert.ok(patterns.includes('docs/CODEMAPS/'));
    assert.ok(patterns.includes('.claude/plans/'));
    assert.ok(patterns.includes('.mcp.json'));
    assert.ok(patterns.includes('CLAUDE.local.md'));
  });

  await test('each entry has pattern and comment', () => {
    for (const entry of ECC_GITIGNORE_ENTRIES) {
      assert.ok(entry.pattern, `Entry missing pattern`);
      assert.ok(entry.comment, `Entry ${entry.pattern} missing comment`);
    }
  });

  // --- ensureGitignoreEntries ---
  describe('ensureGitignoreEntries');

  const tmpDir3 = makeTempDir();
  try {
    await test('skips non-git repos', () => {
      const result = ensureGitignoreEntries(tmpDir3);
      assert.strictEqual(result.skipped, true);
      assert.strictEqual(result.added.length, 0);
    });
  } finally {
    cleanup(tmpDir3);
  }

  const tmpDir4 = makeTempDir();
  try {
    await test('creates .gitignore with ECC entries in git repo', () => {
      initGitRepo(tmpDir4);
      const result = ensureGitignoreEntries(tmpDir4);
      assert.strictEqual(result.skipped, false);
      assert.strictEqual(result.added.length, ECC_GITIGNORE_ENTRIES.length);

      const content = fs.readFileSync(path.join(tmpDir4, '.gitignore'), 'utf8');
      assert.ok(content.includes('# ECC (Everything Claude Code) generated files'));
      assert.ok(content.includes('.claude/settings.local.json'));
      assert.ok(content.includes('docs/CODEMAPS/'));
    });
  } finally {
    cleanup(tmpDir4);
  }

  const tmpDir5 = makeTempDir();
  try {
    await test('appends to existing .gitignore', () => {
      initGitRepo(tmpDir5);
      fs.writeFileSync(path.join(tmpDir5, '.gitignore'), 'node_modules/\n.env\n');

      const result = ensureGitignoreEntries(tmpDir5);
      assert.strictEqual(result.added.length, ECC_GITIGNORE_ENTRIES.length);

      const content = fs.readFileSync(path.join(tmpDir5, '.gitignore'), 'utf8');
      assert.ok(content.includes('node_modules/'));
      assert.ok(content.includes('.env'));
      assert.ok(content.includes('.claude/settings.local.json'));
    });
  } finally {
    cleanup(tmpDir5);
  }

  const tmpDir6 = makeTempDir();
  try {
    await test('does not duplicate existing entries', () => {
      initGitRepo(tmpDir6);
      fs.writeFileSync(path.join(tmpDir6, '.gitignore'), '.claude/settings.local.json\n');

      const result = ensureGitignoreEntries(tmpDir6);
      assert.ok(result.alreadyPresent.includes('.claude/settings.local.json'));
      assert.strictEqual(result.added.length, ECC_GITIGNORE_ENTRIES.length - 1);

      const content = fs.readFileSync(path.join(tmpDir6, '.gitignore'), 'utf8');
      const occurrences = content.split('.claude/settings.local.json').length - 1;
      assert.strictEqual(occurrences, 1);
    });
  } finally {
    cleanup(tmpDir6);
  }

  const tmpDir7 = makeTempDir();
  try {
    await test('is idempotent (running twice produces same result)', () => {
      initGitRepo(tmpDir7);

      ensureGitignoreEntries(tmpDir7);
      const content1 = fs.readFileSync(path.join(tmpDir7, '.gitignore'), 'utf8');

      const result2 = ensureGitignoreEntries(tmpDir7);
      const content2 = fs.readFileSync(path.join(tmpDir7, '.gitignore'), 'utf8');

      assert.strictEqual(content1, content2);
      assert.strictEqual(result2.added.length, 0);
      assert.strictEqual(result2.alreadyPresent.length, ECC_GITIGNORE_ENTRIES.length);
    });
  } finally {
    cleanup(tmpDir7);
  }

  const tmpDir8 = makeTempDir();
  try {
    await test('supports custom entries', () => {
      initGitRepo(tmpDir8);
      const customEntries = [{ pattern: 'custom/path', comment: 'Custom path' }];

      const result = ensureGitignoreEntries(tmpDir8, customEntries);
      assert.strictEqual(result.added.length, 1);
      assert.ok(result.added.includes('custom/path'));

      const content = fs.readFileSync(path.join(tmpDir8, '.gitignore'), 'utf8');
      assert.ok(content.includes('custom/path'));
    });
  } finally {
    cleanup(tmpDir8);
  }

  // --- findTrackedEccFiles ---
  describe('findTrackedEccFiles');

  const tmpDir9 = makeTempDir();
  try {
    await test('returns empty for non-git repo', () => {
      const result = findTrackedEccFiles(tmpDir9);
      assert.deepStrictEqual(result, []);
    });
  } finally {
    cleanup(tmpDir9);
  }

  const tmpDir10 = makeTempDir();
  try {
    await test('detects tracked ECC files', () => {
      initGitRepo(tmpDir10);

      // Create and track a file that should be gitignored
      const claudeDir = path.join(tmpDir10, '.claude');
      fs.mkdirSync(claudeDir, { recursive: true });
      fs.writeFileSync(path.join(claudeDir, 'settings.local.json'), '{}');
      spawnSync('git', ['add', '.claude/settings.local.json'], { cwd: tmpDir10, stdio: 'pipe' });
      spawnSync('git', ['commit', '-m', 'add settings'], { cwd: tmpDir10, stdio: 'pipe' });

      const result = findTrackedEccFiles(tmpDir10);
      assert.ok(result.includes('.claude/settings.local.json'));
    });
  } finally {
    cleanup(tmpDir10);
  }
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
