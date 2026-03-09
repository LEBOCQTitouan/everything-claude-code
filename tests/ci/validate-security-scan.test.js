/**
 * Structural validation for security scan wrapper and ignore file.
 *
 * Run with: npx tsx tests/ci/validate-security-scan.test.js
 */

const assert = require('assert');
const fs = require('fs');
const path = require('path');
const { test, describe } = require('../harness');

const ROOT = path.join(__dirname, '..', '..');
const IGNORE_PATH = path.join(ROOT, '.agentshieldignore.json');
const WRAPPER_PATH = path.join(ROOT, 'scripts', 'security-scan.js');

async function runTests() {
  describe('Security scan wrapper and ignore file');

  await test('.agentshieldignore.json exists', () => {
    assert.ok(fs.existsSync(IGNORE_PATH), 'Missing: .agentshieldignore.json');
  });

  await test('.agentshieldignore.json is valid JSON array', () => {
    const content = fs.readFileSync(IGNORE_PATH, 'utf8');
    const data = JSON.parse(content);
    assert.ok(Array.isArray(data), 'Must be a JSON array');
  });

  await test('every entry has a reason field', () => {
    const data = JSON.parse(fs.readFileSync(IGNORE_PATH, 'utf8'));
    for (let i = 0; i < data.length; i++) {
      assert.ok(
        typeof data[i].reason === 'string' && data[i].reason.length > 0,
        `Entry ${i} missing or empty "reason" field`
      );
    }
  });

  await test('every entry has at least one matcher (file, pattern, or severity)', () => {
    const data = JSON.parse(fs.readFileSync(IGNORE_PATH, 'utf8'));
    for (let i = 0; i < data.length; i++) {
      const hasFile = typeof data[i].file === 'string';
      const hasPattern = typeof data[i].pattern === 'string';
      const hasSeverity = typeof data[i].severity === 'string';
      assert.ok(
        hasFile || hasPattern || hasSeverity,
        `Entry ${i} has no matcher (needs file, pattern, or severity)`
      );
    }
  });

  await test('no duplicate entries (same file + pattern + severity)', () => {
    const data = JSON.parse(fs.readFileSync(IGNORE_PATH, 'utf8'));
    const keys = data.map((d) => `${d.file || ''}|${d.pattern || ''}|${d.severity || ''}`);
    const unique = new Set(keys);
    assert.strictEqual(keys.length, unique.size, 'Duplicate ignore rules found');
  });

  await test('pattern fields are valid regex', () => {
    const data = JSON.parse(fs.readFileSync(IGNORE_PATH, 'utf8'));
    for (let i = 0; i < data.length; i++) {
      if (data[i].pattern) {
        assert.doesNotThrow(
          () => new RegExp(data[i].pattern),
          `Entry ${i} has invalid regex pattern: ${data[i].pattern}`
        );
      }
    }
  });

  await test('scripts/security-scan.js exists', () => {
    assert.ok(fs.existsSync(WRAPPER_PATH), 'Missing: scripts/security-scan.js');
  });

  await test('wrapper exports required functions', () => {
    const mod = require(WRAPPER_PATH);
    const required = ['loadIgnoreRules', 'matchesFinding', 'filterFindings', 'recalculateScore', 'appendIgnoreRules'];
    for (const fn of required) {
      assert.ok(typeof mod[fn] === 'function', `Missing export: ${fn}`);
    }
  });

  await test('security-scan skill references ignore file', () => {
    const skillPath = path.join(ROOT, 'skills', 'security-scan', 'SKILL.md');
    if (fs.existsSync(skillPath)) {
      const content = fs.readFileSync(skillPath, 'utf8');
      assert.ok(
        content.includes('agentshieldignore') || content.includes('security-scan.js'),
        'Security scan skill should reference the ignore file or wrapper'
      );
    }
    // Skill may not exist in this project — pass if missing
  });
}

if (require.main === module) {
  runTests().catch((err) => {
    console.error(err);
    process.exit(1);
  });
}

module.exports = { runTests };
