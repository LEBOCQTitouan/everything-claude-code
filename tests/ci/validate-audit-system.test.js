/**
 * Structural validation for audit system agents, commands, and skills.
 * Ensures all audit files have correct frontmatter and required sections.
 *
 * Run with: npx tsx tests/ci/validate-audit-system.test.js
 */

const assert = require('assert');
const fs = require('fs');
const path = require('path');
const { test, describe } = require('../harness');

const ROOT = path.join(__dirname, '..', '..');

const AUDIT_AGENTS = [
  'agents/audit-orchestrator.md',
  'agents/evolution-analyst.md',
  'agents/test-auditor.md',
  'agents/observability-auditor.md',
  'agents/error-handling-auditor.md',
  'agents/convention-auditor.md',
];

const AUDIT_COMMANDS = ['commands/audit.md'];

const AUDIT_SKILLS = [
  'skills/evolutionary-analysis/SKILL.md',
  'skills/test-architecture/SKILL.md',
  'skills/observability-audit/SKILL.md',
  'skills/error-handling-audit/SKILL.md',
  'skills/convention-consistency/SKILL.md',
];

const EXTENDED_SKILL = 'skills/architecture-review/SKILL.md';

function parseFrontmatter(content) {
  const match = content.match(/^---\n([\s\S]*?)\n---/);
  if (!match) return null;
  const pairs = {};
  for (const line of match[1].split('\n')) {
    const idx = line.indexOf(':');
    if (idx > 0) {
      const key = line.substring(0, idx).trim();
      const val = line.substring(idx + 1).trim();
      pairs[key] = val;
    }
  }
  return pairs;
}

async function runTests() {
  describe('Audit system agent validation');

  for (const agentFile of AUDIT_AGENTS) {
    const filePath = path.join(ROOT, agentFile);

    await test(`${agentFile} exists`, () => {
      assert.ok(fs.existsSync(filePath), `Missing: ${agentFile}`);
    });

    await test(`${agentFile} has valid YAML frontmatter`, () => {
      const content = fs.readFileSync(filePath, 'utf8');
      const fm = parseFrontmatter(content);
      assert.ok(fm, 'Missing YAML frontmatter');
      assert.ok(fm.name, 'Missing name in frontmatter');
      assert.ok(fm.description, 'Missing description in frontmatter');
      assert.ok(fm.tools, 'Missing tools in frontmatter');
      assert.ok(fm.model, 'Missing model in frontmatter');
    });
  }

  describe('Audit system command validation');

  for (const cmdFile of AUDIT_COMMANDS) {
    const filePath = path.join(ROOT, cmdFile);

    await test(`${cmdFile} exists`, () => {
      assert.ok(fs.existsSync(filePath), `Missing: ${cmdFile}`);
    });

    await test(`${cmdFile} has description frontmatter`, () => {
      const content = fs.readFileSync(filePath, 'utf8');
      const fm = parseFrontmatter(content);
      assert.ok(fm, 'Missing YAML frontmatter');
      assert.ok(fm.description, 'Missing description in frontmatter');
    });

    await test(`${cmdFile} references plan mode`, () => {
      const content = fs.readFileSync(filePath, 'utf8');
      assert.ok(content.includes('EnterPlanMode'), 'Missing EnterPlanMode reference in audit command');
    });
  }

  describe('Audit system skill validation');

  for (const skillFile of AUDIT_SKILLS) {
    const filePath = path.join(ROOT, skillFile);

    await test(`${skillFile} exists`, () => {
      assert.ok(fs.existsSync(filePath), `Missing: ${skillFile}`);
    });

    await test(`${skillFile} has valid frontmatter`, () => {
      const content = fs.readFileSync(filePath, 'utf8');
      const fm = parseFrontmatter(content);
      assert.ok(fm, 'Missing YAML frontmatter');
      assert.ok(fm.name, 'Missing name in frontmatter');
      assert.ok(fm.description, 'Missing description in frontmatter');
    });

    await test(`${skillFile} has When to Activate section`, () => {
      const content = fs.readFileSync(filePath, 'utf8');
      assert.ok(content.includes('## When to Activate'), `Missing "When to Activate" section in ${skillFile}`);
    });

    await test(`${skillFile} has Related section`, () => {
      const content = fs.readFileSync(filePath, 'utf8');
      assert.ok(content.includes('## Related'), `Missing "Related" section in ${skillFile}`);
    });
  }

  describe('Architecture-review skill extended dimensions');

  await test(`${EXTENDED_SKILL} has Dimension 11 (Dependency Metrics)`, () => {
    const content = fs.readFileSync(path.join(ROOT, EXTENDED_SKILL), 'utf8');
    assert.ok(content.includes('### 11. Dependency Metrics'), 'Missing Dimension 11: Dependency Metrics');
  });

  await test(`${EXTENDED_SKILL} has Dimension 12 (Boundary Coherence)`, () => {
    const content = fs.readFileSync(path.join(ROOT, EXTENDED_SKILL), 'utf8');
    assert.ok(content.includes('### 12. Boundary Coherence'), 'Missing Dimension 12: Boundary Coherence');
  });

  describe('CLAUDE.md and README.md reflect audit command');

  await test('CLAUDE.md references /audit command', () => {
    const content = fs.readFileSync(path.join(ROOT, 'CLAUDE.md'), 'utf8');
    assert.ok(content.includes('/audit'), 'CLAUDE.md missing /audit reference');
    assert.ok(content.includes('6 commands'), 'CLAUDE.md should reference 6 commands');
  });

  await test('README.md references /audit command', () => {
    const content = fs.readFileSync(path.join(ROOT, 'README.md'), 'utf8');
    assert.ok(content.includes('/audit'), 'README.md missing /audit reference');
    assert.ok(content.includes('6 commands'), 'README.md should reference 6 commands');
  });
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
