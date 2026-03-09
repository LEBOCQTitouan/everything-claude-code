#!/usr/bin/env node
/**
 * Security scan wrapper with false-positive suppression.
 *
 * Runs ecc-agentshield, filters findings against .agentshieldignore.json,
 * displays filtered results, and interactively lets users mark false positives.
 *
 * Usage:
 *   node scripts/security-scan.js [--path <dir>] [--no-interactive]
 */

const fs = require('fs');
const path = require('path');
const { execFileSync } = require('child_process');
const readline = require('readline');

// --- Types (documented via JSDoc) ---

/**
 * @typedef {Object} IgnoreRule
 * @property {string} [file] - Exact file path match
 * @property {string} [pattern] - Regex pattern matched against title + description + evidence
 * @property {string} [severity] - Severity level: critical, high, medium, low, info
 * @property {string} reason - Why this is a false positive (required)
 */

/**
 * @typedef {Object} Finding
 * @property {string} id
 * @property {string} severity
 * @property {string} category
 * @property {string} title
 * @property {string} description
 * @property {string} [file]
 * @property {number} [line]
 * @property {string} [evidence]
 */

/**
 * @typedef {Object} ScanResult
 * @property {string} timestamp
 * @property {string} targetPath
 * @property {Finding[]} findings
 * @property {Object} scores
 */

// --- Colors (NO_COLOR-aware) ---

const NO_COLOR = process.env.NO_COLOR !== undefined;
const c = {
  bold: (s) => NO_COLOR ? s : `\x1b[1m${s}\x1b[0m`,
  red: (s) => NO_COLOR ? s : `\x1b[31m${s}\x1b[0m`,
  yellow: (s) => NO_COLOR ? s : `\x1b[33m${s}\x1b[0m`,
  green: (s) => NO_COLOR ? s : `\x1b[32m${s}\x1b[0m`,
  cyan: (s) => NO_COLOR ? s : `\x1b[36m${s}\x1b[0m`,
  dim: (s) => NO_COLOR ? s : `\x1b[2m${s}\x1b[0m`,
};

// --- Pure functions ---

/**
 * Load ignore rules from .agentshieldignore.json.
 * Returns empty array if file does not exist or is invalid.
 * @param {string} filePath
 * @returns {IgnoreRule[]}
 */
function loadIgnoreRules(filePath) {
  if (!fs.existsSync(filePath)) return [];
  try {
    const data = JSON.parse(fs.readFileSync(filePath, 'utf8'));
    return Array.isArray(data) ? data : [];
  } catch {
    return [];
  }
}

/**
 * Check if a single ignore rule matches a finding.
 * @param {IgnoreRule} rule
 * @param {Finding} finding
 * @returns {boolean}
 */
function matchesFinding(rule, finding) {
  // Must match ALL specified fields (AND logic)
  if (rule.file !== undefined && rule.file !== finding.file) return false;
  if (rule.severity !== undefined && rule.severity !== finding.severity) return false;
  if (rule.pattern !== undefined) {
    const searchText = [finding.title, finding.description, finding.evidence || ''].join(' ');
    try {
      if (!new RegExp(rule.pattern, 'i').test(searchText)) return false;
    } catch {
      return false; // Invalid regex → no match
    }
  }
  // At least one matcher must have been specified (reason alone doesn't match)
  const hasAnyMatcher = rule.file !== undefined || rule.severity !== undefined || rule.pattern !== undefined;
  return hasAnyMatcher;
}

/**
 * Filter findings by removing those matched by any ignore rule.
 * @param {Finding[]} findings
 * @param {IgnoreRule[]} rules
 * @returns {{ kept: Finding[], ignored: Finding[] }}
 */
function filterFindings(findings, rules) {
  const kept = [];
  const ignored = [];
  for (const finding of findings) {
    const isIgnored = rules.some((rule) => matchesFinding(rule, finding));
    if (isIgnored) {
      ignored.push(finding);
    } else {
      kept.push(finding);
    }
  }
  return { kept, ignored };
}

/**
 * Recalculate score from filtered findings.
 * Scoring: each critical=-25, high=-5, medium=-2, low=-1, info=0.
 * @param {Finding[]} findings
 * @returns {{ score: number, grade: string, counts: Record<string, number> }}
 */
