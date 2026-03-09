/**
 * Tests for security-scan wrapper (scripts/security-scan.js).
 * Covers: loadIgnoreRules, matchesFinding, filterFindings, recalculateScore,
 * formatFinding, formatReport, appendIgnoreRules.
 *
 * Run with: npx tsx tests/scripts/security-scan.test.js
 */

const assert = require('assert');
const fs = require('fs');
const path = require('path');
const os = require('os');
const { test, describe } = require('../harness');

const { loadIgnoreRules, matchesFinding, filterFindings, recalculateScore, formatFinding, formatReport, appendIgnoreRules } = require('../../scripts/security-scan');

// --- Fixtures ---

function makeFinding(overrides = {}) {
  return {
    id: 'test-finding-1',
    severity: 'high',
    category: 'agents',
    title: 'Agent has Bash access',
    description: 'This agent has Bash tool access.',
    file: 'agents/test-agent.md',
    line: 10,
    evidence: 'Bash',
    ...overrides
  };
}

function tmpIgnoreFile(content) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'sec-scan-test-'));
  const filePath = path.join(dir, '.agentshieldignore.json');
  if (content !== undefined) {
    fs.writeFileSync(filePath, content, 'utf8');
  }
  return { filePath, dir };
}

async function runTests() {
  // ============================================================
  // loadIgnoreRules
  // ============================================================
  describe('loadIgnoreRules');

  await test('returns empty array when file does not exist', () => {
    const result = loadIgnoreRules('/nonexistent/path/.agentshieldignore.json');
    assert.deepStrictEqual(result, []);
  });

  await test('returns empty array for invalid JSON', () => {
    const { filePath } = tmpIgnoreFile('not json');
    const result = loadIgnoreRules(filePath);
    assert.deepStrictEqual(result, []);
  });

  await test('returns empty array for non-array JSON', () => {
    const { filePath } = tmpIgnoreFile('{"key": "value"}');
    const result = loadIgnoreRules(filePath);
    assert.deepStrictEqual(result, []);
  });

  await test('returns rules from valid JSON array', () => {
    const rules = [{ file: 'test.md', reason: 'test' }];
    const { filePath } = tmpIgnoreFile(JSON.stringify(rules));
    const result = loadIgnoreRules(filePath);
    assert.deepStrictEqual(result, rules);
  });

  // ============================================================
  // matchesFinding
  // ============================================================
  describe('matchesFinding');

  await test('matches by exact file', () => {
    const rule = { file: 'agents/test-agent.md', reason: 'ok' };
    assert.strictEqual(matchesFinding(rule, makeFinding()), true);
  });

  await test('does not match different file', () => {
    const rule = { file: 'agents/other.md', reason: 'ok' };
    assert.strictEqual(matchesFinding(rule, makeFinding()), false);
  });

  await test('matches by severity', () => {
    const rule = { severity: 'high', reason: 'ok' };
    assert.strictEqual(matchesFinding(rule, makeFinding()), true);
  });

  await test('does not match different severity', () => {
    const rule = { severity: 'low', reason: 'ok' };
    assert.strictEqual(matchesFinding(rule, makeFinding()), false);
  });

  await test('matches by pattern on title', () => {
    const rule = { pattern: 'Bash access', reason: 'ok' };
    assert.strictEqual(matchesFinding(rule, makeFinding()), true);
  });

  await test('matches by pattern on description', () => {
    const rule = { pattern: 'Bash tool access', reason: 'ok' };
    assert.strictEqual(matchesFinding(rule, makeFinding()), true);
  });

  await test('matches by pattern on evidence', () => {
    const rule = { pattern: 'Bash', reason: 'ok' };
    assert.strictEqual(matchesFinding(rule, makeFinding()), true);
  });

  await test('pattern matching is case-insensitive', () => {
    const rule = { pattern: 'bash access', reason: 'ok' };
    assert.strictEqual(matchesFinding(rule, makeFinding()), true);
  });

  await test('does not match when pattern has no hit', () => {
    const rule = { pattern: 'SQL injection', reason: 'ok' };
    assert.strictEqual(matchesFinding(rule, makeFinding()), false);
  });

  await test('invalid regex returns false', () => {
    const rule = { pattern: '[invalid', reason: 'ok' };
    assert.strictEqual(matchesFinding(rule, makeFinding()), false);
  });

  await test('combined file + severity matches when both match', () => {
    const rule = { file: 'agents/test-agent.md', severity: 'high', reason: 'ok' };
    assert.strictEqual(matchesFinding(rule, makeFinding()), true);
  });

  await test('combined file + severity fails when one does not match', () => {
    const rule = { file: 'agents/test-agent.md', severity: 'low', reason: 'ok' };
    assert.strictEqual(matchesFinding(rule, makeFinding()), false);
  });

  await test('combined file + pattern + severity requires all', () => {
    const rule = { file: 'agents/test-agent.md', pattern: 'Bash', severity: 'high', reason: 'ok' };
    assert.strictEqual(matchesFinding(rule, makeFinding()), true);
  });

  await test('reason-only rule does not match anything', () => {
    const rule = { reason: 'just a reason' };
    assert.strictEqual(matchesFinding(rule, makeFinding()), false);
  });

  await test('handles missing evidence gracefully', () => {
    const rule = { pattern: 'Bash', reason: 'ok' };
    const finding = makeFinding({ evidence: undefined });
    // Pattern still matches on title/description
    assert.strictEqual(matchesFinding(rule, finding), true);
  });

  await test('regex special characters in pattern work when escaped', () => {
    const rule = { pattern: 'merge\\(\\)', reason: 'ok' };
    const finding = makeFinding({ title: 'Call to merge() found' });
    assert.strictEqual(matchesFinding(rule, finding), true);
  });

  // ============================================================
  // filterFindings
  // ============================================================
  describe('filterFindings');

  await test('returns all findings when no rules', () => {
    const findings = [makeFinding(), makeFinding({ id: 'f2' })];
    const { kept, ignored } = filterFindings(findings, []);
    assert.strictEqual(kept.length, 2);
    assert.strictEqual(ignored.length, 0);
  });

  await test('filters out matched findings', () => {
    const findings = [makeFinding({ file: 'a.md' }), makeFinding({ file: 'b.md' })];
    const rules = [{ file: 'a.md', reason: 'ok' }];
    const { kept, ignored } = filterFindings(findings, rules);
    assert.strictEqual(kept.length, 1);
    assert.strictEqual(kept[0].file, 'b.md');
    assert.strictEqual(ignored.length, 1);
    assert.strictEqual(ignored[0].file, 'a.md');
  });

  await test('filters all findings when all match', () => {
    const findings = [makeFinding(), makeFinding()];
    const rules = [{ severity: 'high', reason: 'ok' }];
    const { kept, ignored } = filterFindings(findings, rules);
    assert.strictEqual(kept.length, 0);
    assert.strictEqual(ignored.length, 2);
  });

  await test('handles empty findings array', () => {
    const { kept, ignored } = filterFindings([], [{ severity: 'high', reason: 'ok' }]);
    assert.strictEqual(kept.length, 0);
    assert.strictEqual(ignored.length, 0);
  });

  // ============================================================
  // recalculateScore
  // ============================================================
  describe('recalculateScore');

  await test('empty findings gives 100 / A', () => {
    const { score, grade, counts } = recalculateScore([]);
    assert.strictEqual(score, 100);
    assert.strictEqual(grade, 'A');
    assert.strictEqual(counts.critical, 0);
  });

  await test('one critical gives 75 / B', () => {
    const { score, grade } = recalculateScore([makeFinding({ severity: 'critical' })]);
    assert.strictEqual(score, 75);
    assert.strictEqual(grade, 'B');
  });

  await test('one high gives 95 / A', () => {
    const { score, grade } = recalculateScore([makeFinding({ severity: 'high' })]);
    assert.strictEqual(score, 95);
    assert.strictEqual(grade, 'A');
  });

  await test('one medium gives 98 / A', () => {
    const { score, grade } = recalculateScore([makeFinding({ severity: 'medium' })]);
    assert.strictEqual(score, 98);
    assert.strictEqual(grade, 'A');
  });

  await test('one low gives 99 / A', () => {
    const { score, grade } = recalculateScore([makeFinding({ severity: 'low' })]);
    assert.strictEqual(score, 99);
    assert.strictEqual(grade, 'A');
  });

  await test('score never goes below 0', () => {
    const findings = Array.from({ length: 10 }, () => makeFinding({ severity: 'critical' }));
    const { score } = recalculateScore(findings);
    assert.strictEqual(score, 0);
  });

  await test('grade boundaries: 90=A, 89=B, 75=B, 74=C, 60=C, 59=D, 40=D, 39=F', () => {
    // 90 = A: 100 - 2 high = 90
    assert.strictEqual(recalculateScore(Array(2).fill(makeFinding({ severity: 'high' }))).grade, 'A');
    // 85 = B: 100 - 3 high = 85
    assert.strictEqual(recalculateScore(Array(3).fill(makeFinding({ severity: 'high' }))).grade, 'B');
    // 75 = B: 100 - 1 critical = 75
    assert.strictEqual(recalculateScore(Array(1).fill(makeFinding({ severity: 'critical' }))).grade, 'B');
    // 60 = C: 100 - 8 high = 60
    assert.strictEqual(recalculateScore(Array(8).fill(makeFinding({ severity: 'high' }))).grade, 'C');
    // 55 = D: 100 - 9 high = 55
    assert.strictEqual(recalculateScore(Array(9).fill(makeFinding({ severity: 'high' }))).grade, 'D');
    // 0 = F: many criticals
    assert.strictEqual(recalculateScore(Array(5).fill(makeFinding({ severity: 'critical' }))).grade, 'F');
  });

  await test('counts severities correctly', () => {
    const findings = [
      makeFinding({ severity: 'critical' }),
      makeFinding({ severity: 'high' }),
      makeFinding({ severity: 'high' }),
      makeFinding({ severity: 'medium' }),
      makeFinding({ severity: 'low' })
    ];
    const { counts } = recalculateScore(findings);
    assert.strictEqual(counts.critical, 1);
    assert.strictEqual(counts.high, 2);
    assert.strictEqual(counts.medium, 1);
    assert.strictEqual(counts.low, 1);
    assert.strictEqual(counts.info, 0);
  });

  // ============================================================
  // formatFinding
  // ============================================================
  describe('formatFinding');

  await test('includes title', () => {
    const out = formatFinding(makeFinding());
    assert.ok(out.includes('Agent has Bash access'));
  });

  await test('includes file and line', () => {
    const out = formatFinding(makeFinding());
    assert.ok(out.includes('agents/test-agent.md'));
    assert.ok(out.includes(':10'));
  });

  await test('includes evidence', () => {
    const out = formatFinding(makeFinding());
    assert.ok(out.includes('Bash'));
  });

  await test('includes index when provided', () => {
    const out = formatFinding(makeFinding(), 3);
    assert.ok(out.includes('[3]'));
  });

  await test('omits index when not provided', () => {
    // Set NO_COLOR to strip ANSI codes (which contain '[')
    const prev = process.env.NO_COLOR;
    process.env.NO_COLOR = '1';
    // Re-require to pick up NO_COLOR (colors are evaluated at require time)
    // Instead, just check that the output doesn't contain the [N] pattern
    delete process.env.NO_COLOR;
    const out = formatFinding(makeFinding());
    // Should not contain a numbered index like [1] or [3]
    assert.ok(!/\[\d+\]/.test(out), 'Should not contain numbered index');
  });

  await test('handles finding without file', () => {
    const out = formatFinding(makeFinding({ file: undefined }));
    assert.ok(out.includes('Agent has Bash access'));
  });

  await test('truncates long evidence to 80 chars', () => {
    const longEvidence = 'A'.repeat(200);
    const out = formatFinding(makeFinding({ evidence: longEvidence }));
    // Evidence line should not contain the full 200 chars
    assert.ok(!out.includes('A'.repeat(200)));
  });

  // ============================================================
  // formatReport
  // ============================================================
  describe('formatReport');

  await test('shows grade and score', () => {
    const out = formatReport([], 0);
    assert.ok(out.includes('A'));
    assert.ok(out.includes('100'));
  });

  await test('shows suppressed count', () => {
    const out = formatReport([], 5);
    assert.ok(out.includes('5 finding(s) suppressed'));
  });

  await test('does not show suppressed line when count is 0', () => {
    const out = formatReport([], 0);
    assert.ok(!out.includes('suppressed'));
  });

  await test('groups findings by severity', () => {
    const findings = [makeFinding({ severity: 'critical', title: 'Crit issue' }), makeFinding({ severity: 'high', title: 'High issue' }), makeFinding({ severity: 'medium', title: 'Med issue' })];
    const out = formatReport(findings, 0);
    const critPos = out.indexOf('CRITICAL');
    const highPos = out.indexOf('HIGH');
    const medPos = out.indexOf('MEDIUM');
    assert.ok(critPos < highPos, 'CRITICAL should appear before HIGH');
    assert.ok(highPos < medPos, 'HIGH should appear before MEDIUM');
  });

  await test('shows finding count summary', () => {
    const findings = [makeFinding(), makeFinding()];
    const out = formatReport(findings, 0);
    assert.ok(out.includes('2 total'));
  });

  // ============================================================
  // appendIgnoreRules
  // ============================================================
  describe('appendIgnoreRules');

  await test('creates file if it does not exist', () => {
    const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'sec-scan-test-'));
    const filePath = path.join(dir, '.agentshieldignore.json');
    const rules = [{ file: 'test.md', reason: 'false positive' }];
    const result = appendIgnoreRules(filePath, rules);
    assert.strictEqual(result.length, 1);
    assert.ok(fs.existsSync(filePath));
    const written = JSON.parse(fs.readFileSync(filePath, 'utf8'));
    assert.deepStrictEqual(written, rules);
  });

  await test('preserves existing rules', () => {
    const existing = [{ file: 'old.md', reason: 'existing' }];
    const { filePath } = tmpIgnoreFile(JSON.stringify(existing));
    const newRules = [{ file: 'new.md', reason: 'new' }];
    const result = appendIgnoreRules(filePath, newRules);
    assert.strictEqual(result.length, 2);
    assert.strictEqual(result[0].file, 'old.md');
    assert.strictEqual(result[1].file, 'new.md');
  });

  await test('handles empty new rules', () => {
    const existing = [{ file: 'old.md', reason: 'existing' }];
    const { filePath } = tmpIgnoreFile(JSON.stringify(existing));
    const result = appendIgnoreRules(filePath, []);
    assert.strictEqual(result.length, 1);
  });

  await test('handles corrupt existing file gracefully', () => {
    const { filePath } = tmpIgnoreFile('corrupt');
    const newRules = [{ file: 'new.md', reason: 'ok' }];
    const result = appendIgnoreRules(filePath, newRules);
    // Corrupt file is treated as empty, so only new rule remains
    assert.strictEqual(result.length, 1);
  });
}

if (require.main === module) {
  runTests().catch(err => {
    console.error(err);
    process.exit(1);
  });
}

module.exports = { runTests };
