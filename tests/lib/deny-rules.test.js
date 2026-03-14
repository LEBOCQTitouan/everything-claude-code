/**
 * Tests for src/lib/deny-rules.ts
 *
 * Run with: npx tsx tests/lib/deny-rules.test.js
 */

const assert = require('assert');
const fs = require('fs');
const path = require('path');
const os = require('os');

const { ensureDenyRules, ECC_DENY_RULES } = require('../../src/lib/deny-rules');
const { test, describe } = require('../harness');

function makeTempDir() {
  return fs.mkdtempSync(path.join(os.tmpdir(), 'ecc-deny-test-'));
}

function cleanup(dir) {
  fs.rmSync(dir, { recursive: true, force: true });
}

async function runTests() {
  describe('Testing deny-rules.ts');

  // --- ECC_DENY_RULES ---
  describe('ECC_DENY_RULES');

  await test('contains rules for .env files', () => {
    assert.ok(ECC_DENY_RULES.some(r => r.includes('.env')));
  });

  await test('contains rules for SSH keys', () => {
    assert.ok(ECC_DENY_RULES.some(r => r.includes('.ssh')));
  });

  await test('contains rules for PEM files', () => {
    assert.ok(ECC_DENY_RULES.some(r => r.includes('.pem')));
  });

  await test('contains rules for destructive bash commands', () => {
    assert.ok(ECC_DENY_RULES.some(r => r.includes('rm -rf')));
    assert.ok(ECC_DENY_RULES.some(r => r.includes('chmod 777')));
  });

  await test('contains rules for force push', () => {
    assert.ok(ECC_DENY_RULES.some(r => r.includes('--force')));
  });

  // --- ensureDenyRules ---
  describe('ensureDenyRules');

  const tmpDir1 = makeTempDir();
  try {
    await test('creates deny rules in empty settings', () => {
      const settingsPath = path.join(tmpDir1, 'settings.json');
      fs.writeFileSync(settingsPath, '{}');

      const result = ensureDenyRules(settingsPath);
      assert.strictEqual(result.added, ECC_DENY_RULES.length);
      assert.strictEqual(result.existing, 0);

      const settings = JSON.parse(fs.readFileSync(settingsPath, 'utf8'));
      assert.ok(Array.isArray(settings.permissions.deny));
      assert.strictEqual(settings.permissions.deny.length, ECC_DENY_RULES.length);
    });
  } finally {
    cleanup(tmpDir1);
  }

  const tmpDir2 = makeTempDir();
  try {
    await test('does not duplicate existing deny rules', () => {
      const settingsPath = path.join(tmpDir2, 'settings.json');
      fs.writeFileSync(settingsPath, JSON.stringify({
        permissions: { deny: [ECC_DENY_RULES[0]] }
      }));

      const result = ensureDenyRules(settingsPath);
      assert.strictEqual(result.existing, 1);
      assert.strictEqual(result.added, ECC_DENY_RULES.length - 1);

      const settings = JSON.parse(fs.readFileSync(settingsPath, 'utf8'));
      assert.strictEqual(settings.permissions.deny.length, ECC_DENY_RULES.length);
    });
  } finally {
    cleanup(tmpDir2);
  }

  const tmpDir3 = makeTempDir();
  try {
    await test('is idempotent', () => {
      const settingsPath = path.join(tmpDir3, 'settings.json');
      fs.writeFileSync(settingsPath, '{}');

      ensureDenyRules(settingsPath);
      const result2 = ensureDenyRules(settingsPath);

      assert.strictEqual(result2.added, 0);
      assert.strictEqual(result2.existing, ECC_DENY_RULES.length);
    });
  } finally {
    cleanup(tmpDir3);
  }

  const tmpDir4 = makeTempDir();
  try {
    await test('preserves existing settings fields', () => {
      const settingsPath = path.join(tmpDir4, 'settings.json');
      fs.writeFileSync(settingsPath, JSON.stringify({
        hooks: { PreToolUse: [] },
        effortLevel: 'medium'
      }));

      ensureDenyRules(settingsPath);

      const settings = JSON.parse(fs.readFileSync(settingsPath, 'utf8'));
      assert.deepStrictEqual(settings.hooks, { PreToolUse: [] });
      assert.strictEqual(settings.effortLevel, 'medium');
    });
  } finally {
    cleanup(tmpDir4);
  }

  const tmpDir5 = makeTempDir();
  try {
    await test('preserves user custom deny rules', () => {
      const settingsPath = path.join(tmpDir5, 'settings.json');
      const customRule = 'Bash(my-custom-dangerous-thing:*)';
      fs.writeFileSync(settingsPath, JSON.stringify({
        permissions: { deny: [customRule] }
      }));

      ensureDenyRules(settingsPath);

      const settings = JSON.parse(fs.readFileSync(settingsPath, 'utf8'));
      assert.ok(settings.permissions.deny.includes(customRule));
      assert.strictEqual(settings.permissions.deny.length, ECC_DENY_RULES.length + 1);
    });
  } finally {
    cleanup(tmpDir5);
  }

  const tmpDir6 = makeTempDir();
  try {
    await test('creates settings file if it does not exist', () => {
      const settingsPath = path.join(tmpDir6, 'settings.json');
      // file doesn't exist yet

      const result = ensureDenyRules(settingsPath);
      assert.strictEqual(result.added, ECC_DENY_RULES.length);

      assert.ok(fs.existsSync(settingsPath));
      const settings = JSON.parse(fs.readFileSync(settingsPath, 'utf8'));
      assert.strictEqual(settings.permissions.deny.length, ECC_DENY_RULES.length);
    });
  } finally {
    cleanup(tmpDir6);
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