function recalculateScore(findings) {
  const weights = { critical: -25, high: -5, medium: -2, low: -1, info: 0 };
  const counts = { critical: 0, high: 0, medium: 0, low: 0, info: 0 };
  let penalty = 0;
  for (const f of findings) {
    const sev = f.severity || 'info';
    counts[sev] = (counts[sev] || 0) + 1;
    penalty += weights[sev] || 0;
  }
  const score = Math.max(0, Math.min(100, 100 + penalty));
  let grade;
  if (score >= 90) grade = 'A';
  else if (score >= 75) grade = 'B';
  else if (score >= 60) grade = 'C';
  else if (score >= 40) grade = 'D';
  else grade = 'F';
  return { score, grade, counts };
}

/**
 * Format a finding for terminal display.
 * @param {Finding} finding
 * @param {number} [index] - Optional index number for interactive selection
 * @returns {string}
 */
function formatFinding(finding, index) {
  const sevColors = { critical: c.red, high: c.red, medium: c.yellow, low: c.dim, info: c.dim };
  const colorFn = sevColors[finding.severity] || c.dim;
  const prefix = index !== undefined ? c.cyan(`  [${index}] `) : '    ';
  const sevLabel = colorFn(`● ${finding.severity.toUpperCase()}`);
  let out = `${prefix}${sevLabel} ${c.bold(finding.title)}\n`;
  if (finding.file) {
    out += `${prefix}  ${c.dim(finding.file)}${finding.line ? c.dim(`:${finding.line}`) : ''}\n`;
  }
  if (finding.evidence) {
    out += `${prefix}  ${c.dim(`Evidence: ${finding.evidence.slice(0, 80)}`)}\n`;
  }
  return out;
}

/**
 * Format the full report for terminal display.
 * @param {Finding[]} findings
 * @param {number} ignoredCount
 * @returns {string}
 */
function formatReport(findings, ignoredCount) {
  const { score, grade, counts } = recalculateScore(findings);
  const gradeColors = { A: c.green, B: c.green, C: c.yellow, D: c.red, F: c.red };
  const gradeColor = gradeColors[grade] || c.dim;

  let out = '\n';
  out += c.bold('  Security Scan Report (filtered)\n');
  out += `  ${c.dim('─'.repeat(45))}\n`;
  out += `  Grade: ${gradeColor(`${grade} (${score}/100)`)}\n`;
  if (ignoredCount > 0) {
    out += `  ${c.dim(`${ignoredCount} finding(s) suppressed via .agentshieldignore.json`)}\n`;
  }
  out += '\n';

  const total = Object.values(counts).reduce((a, b) => a + b, 0);
  out += `  ${c.bold('Findings:')} ${total} total`;
  const parts = [];
  if (counts.critical) parts.push(c.red(`${counts.critical} critical`));
  if (counts.high) parts.push(c.red(`${counts.high} high`));
  if (counts.medium) parts.push(c.yellow(`${counts.medium} medium`));
  if (counts.low) parts.push(c.dim(`${counts.low} low`));
  if (counts.info) parts.push(c.dim(`${counts.info} info`));
  if (parts.length) out += ` — ${parts.join(', ')}`;
  out += '\n\n';

  // Group by severity
  const severityOrder = ['critical', 'high', 'medium', 'low', 'info'];
  for (const sev of severityOrder) {
    const group = findings.filter((f) => f.severity === sev);
    if (group.length === 0) continue;
    const sevColors = { critical: c.red, high: c.red, medium: c.yellow, low: c.dim, info: c.dim };
    out += `  ${(sevColors[sev] || c.dim)(`● ${sev.toUpperCase()} (${group.length})`)}\n\n`;
    for (const f of group) {
      out += formatFinding(f) + '\n';
    }
  }

  return out;
}

/**
 * Append new ignore rules to the ignore file.
 * Preserves existing rules.
 * @param {string} filePath
 * @param {IgnoreRule[]} newRules
 * @returns {IgnoreRule[]} All rules after append
 */
function appendIgnoreRules(filePath, newRules) {
  const existing = loadIgnoreRules(filePath);
  const combined = [...existing, ...newRules];
  fs.writeFileSync(filePath, JSON.stringify(combined, null, 2) + '\n', 'utf8');
  return combined;
}

// --- Interactive prompting ---

/**
 * Ask user a question and return the answer.
 * @param {string} question
 * @returns {Promise<string>}
 */
function ask(question) {
  const rl = readline.createInterface({ input: process.stdin, output: process.stdout });
  return new Promise((resolve) => {
    rl.question(question, (answer) => {
      rl.close();
      resolve(answer.trim());
    });
  });
}

/**
 * Run the full scan → filter → display → interactive loop.
 * @param {string} targetPath
 * @param {boolean} interactive
 */
