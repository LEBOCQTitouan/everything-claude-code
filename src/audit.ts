/**
 * ECC Audit CLI — `ecc audit`
 *
 * Reads the user's Claude Code configuration and checks against best practices.
 * Outputs a scored report with prioritised fix suggestions.
 */

import path from 'path';
import { runAllChecks } from './lib/audit-checks';
import type { AuditReport, Severity } from './lib/audit-checks';
import { bold, dim, red, yellow, green } from './lib/ansi';

// ---------------------------------------------------------------------------
// Display
// ---------------------------------------------------------------------------

const SEVERITY_COLORS: Record<Severity, (s: string) => string> = {
  critical: red,
  high: yellow,
  medium: (s: string) => s,
  low: dim
};

const SEVERITY_LABELS: Record<Severity, string> = {
  critical: 'CRIT',
  high: 'HIGH',
  medium: ' MED',
  low: ' LOW'
};

function printReport(report: AuditReport): void {
  console.log('');
  console.log(bold('ECC Configuration Audit Report'));
  console.log('═'.repeat(50));
  console.log('');

  for (const check of report.checks) {
    const icon = check.passed ? green('✓') : red('✗');
    console.log(`${icon} ${bold(check.name)}`);

    for (const finding of check.findings) {
      const color = SEVERITY_COLORS[finding.severity];
      const label = SEVERITY_LABELS[finding.severity];
      console.log(`  ${color(`[${label}]`)} ${finding.title}`);
      console.log(`         ${dim(finding.detail)}`);
      console.log(`         ${dim('Fix: ' + finding.fix)}`);
    }

    if (check.passed) {
      console.log(`  ${dim('All checks passed.')}`);
    }
    console.log('');
  }

  // Summary
  console.log('═'.repeat(50));
  const gradeColor = report.grade <= 'B' ? green : report.grade <= 'C' ? yellow : red;
  console.log(`${bold('Grade:')} ${gradeColor(report.grade)} (${report.score}/100)`);

  const allFindings = report.checks.flatMap(c => c.findings);
  const critCount = allFindings.filter(f => f.severity === 'critical').length;
  const highCount = allFindings.filter(f => f.severity === 'high').length;
  const medCount = allFindings.filter(f => f.severity === 'medium').length;
  const lowCount = allFindings.filter(f => f.severity === 'low').length;

  if (allFindings.length === 0) {
    console.log(green('No issues found. Configuration is clean.'));
  } else {
    const parts: string[] = [];
    if (critCount > 0) parts.push(red(`${critCount} critical`));
    if (highCount > 0) parts.push(yellow(`${highCount} high`));
    if (medCount > 0) parts.push(`${medCount} medium`);
    if (lowCount > 0) parts.push(dim(`${lowCount} low`));
    console.log(`${bold('Findings:')} ${parts.join(', ')}`);
  }

  console.log('');
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

function main(): void {
  const claudeDir = process.env.CLAUDE_DIR || path.join(process.env.HOME || '', '.claude');
  const projectDir = process.cwd();

  // ECC root: when running from dist/audit.js, go up 1 level
  const eccRoot = path.resolve(__dirname, '..');

  const report = runAllChecks({ claudeDir, projectDir, eccRoot });
  printReport(report);

  // Exit with non-zero if critical findings
  const hasCritical = report.checks.flatMap(c => c.findings).some(f => f.severity === 'critical');
  process.exit(hasCritical ? 1 : 0);
}

main();
