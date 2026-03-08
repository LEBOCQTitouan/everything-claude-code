#!/usr/bin/env node
/**
 * Skill Creator - Pretty Output Formatter
 *
 * Creates beautiful terminal output for the /skill-create command
 */

import { bold, cyan, green, yellow, magenta, gray, white, red, dim, bgCyan } from './lib/ansi';

const chalk = { bold, cyan, green, yellow, magenta, gray, white, red, dim, bgCyan };

const BOX = {
  topLeft: '\u256d',
  topRight: '\u256e',
  bottomLeft: '\u2570',
  bottomRight: '\u256f',
  horizontal: '\u2500',
  vertical: '\u2502',
  verticalRight: '\u251c',
  verticalLeft: '\u2524'
};

const SPINNER = ['\u280b', '\u2819', '\u2839', '\u2838', '\u283c', '\u2834', '\u2826', '\u2827', '\u2807', '\u280f'];

import { stripAnsi } from './lib/ansi';

function box(title: string, content: string, width = 60): string {
  const lines = content.split('\n');
  const top = `${BOX.topLeft}${BOX.horizontal} ${chalk.bold(chalk.cyan(title))} ${BOX.horizontal.repeat(Math.max(0, width - title.length - 5))}${BOX.topRight}`;
  const bottom = `${BOX.bottomLeft}${BOX.horizontal.repeat(width - 2)}${BOX.bottomRight}`;
  const middle = lines
    .map(line => {
      const padding = width - 4 - stripAnsi(line).length;
      return `${BOX.vertical} ${line}${' '.repeat(Math.max(0, padding))} ${BOX.vertical}`;
    })
    .join('\n');
  return `${top}\n${middle}\n${bottom}`;
}

function progressBar(percent: number, width = 30): string {
  const filled = Math.min(width, Math.max(0, Math.round((width * percent) / 100)));
  const empty = width - filled;
  const bar = chalk.green('\u2588'.repeat(filled)) + chalk.gray('\u2591'.repeat(empty));
  return `${bar} ${chalk.bold(String(percent))}%`;
}

function sleep(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}

interface AnimateStep {
  name: string;
  duration?: number;
}

async function animateProgress(label: string, steps: AnimateStep[], callback?: (step: AnimateStep, index: number) => void): Promise<void> {
  process.stdout.write(`\n${chalk.cyan('\u23f3')} ${label}...\n`);

  for (let i = 0; i < steps.length; i++) {
    const step = steps[i];
    process.stdout.write(`   ${chalk.gray(SPINNER[i % SPINNER.length])} ${step.name}`);
    await sleep(step.duration || 500);
    process.stdout.clearLine?.(0);
    process.stdout.cursorTo?.(0);
    process.stdout.write(`   ${chalk.green('\u2713')} ${step.name}\n`);
    if (callback) callback(step, i);
  }
}

interface Pattern {
  name: string;
  trigger: string;
  confidence?: number;
  evidence: string;
}

interface Instinct {
  name: string;
  confidence: number;
}

export class SkillCreateOutput {
  repoName: string;
  width: number;

  constructor(repoName: string, options: { width?: number } = {}) {
    this.repoName = repoName;
    this.width = options.width || 70;
  }

  header(): void {
    const subtitle = `Extracting patterns from ${chalk.cyan(this.repoName)}`;

    console.log('\n');
    console.log(
      chalk.bold(
        chalk.magenta(
          '\u2554\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2557'
        )
      )
    );
    console.log(chalk.bold(chalk.magenta('\u2551')) + chalk.bold('  \ud83d\udd2e ECC Skill Creator                                          ') + chalk.bold(chalk.magenta('\u2551')));
    console.log(chalk.bold(chalk.magenta('\u2551')) + `     ${subtitle}${' '.repeat(Math.max(0, 59 - stripAnsi(subtitle).length))}` + chalk.bold(chalk.magenta('\u2551')));
    console.log(
      chalk.bold(
        chalk.magenta(
          '\u255a\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u255d'
        )
      )
    );
    console.log('');
  }

  async analyzePhase(data: { commits: number }): Promise<void> {
    const steps: AnimateStep[] = [
      { name: 'Parsing git history...', duration: 300 },
      { name: `Found ${chalk.yellow(String(data.commits))} commits`, duration: 200 },
      { name: 'Analyzing commit patterns...', duration: 400 },
      { name: 'Detecting file co-changes...', duration: 300 },
      { name: 'Identifying workflows...', duration: 400 },
      { name: 'Extracting architecture patterns...', duration: 300 }
    ];

    await animateProgress('Analyzing Repository', steps);
  }

