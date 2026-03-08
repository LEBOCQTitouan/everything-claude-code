/**
 * Tests for src/lib/detect.ts
 *
 * Run with: npx tsx tests/lib/detect.test.js
 */

const assert = require('assert');
const fs = require('fs');
const path = require('path');
const os = require('os');

const { detectAgents, detectCommands, detectSkills, detectRules, detectHooks, detectClaudeMd, detect, generateReport } = require('../../src/lib/detect');
const { test, describe } = require('../harness');

function makeTempDir() {
  return fs.mkdtempSync(path.join(os.tmpdir(), 'ecc-detect-test-'));
}

function cleanup(dir) {
  fs.rmSync(dir, { recursive: true, force: true });
}

async function runTests() {
  describe('Testing detect.ts');

  const tmpDir = makeTempDir();

  try {
    // --- detectAgents ---
    describe('detectAgents');

    await test('returns empty array for non-existent dir', () => {
      const result = detectAgents(path.join(tmpDir, 'nonexistent'));
      assert.deepStrictEqual(result, []);
    });

    await test('detects agents with frontmatter name', () => {
      const agentsDir = path.join(tmpDir, 'agents');
      fs.mkdirSync(agentsDir, { recursive: true });
      fs.writeFileSync(path.join(agentsDir, 'planner.md'), '---\nname: planner\ndescription: Plans things\n---\n# Planner');
      fs.writeFileSync(path.join(agentsDir, 'custom.md'), '# Custom Agent\nNo frontmatter');

      const result = detectAgents(tmpDir);
      assert.strictEqual(result.length, 2);

      const planner = result.find(a => a.filename === 'planner.md');
      assert.ok(planner);
      assert.strictEqual(planner.name, 'planner');

      const custom = result.find(a => a.filename === 'custom.md');
      assert.ok(custom);
      assert.strictEqual(custom.name, null);
    });

    await test('ignores non-md files in agents dir', () => {
      fs.writeFileSync(path.join(tmpDir, 'agents', 'readme.txt'), 'not an agent');
      const result = detectAgents(tmpDir);
      assert.ok(result.every(a => a.filename.endsWith('.md')));
    });

    // --- detectCommands ---
    describe('detectCommands');

    await test('returns empty for non-existent commands dir', () => {
      const result = detectCommands(path.join(tmpDir, 'nonexistent'));
      assert.deepStrictEqual(result, []);
    });

    await test('detects command files', () => {
      const cmdDir = path.join(tmpDir, 'commands');
      fs.mkdirSync(cmdDir, { recursive: true });
      fs.writeFileSync(path.join(cmdDir, 'tdd.md'), '# TDD');
      fs.writeFileSync(path.join(cmdDir, 'plan.md'), '# Plan');

      const result = detectCommands(tmpDir);
      assert.strictEqual(result.length, 2);
      assert.ok(result.includes('plan.md'));
      assert.ok(result.includes('tdd.md'));
    });

    // --- detectSkills ---
    describe('detectSkills');

    await test('returns empty for non-existent skills dir', () => {
      const result = detectSkills(path.join(tmpDir, 'nonexistent'));
      assert.deepStrictEqual(result, []);
    });

    await test('detects skills with and without SKILL.md', () => {
      const skillsDir = path.join(tmpDir, 'skills');
      fs.mkdirSync(path.join(skillsDir, 'tdd-workflow'), { recursive: true });
      fs.writeFileSync(path.join(skillsDir, 'tdd-workflow', 'SKILL.md'), '# TDD');
      fs.mkdirSync(path.join(skillsDir, 'custom-skill'), { recursive: true });

      const result = detectSkills(tmpDir);
      assert.strictEqual(result.length, 2);

      const tdd = result.find(s => s.dirname === 'tdd-workflow');
      assert.ok(tdd);
      assert.strictEqual(tdd.hasSkillMd, true);

      const custom = result.find(s => s.dirname === 'custom-skill');
      assert.ok(custom);
      assert.strictEqual(custom.hasSkillMd, false);
    });

    // --- detectRules ---
    describe('detectRules');

    await test('returns empty for non-existent rules dir', () => {
      const result = detectRules(path.join(tmpDir, 'nonexistent'));
      assert.deepStrictEqual(result, {});
    });

    await test('detects rules grouped by subdirectory', () => {
      const rulesDir = path.join(tmpDir, 'rules');
      fs.mkdirSync(path.join(rulesDir, 'common'), { recursive: true });
      fs.writeFileSync(path.join(rulesDir, 'common', 'coding-style.md'), '# Style');
      fs.writeFileSync(path.join(rulesDir, 'common', 'testing.md'), '# Testing');
      fs.mkdirSync(path.join(rulesDir, 'typescript'), { recursive: true });
      fs.writeFileSync(path.join(rulesDir, 'typescript', 'ts-rules.md'), '# TS');

      const result = detectRules(tmpDir);
      assert.ok(result.common);
      assert.strictEqual(result.common.length, 2);
      assert.ok(result.typescript);
      assert.strictEqual(result.typescript.length, 1);
    });

    // --- detectHooks ---
    describe('detectHooks');

    await test('returns empty for non-existent settings.json', () => {
      const result = detectHooks(path.join(tmpDir, 'nonexistent'));
      assert.deepStrictEqual(result, []);
    });

    await test('detects hooks from settings.json', () => {
      const settingsDir = path.join(tmpDir, 'hooks-test');
      fs.mkdirSync(settingsDir, { recursive: true });
      fs.writeFileSync(
        path.join(settingsDir, 'settings.json'),
        JSON.stringify({
          hooks: {
            PreToolUse: [{ matcher: 'Bash', hooks: [{ type: 'command', command: 'echo test' }], description: 'Test hook' }],
            Stop: [{ matcher: '*', hooks: [{ type: 'command', command: 'echo stop' }], description: 'Stop hook' }]
          }
        })
      );

      const result = detectHooks(settingsDir);
      assert.strictEqual(result.length, 2);
      assert.ok(result.find(h => h.event === 'PreToolUse' && h.description === 'Test hook'));
      assert.ok(result.find(h => h.event === 'Stop' && h.description === 'Stop hook'));
    });

    await test('handles malformed settings.json', () => {
      const badDir = path.join(tmpDir, 'bad-settings');
      fs.mkdirSync(badDir, { recursive: true });
      fs.writeFileSync(path.join(badDir, 'settings.json'), 'not json');
      const result = detectHooks(badDir);
      assert.deepStrictEqual(result, []);
    });

    // --- detectClaudeMd ---
    describe('detectClaudeMd');

    await test('returns empty for non-existent CLAUDE.md', () => {
      const result = detectClaudeMd(path.join(tmpDir, 'nonexistent'));
      assert.deepStrictEqual(result, []);
    });

    await test('extracts ## headings from CLAUDE.md', () => {
      const projectDir = path.join(tmpDir, 'project');
      fs.mkdirSync(projectDir, { recursive: true });
      fs.writeFileSync(path.join(projectDir, 'CLAUDE.md'), '# Main\n## Overview\nText\n## Setup\nMore text\n### Subsection\n## Testing\n');

      const result = detectClaudeMd(projectDir);
      assert.deepStrictEqual(result, ['## Overview', '## Setup', '## Testing']);
    });

    // --- detect (full) ---
    describe('detect (full scan)');

    await test('returns complete detection result', () => {
      const result = detect(tmpDir);
      assert.ok('agents' in result);
      assert.ok('commands' in result);
      assert.ok('skills' in result);
      assert.ok('rules' in result);
      assert.ok('hooks' in result);
      assert.ok('hasSettingsJson' in result);
      assert.ok(Array.isArray(result.agents));
    });

    // --- generateReport ---
    describe('generateReport');

    await test('generates report for populated detection', () => {
      const result = detect(tmpDir);
      const report = generateReport(result);
      assert.ok(report.includes('Existing Claude Code configuration:'));
      assert.ok(report.includes('Agents:'));
    });

    await test('generates report for empty detection', () => {
      const emptyDir = path.join(tmpDir, 'empty');
      fs.mkdirSync(emptyDir, { recursive: true });
      const result = detect(emptyDir);
      const report = generateReport(result);
      assert.ok(report.includes('(no existing configuration found)'));
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