async function run(targetPath, interactive) {
  const ignorePath = path.join(targetPath, '.agentshieldignore.json');

  // Loop: scan → filter → display → ask → repeat if user adds ignores
  while (true) {
    // 1. Run scan
    let scanJson;
    try {
      const raw = execFileSync(
        'npx',
        ['ecc-agentshield', 'scan', '--format', 'json', '--path', targetPath],
        { encoding: 'utf8', stdio: ['pipe', 'pipe', 'pipe'], timeout: 120000 }
      );
      scanJson = JSON.parse(raw);
    } catch (err) {
      // AgentShield exits non-zero when findings exist; output is still in stdout
      if (err.stdout) {
        try {
          scanJson = JSON.parse(err.stdout);
        } catch {
          console.error('Failed to parse AgentShield output');
          process.exit(1);
        }
      } else {
        console.error('Failed to run AgentShield:', err.message);
        process.exit(1);
      }
    }

    const allFindings = scanJson.findings || [];

    // 2. Load ignore rules and filter
    const rules = loadIgnoreRules(ignorePath);
    const { kept, ignored } = filterFindings(allFindings, rules);

    // 3. Display filtered report
    console.log(formatReport(kept, ignored.length));

    // 4. Interactive prompt
    if (!interactive || kept.length === 0) {
      const hasCriticalOrHigh = kept.some((f) => f.severity === 'critical' || f.severity === 'high');
      process.exit(hasCriticalOrHigh ? 1 : 0);
    }

    const answer = await ask('  Do any of the above look like false positives? (y/n) ');
    if (answer.toLowerCase() !== 'y' && answer.toLowerCase() !== 'yes') {
      const hasCriticalOrHigh = kept.some((f) => f.severity === 'critical' || f.severity === 'high');
      process.exit(hasCriticalOrHigh ? 1 : 0);
    }

    // 5. Display numbered list
    console.log('\n  Select findings to suppress (comma-separated numbers):\n');
    for (let i = 0; i < kept.length; i++) {
      console.log(formatFinding(kept[i], i + 1));
    }

    const selection = await ask('\n  Enter numbers: ');
    const indices = selection
      .split(',')
      .map((s) => parseInt(s.trim(), 10) - 1)
      .filter((n) => !isNaN(n) && n >= 0 && n < kept.length);

    if (indices.length === 0) {
      console.log('  No valid selections. Exiting.');
      const hasCriticalOrHigh = kept.some((f) => f.severity === 'critical' || f.severity === 'high');
      process.exit(hasCriticalOrHigh ? 1 : 0);
    }

    // 6. Prompt for reasons and build new rules
    const newRules = [];
    for (const idx of indices) {
      const finding = kept[idx];
      console.log(`\n  ${c.bold(finding.title)}`);
      const reason = await ask('  Reason for suppression: ');
      if (!reason) {
        console.log('  Skipped (reason required).');
        continue;
      }

      /** @type {IgnoreRule} */
      const rule = { reason };
      if (finding.file) rule.file = finding.file;
      if (finding.id) rule.pattern = finding.id.replace(/[-[\]/{}()*+?.\\^$|]/g, '\\$&');
      newRules.push(rule);
    }

    if (newRules.length > 0) {
      appendIgnoreRules(ignorePath, newRules);
      console.log(`\n  ${c.green(`Added ${newRules.length} rule(s) to .agentshieldignore.json`)}`);
      console.log(`  ${c.bold('Re-running scan with updated ignore rules...')}\n`);
      // Loop back to step 1
    } else {
      console.log('  No rules added. Exiting.');
      const hasCriticalOrHigh = kept.some((f) => f.severity === 'critical' || f.severity === 'high');
      process.exit(hasCriticalOrHigh ? 1 : 0);
    }
  }
}

// --- CLI entry ---

function main() {
  const args = process.argv.slice(2);
  let targetPath = process.cwd();
  let interactive = true;

  for (let i = 0; i < args.length; i++) {
    if (args[i] === '--path' && args[i + 1]) {
      targetPath = path.resolve(args[++i]);
    } else if (args[i] === '--no-interactive') {
      interactive = false;
    }
  }

  run(targetPath, interactive);
}

// Export for testing
module.exports = {
  loadIgnoreRules,
  matchesFinding,
  filterFindings,
  recalculateScore,
  formatFinding,
  formatReport,
  appendIgnoreRules,
};

// Run if invoked directly
if (require.main === module) {
  main();
}
