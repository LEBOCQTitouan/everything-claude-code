/**
 * Structural validation for doc system agents, commands, and skills.
 * Ensures all new doc-* files have correct frontmatter and required sections.
 *
 * Run with: npx tsx tests/ci/validate-doc-agents.test.js
 */

const assert = require('assert');
const fs = require('fs');
const path = require('path');
const { test, describe } = require('../harness');

const ROOT = path.join(__dirname, '..', '..');

const DOC_AGENTS = [
  'agents/doc-analyzer.md',
  'agents/doc-generator.md',
  'agents/doc-validator.md',
  'agents/doc-reporter.md',
  'agents/doc-orchestrator.md',
  'agents/doc-updater.md',
  'agents/diagram-generator.md'
];

// doc-analyze, doc-generate, doc-validate, doc-coverage were archived to
// commands/_archive/ during the 5-command simplification. Only doc-suite remains.
const DOC_COMMANDS = ['commands/doc-suite.md'];

const DOC_SKILLS = [
  'skills/doc-analysis/SKILL.md',
  'skills/doc-quality-scoring/SKILL.md',
  'skills/doc-guidelines/SKILL.md',
  'skills/diagram-generation/SKILL.md',
  'skills/symbol-extraction/SKILL.md',
  'skills/behaviour-extraction/SKILL.md',
  'skills/example-extraction/SKILL.md',
  'skills/git-narrative/SKILL.md',
  'skills/config-extraction/SKILL.md',
  'skills/dependency-docs/SKILL.md',
  'skills/failure-modes/SKILL.md',
  'skills/api-reference-gen/SKILL.md',
  'skills/architecture-gen/SKILL.md',
  'skills/runbook-gen/SKILL.md',
  'skills/changelog-gen/SKILL.md',
  'skills/readme-gen/SKILL.md',
  'skills/doc-drift-detector/SKILL.md',
  'skills/doc-gap-analyser/SKILL.md'
];

const DOC_REFERENCE_FILES = [
  'skills/symbol-extraction/references/language-patterns.md',
  'skills/failure-modes/references/common-patterns.md',
  'skills/api-reference-gen/references/bad-examples.md',
  'skills/architecture-gen/assets/c4-template.md'
];

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
  describe('Doc system agent validation');

  for (const agentFile of DOC_AGENTS) {
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

  describe('Doc system command validation');

  for (const cmdFile of DOC_COMMANDS) {
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
  }

  describe('Doc system skill validation');

  for (const skillFile of DOC_SKILLS) {
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

  describe('Doc system reference file validation');

  for (const refFile of DOC_REFERENCE_FILES) {
    const filePath = path.join(ROOT, refFile);

    await test(`${refFile} exists`, () => {
      assert.ok(fs.existsSync(filePath), `Missing reference file: ${refFile}`);
    });

    await test(`${refFile} has content`, () => {
      const content = fs.readFileSync(filePath, 'utf8');
      assert.ok(content.trim().length > 0, `Reference file is empty: ${refFile}`);
    });
  }

  describe('Doc manifest schema validation');

  await test('schemas/doc-manifest.schema.json exists and is valid JSON', () => {
    const schemaPath = path.join(ROOT, 'schemas/doc-manifest.schema.json');
    assert.ok(fs.existsSync(schemaPath), 'Missing doc-manifest.schema.json');
    const content = fs.readFileSync(schemaPath, 'utf8');
    const schema = JSON.parse(content);
    assert.ok(schema.$schema, 'Missing $schema field');
    assert.ok(schema.properties, 'Missing properties field');
    assert.ok(schema.properties.files, 'Missing files property');
    assert.ok(schema.properties.version, 'Missing version property');
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
