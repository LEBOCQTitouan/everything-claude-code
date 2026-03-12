/**
 * Tests for src/lib/clean.ts
 *
 * Run with: npx tsx tests/lib/clean.test.js
 */

const assert = require('assert');
const fs = require('fs');
const path = require('path');
const os = require('os');

const { cleanFromManifest, cleanAll, printCleanReport } = require('../../src/lib/clean');
const { createManifest } = require('../../src/lib/manifest');
const { test, describe } = require('../harness');

function makeTempDir() {
  return fs.mkdtempSync(path.join(os.tmpdir(), 'ecc-clean-test-'));
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
    hookDescriptions: ['Test hook']
  };
}

/**
 * Set up a temp claude dir with files matching the sample manifest.
 */
function setupClaudeDir(tmpDir) {
  const claudeDir = path.join(tmpDir, '.claude');

  // Create agent files
  const agentsDir = path.join(claudeDir, 'agents');
  fs.mkdirSync(agentsDir, { recursive: true });
  fs.writeFileSync(path.join(agentsDir, 'planner.md'), '# Planner');
  fs.writeFileSync(path.join(agentsDir, 'architect.md'), '# Architect');
  fs.writeFileSync(path.join(agentsDir, 'custom-agent.md'), '# User custom agent');

  // Create command files
  const commandsDir = path.join(claudeDir, 'commands');
  fs.mkdirSync(commandsDir, { recursive: true });
  fs.writeFileSync(path.join(commandsDir, 'tdd.md'), '# TDD');
  fs.writeFileSync(path.join(commandsDir, 'plan.md'), '# Plan');

  // Create skill directories
  const skillsDir = path.join(claudeDir, 'skills');
  fs.mkdirSync(path.join(skillsDir, 'tdd-workflow'), { recursive: true });
  fs.writeFileSync(path.join(skillsDir, 'tdd-workflow', 'SKILL.md'), '# TDD Workflow');
  fs.mkdirSync(path.join(skillsDir, 'security-review'), { recursive: true });
  fs.writeFileSync(path.join(skillsDir, 'security-review', 'SKILL.md'), '# Security Review');

  // Create rule files
  const rulesCommonDir = path.join(claudeDir, 'rules', 'common');
  fs.mkdirSync(rulesCommonDir, { recursive: true });
  fs.writeFileSync(path.join(rulesCommonDir, 'coding-style.md'), '# Coding Style');
  const rulesTsDir = path.join(claudeDir, 'rules', 'typescript');
  fs.mkdirSync(rulesTsDir, { recursive: true });
  fs.writeFileSync(path.join(rulesTsDir, 'ts-rules.md'), '# TS Rules');

  // Create manifest
  const manifest = createManifest('1.0.0', ['typescript'], sampleArtifacts());
  fs.writeFileSync(path.join(claudeDir, '.ecc-manifest.json'), JSON.stringify(manifest, null, 2));

  // Create settings.json with ECC hooks + user hooks
  const settings = {
    hooks: {
      PreToolUse: [
        {
          description: 'ECC hook',
          hooks: [{ command: 'ecc-hook pre-tool-use' }]
        },
        {
          description: 'User custom hook',
          hooks: [{ command: 'my-custom-hook' }]
        }
      ]
    }
  };
  fs.writeFileSync(path.join(claudeDir, 'settings.json'), JSON.stringify(settings, null, 2));

  return claudeDir;
}