  analysisResults(data: { commits: number; timeRange: string; contributors: number; files: number }): void {
    console.log('\n');
    console.log(
      box(
        '\ud83d\udcca Analysis Results',
        `
${chalk.bold('Commits Analyzed:')} ${chalk.yellow(String(data.commits))}
${chalk.bold('Time Range:')}       ${chalk.gray(data.timeRange)}
${chalk.bold('Contributors:')}     ${chalk.cyan(String(data.contributors))}
${chalk.bold('Files Tracked:')}    ${chalk.green(String(data.files))}
`
      )
    );
  }

  patterns(patterns: Pattern[]): void {
    console.log('\n');
    console.log(chalk.bold(chalk.cyan('\ud83d\udd0d Key Patterns Discovered:')));
    console.log(chalk.gray('\u2500'.repeat(50)));

    patterns.forEach((pattern, i) => {
      const confidence = pattern.confidence ?? 0.8;
      const confidenceBar = progressBar(Math.round(confidence * 100), 15);
      console.log(`
  ${chalk.bold(chalk.yellow(`${i + 1}.`))} ${chalk.bold(pattern.name)}
     ${chalk.gray('Trigger:')} ${pattern.trigger}
     ${chalk.gray('Confidence:')} ${confidenceBar}
     ${chalk.dim(pattern.evidence)}`);
    });
  }

  instincts(instincts: Instinct[]): void {
    console.log('\n');
    console.log(
      box('\ud83e\udde0 Instincts Generated', instincts.map((inst, i) => `${chalk.yellow(`${i + 1}.`)} ${chalk.bold(inst.name)} ${chalk.gray(`(${Math.round(inst.confidence * 100)}%)`)}`).join('\n'))
    );
  }

  output(skillPath: string, instinctsPath: string): void {
    console.log('\n');
    console.log(chalk.bold(chalk.green('\u2728 Generation Complete!')));
    console.log(chalk.gray('\u2500'.repeat(50)));
    console.log(`
  ${chalk.green('\ud83d\udcc4')} ${chalk.bold('Skill File:')}
     ${chalk.cyan(skillPath)}

  ${chalk.green('\ud83e\udde0')} ${chalk.bold('Instincts File:')}
     ${chalk.cyan(instinctsPath)}
`);
  }

  nextSteps(): void {
    console.log(
      box(
        '\ud83d\udccb Next Steps',
        `
${chalk.yellow('1.')} Review the generated SKILL.md
${chalk.yellow('2.')} Import instincts: ${chalk.cyan('/instinct-import <path>')}
${chalk.yellow('3.')} View learned patterns: ${chalk.cyan('/instinct-status')}
${chalk.yellow('4.')} Evolve into skills: ${chalk.cyan('/evolve')}
`
      )
    );
    console.log('\n');
  }

  footer(): void {
    console.log(chalk.gray('\u2500'.repeat(60)));
    console.log(chalk.dim(`  Powered by Everything Claude Code \u2022 ecc.tools`));
    console.log(chalk.dim(`  GitHub App: github.com/apps/skill-creator`));
    console.log('\n');
  }
}

async function demo(): Promise<void> {
  const out = new SkillCreateOutput('PMX');

  out.header();

  await out.analyzePhase({ commits: 200 });

  out.analysisResults({ commits: 200, timeRange: 'Nov 2024 - Jan 2025', contributors: 4, files: 847 });

  out.patterns([
    { name: 'Conventional Commits', trigger: 'when writing commit messages', confidence: 0.85, evidence: 'Found in 150/200 commits (feat:, fix:, refactor:)' },
    { name: 'Client/Server Component Split', trigger: 'when creating Next.js pages', confidence: 0.9, evidence: 'Observed in markets/, premarkets/, portfolio/' },
    { name: 'Service Layer Architecture', trigger: 'when adding backend logic', confidence: 0.85, evidence: 'Business logic in services/, not routes/' },
    { name: 'TDD with E2E Tests', trigger: 'when adding features', confidence: 0.75, evidence: '9 E2E test files, test(e2e) commits common' }
  ]);

  out.instincts([
    { name: 'pmx-conventional-commits', confidence: 0.85 },
    { name: 'pmx-client-component-pattern', confidence: 0.9 },
    { name: 'pmx-service-layer', confidence: 0.85 },
    { name: 'pmx-e2e-test-location', confidence: 0.9 },
    { name: 'pmx-package-manager', confidence: 0.95 },
    { name: 'pmx-hot-path-caution', confidence: 0.9 }
  ]);

  out.output('.claude/skills/pmx-patterns/SKILL.md', '.claude/homunculus/instincts/inherited/pmx-instincts.yaml');
  out.nextSteps();
  out.footer();
}

export { demo };

if (require.main === module) {
  demo().catch(console.error);
}
