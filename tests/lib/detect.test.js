/**
 * Tests for src/lib/detect.ts
 *
 * Run with: npx tsx tests/lib/detect.test.js
 */

const assert = require('assert');
const fs = require('fs');
const path = require('path');
const os = require('os');

const {
  detectAgents,
  detectCommands,
  detectSkills,
  detectRules,
  detectHooks,
  detectClaudeMd,
  detect,
  generateReport,
} = require('../../src/lib/detect');

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
  return fs.mkdtempSync(path.join(os.tmpdir(), 'ecc-detect-test-'));
}

function cleanup(dir) {
  fs.rmSync(dir, { recursive: true, force: true });
}

function runTests() {
  console.log('\n=== Testing detect.ts ===\n');
  let passed = 0;
  let failed = 0;

  const tmpDir = makeTempDir();

  try {
    // --- detectAgents ---
    console.log('detectAgents:');

    if (test('returns empty array for non-existent dir', () => {
      const result = detectAgents(path.join(tmpDir, 'nonexistent'));
      assert.deepStrictEqual(result, []);
    })) passed++; else failed++;

    if (test('detects agents with frontmatter name', () => {
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
    })) passed++; else failed++;

    if (test('ignores non-md files in agents dir', () => {
      fs.writeFileSync(path.join(tmpDir, 'agents', 'readme.txt'), 'not an agent');
      const result = detectAgents(tmpDir);
      assert.ok(result.every(a => a.filename.endsWith('.md')));
    })) passed++; else failed++;

    // --- detectCommands ---
    console.log('\ndetectCommands:');

    if (test('returns empty for non-existent commands dir', () => {
      const result = detectCommands(path.join(tmpDir, 'nonexistent'));
      assert.deepStrictEqual(result, []);
    })) passed++; else failed++;

    if (test('detects command files', () => {
      const cmdDir = path.join(tmpDir, 'commands');
      fs.mkdirSync(cmdDir, { recursive: true });
      fs.writeFileSync(path.join(cmdDir, 'tdd.md'), '# TDD');
      fs.writeFileSync(path.join(cmdDir, 'plan.md'), '# Plan');

      const result = detectCommands(tmpDir);
      assert.strictEqual(result.length, 2);
      assert.ok(result.includes('plan.md'));
      assert.ok(result.includes('tdd.md'));
    })) passed++; else failed++;

    // --- detectSkills ---
    console.log('\ndetectSkills:');

    if (test('returns empty for non-existent skills dir', () => {
      const result = detectSkills(path.join(tmpDir, 'nonexistent'));
      assert.deepStrictEqual(result, []);
    })) passed++; else failed++;

    if (test('detects skills with and without SKILL.md', () => {
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
    })) passed++; else failed++;

    // --- detectRules ---
    console.log('\ndetectRules:');

    if (test('returns empty for non-existent rules dir', () => {
      const result = detectRules(path.join(tmpDir, 'nonexistent'));
      assert.deepStrictEqual(result, {});
    })) passed++; else failed++;

    if (test('detects rules grouped by subdirectory', () => {
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
    })) passed++; else failed++;

    // --- detectHooks ---
    console.log('\ndetectHooks:');

    if (test('returns empty for non-existent settings.json', () => {
      const result = detectHooks(path.join(tmpDir, 'nonexistent'));
      assert.deepStrictEqual(result, []);
    })) passed++; else failed++;

    if (test('detects hooks from settings.json', () => {
      const settingsDir = path.join(tmpDir, 'hooks-test');
      fs.mkdirSync(settingsDir, { recursive: true });
      fs.writeFileSync(path.join(settingsDir, 'settings.json'), JSON.stringify({
        hooks: {
          PreToolUse: [
            { matcher: 'Bash', hooks: [{ type: 'command', command: 'echo test' }], description: 'Test hook' },
          ],
          Stop: [
            { matcher: '*', hooks: [{ type: 'command', command: 'echo stop' }], description: 'Stop hook' },
          ],
        },
      }));

      const result = detectHooks(settingsDir);
      assert.strictEqual(result.length, 2);
      assert.ok(result.find(h => h.event === 'PreToolUse' && h.description === 'Test hook'));
      assert.ok(result.find(h => h.event === 'Stop' && h.description === 'Stop hook'));
    })) passed++; else failed++;

    if (test('handles malformed settings.json', () => {
      const badDir = path.join(tmpDir, 'bad-settings');
      fs.mkdirSync(badDir, { recursive: true });
      fs.writeFileSync(path.join(badDir, 'settings.json'), 'not json');
      const result = detectHooks(badDir);
      assert.deepStrictEqual(result, []);
    })) passed++; else failed++;

    // --- detectClaudeMd ---
    console.log('\ndetectClaudeMd:');

    if (test('returns empty for non-existent CLAUDE.md', () => {
      const result = detectClaudeMd(path.join(tmpDir, 'nonexistent'));
      assert.deepStrictEqual(result, []);
    })) passed++; else failed++;

    if (test('extracts ## headings from CLAUDE.md', () => {
      const projectDir = path.join(tmpDir, 'project');
      fs.mkdirSync(projectDir, { recursive: true });
      fs.writeFileSync(path.join(projectDir, 'CLAUDE.md'),
        '# Main\n## Overview\nText\n## Setup\nMore text\n### Subsection\n## Testing\n');

      const result = detectClaudeMd(projectDir);
      assert.deepStrictEqual(result, ['## Overview', '## Setup', '## Testing']);
    })) passed++; else failed++;

    // --- detect (full) ---
    console.log('\ndetect (full scan):');

    if (test('returns complete detection result', () => {
      const result = detect(tmpDir);
      assert.ok('agents' in result);
      assert.ok('commands' in result);
      assert.ok('skills' in result);
      assert.ok('rules' in result);
      assert.ok('hooks' in result);
      assert.ok('hasSettingsJson' in result);
      assert.ok(Array.isArray(result.agents));
    })) passed++; else failed++;

    // --- generateReport ---
    console.log('\ngenerateReport:');

    if (test('generates report for populated detection', () => {
      const result = detect(tmpDir);
      const report = generateReport(result);
      assert.ok(report.includes('Existing Claude Code configuration:'));
      assert.ok(report.includes('Agents:'));
    })) passed++; else failed++;

    if (test('generates report for empty detection', () => {
      const emptyDir = path.join(tmpDir, 'empty');
      fs.mkdirSync(emptyDir, { recursive: true });
      const result = detect(emptyDir);
      const report = generateReport(result);
      assert.ok(report.includes('(no existing configuration found)'));
    })) passed++; else failed++;

  } finally {
    cleanup(tmpDir);
  }

  console.log(`\n${passed} passed, ${failed} failed\n`);
  if (failed > 0) process.exit(1);
}

runTests();