async function runTests() {
  describe('Testing clean.ts');

  const tmpDir = makeTempDir();

  try {
    // --- cleanFromManifest ---
    describe('cleanFromManifest');

    await test('removes only manifest-tracked files', () => {
      const claudeDir = setupClaudeDir(tmpDir + '/m1');
      const manifest = createManifest('1.0.0', ['typescript'], sampleArtifacts());

      const report = cleanFromManifest(claudeDir, manifest, false);

      // ECC files should be removed
      assert.ok(!fs.existsSync(path.join(claudeDir, 'agents', 'planner.md')));
      assert.ok(!fs.existsSync(path.join(claudeDir, 'agents', 'architect.md')));
      assert.ok(!fs.existsSync(path.join(claudeDir, 'commands', 'tdd.md')));
      assert.ok(!fs.existsSync(path.join(claudeDir, 'commands', 'plan.md')));
      assert.ok(!fs.existsSync(path.join(claudeDir, 'skills', 'tdd-workflow')));
      assert.ok(!fs.existsSync(path.join(claudeDir, 'skills', 'security-review')));
      assert.ok(!fs.existsSync(path.join(claudeDir, 'rules', 'common', 'coding-style.md')));
      assert.ok(!fs.existsSync(path.join(claudeDir, 'rules', 'typescript', 'ts-rules.md')));

      // User custom file should remain
      assert.ok(fs.existsSync(path.join(claudeDir, 'agents', 'custom-agent.md')));

      // Manifest itself removed
      assert.ok(!fs.existsSync(path.join(claudeDir, '.ecc-manifest.json')));

      // Report should list removed items
      assert.ok(report.removed.length > 0);
      assert.strictEqual(report.errors.length, 0);
    });

    await test('reports skipped for missing files', () => {
      const claudeDir = path.join(tmpDir, 'm2-empty');
      fs.mkdirSync(claudeDir, { recursive: true });
      const manifest = createManifest('1.0.0', ['typescript'], sampleArtifacts());

      const report = cleanFromManifest(claudeDir, manifest, false);

      // All files should be skipped (not found)
      assert.ok(report.skipped.length > 0);
      assert.strictEqual(report.removed.length, 0);
      assert.strictEqual(report.errors.length, 0);
    });

    await test('dry-run does not delete files', () => {
      const claudeDir = setupClaudeDir(tmpDir + '/m3');
      const manifest = createManifest('1.0.0', ['typescript'], sampleArtifacts());

      const report = cleanFromManifest(claudeDir, manifest, true);

      // Files should still exist
      assert.ok(fs.existsSync(path.join(claudeDir, 'agents', 'planner.md')));
      assert.ok(fs.existsSync(path.join(claudeDir, 'commands', 'tdd.md')));
      assert.ok(fs.existsSync(path.join(claudeDir, 'skills', 'tdd-workflow')));

      // Report should still list what would be removed
      assert.ok(report.removed.length > 0);
    });

    // --- cleanAll ---
    describe('cleanAll');

    await test('removes entire ECC directories', () => {
      const claudeDir = setupClaudeDir(tmpDir + '/a1');

      const report = cleanAll(claudeDir, false);

      // All ECC directories should be gone (including user custom files)
      assert.ok(!fs.existsSync(path.join(claudeDir, 'agents')));
      assert.ok(!fs.existsSync(path.join(claudeDir, 'commands')));
      assert.ok(!fs.existsSync(path.join(claudeDir, 'skills')));
      assert.ok(!fs.existsSync(path.join(claudeDir, 'rules')));

      // Manifest should be gone
      assert.ok(!fs.existsSync(path.join(claudeDir, '.ecc-manifest.json')));

      assert.ok(report.removed.length > 0);
      assert.strictEqual(report.errors.length, 0);
    });

    await test('removes ECC hooks but preserves user hooks in settings.json', () => {
      const claudeDir = setupClaudeDir(tmpDir + '/a2');

      cleanAll(claudeDir, false);

      // settings.json should still exist with user hook
      const settings = JSON.parse(fs.readFileSync(path.join(claudeDir, 'settings.json'), 'utf8'));
      assert.ok(settings.hooks);
      assert.ok(settings.hooks.PreToolUse);

      // User hook should remain
      const remaining = settings.hooks.PreToolUse;
      assert.strictEqual(remaining.length, 1);
      assert.strictEqual(remaining[0].description, 'User custom hook');
    });

    await test('dry-run does not delete directories', () => {
      const claudeDir = setupClaudeDir(tmpDir + '/a3');

      const report = cleanAll(claudeDir, true);

      // Directories should still exist
      assert.ok(fs.existsSync(path.join(claudeDir, 'agents')));
      assert.ok(fs.existsSync(path.join(claudeDir, 'commands')));
      assert.ok(fs.existsSync(path.join(claudeDir, 'skills')));
      assert.ok(fs.existsSync(path.join(claudeDir, 'rules')));

      // Report should list what would be removed
      assert.ok(report.removed.length > 0);
    });

    await test('handles missing directories gracefully', () => {
      const claudeDir = path.join(tmpDir, 'a4-empty');
      fs.mkdirSync(claudeDir, { recursive: true });

      const report = cleanAll(claudeDir, false);

      assert.ok(report.skipped.length > 0);
      assert.strictEqual(report.errors.length, 0);
    });

    // --- printCleanReport ---
    describe('printCleanReport');

    await test('does not throw for empty report', () => {
      const report = { removed: [], skipped: [], errors: [] };
      // Should not throw
      printCleanReport(report, false);
      printCleanReport(report, true);
    });

    await test('does not throw for populated report', () => {
      const report = {
        removed: ['agents/planner.md', 'commands/tdd.md'],
        skipped: ['agents/missing.md'],
        errors: ['rules/bad: EACCES']
      };
      printCleanReport(report, false);
      printCleanReport(report, true);
    });
  } finally {
    cleanup(tmpDir);
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
