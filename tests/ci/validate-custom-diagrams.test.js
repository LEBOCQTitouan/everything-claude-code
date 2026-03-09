/**
 * Structural validation for docs/diagrams/CUSTOM.md registry.
 * Ensures the registry file is well-formed and all referenced diagram files
 * and source context paths are plausible.
 *
 * Run with: npx tsx tests/ci/validate-custom-diagrams.test.js
 */

const assert = require('assert');
const fs = require('fs');
const path = require('path');
const { test, describe } = require('../harness');

const ROOT = path.join(__dirname, '..', '..');
const CUSTOM_PATH = path.join(ROOT, 'docs', 'diagrams', 'CUSTOM.md');
const VALID_TYPES = ['flowchart', 'sequence', 'class', 'er', 'state'];

function parseCustomTable(content) {
  const lines = content.split('\n');
  const tableStart = lines.findIndex(l => l.includes('| File') && l.includes('| Type'));
  if (tableStart === -1) return [];

  // Skip header and separator rows
  const rows = [];
  for (let i = tableStart + 2; i < lines.length; i++) {
    const line = lines[i].trim();
    if (!line.startsWith('|')) break;

    const cells = line.split('|').map(c => c.trim()).filter(Boolean);
    if (cells.length >= 4) {
      rows.push({
        file: cells[0],
        type: cells[1],
        title: cells[2],
        sourceContext: cells[3],
        description: cells[4] || ''
      });
    }
  }
  return rows;
}

async function runTests() {
  describe('Custom diagram registry (docs/diagrams/CUSTOM.md)');

  await test('CUSTOM.md exists', () => {
    assert.ok(fs.existsSync(CUSTOM_PATH), 'Missing: docs/diagrams/CUSTOM.md');
  });

  let content = '';
  let rows = [];

  await test('CUSTOM.md has a valid table with required columns', () => {
    content = fs.readFileSync(CUSTOM_PATH, 'utf8');
    assert.ok(
      content.includes('| File') && content.includes('| Type') && content.includes('| Title') && content.includes('| Source Context'),
      'Table must have columns: File, Type, Title, Source Context'
    );
  });

  await test('CUSTOM.md table has at least one entry', () => {
    rows = parseCustomTable(content);
    assert.ok(rows.length > 0, 'Table must have at least one diagram entry');
  });

  await test('all File values end with .md', () => {
    for (const row of rows) {
      assert.ok(row.file.endsWith('.md'), `File "${row.file}" must end with .md`);
    }
  });

  await test('all Type values are valid Mermaid types', () => {
    for (const row of rows) {
      assert.ok(
        VALID_TYPES.includes(row.type),
        `Type "${row.type}" for "${row.file}" must be one of: ${VALID_TYPES.join(', ')}`
      );
    }
  });

  await test('all Title values are non-empty', () => {
    for (const row of rows) {
      assert.ok(row.title.length > 0, `Title for "${row.file}" must not be empty`);
    }
  });

  await test('all Source Context values are non-empty', () => {
    for (const row of rows) {
      assert.ok(row.sourceContext.length > 0, `Source Context for "${row.file}" must not be empty`);
    }
  });

  await test('no duplicate File entries', () => {
    const files = rows.map(r => r.file);
    const unique = new Set(files);
    assert.strictEqual(files.length, unique.size, `Duplicate files found: ${files.filter((f, i) => files.indexOf(f) !== i).join(', ')}`);
  });

  await test('Source Context globs reference existing directories or known patterns', () => {
    for (const row of rows) {
      const globs = row.sourceContext.split(',').map(g => g.trim());
      for (const glob of globs) {
        // Extract the directory prefix (before any wildcard)
        const dirPart = glob.split('*')[0].replace(/\/$/, '');
        if (dirPart) {
          const fullPath = path.join(ROOT, dirPart);
          // Either the exact file exists, or the directory exists
          const exists = fs.existsSync(fullPath) || fs.existsSync(path.dirname(fullPath));
          assert.ok(exists, `Source context "${glob}" for "${row.file}": base path "${dirPart}" not found`);
        }
      }
    }
  });

  // Cross-reference: diagram-generator agent mentions CUSTOM.md
  await test('diagram-generator agent references CUSTOM.md', () => {
    const agentPath = path.join(ROOT, 'agents', 'diagram-generator.md');
    const agentContent = fs.readFileSync(agentPath, 'utf8');
    assert.ok(
      agentContent.includes('CUSTOM.md'),
      'agents/diagram-generator.md must reference CUSTOM.md'
    );
  });

  // Cross-reference: diagram-generation skill mentions custom registry
  await test('diagram-generation skill documents custom registry format', () => {
    const skillPath = path.join(ROOT, 'skills', 'diagram-generation', 'SKILL.md');
    const skillContent = fs.readFileSync(skillPath, 'utf8');
    assert.ok(
      skillContent.includes('## Custom Diagram Registry'),
      'skills/diagram-generation/SKILL.md must have "## Custom Diagram Registry" section'
    );
  });

  // Cross-reference: INDEX.md mentions CUSTOM.md
  await test('diagrams INDEX.md references CUSTOM.md', () => {
    const indexPath = path.join(ROOT, 'docs', 'diagrams', 'INDEX.md');
    const indexContent = fs.readFileSync(indexPath, 'utf8');
    assert.ok(
      indexContent.includes('CUSTOM.md'),
      'docs/diagrams/INDEX.md must reference CUSTOM.md'
    );
  });
}

if (require.main === module) {
  runTests().catch(err => {
    console.error(err);
    process.exit(1);
  });
}

module.exports = { runTests };
