/**
 * Structural validation for plan-TDD integration.
 * Ensures the planner agent produces TDD-ready plans and the plan command
 * includes the TDD execution flow.
 *
 * Run with: npx tsx tests/ci/validate-plan-tdd.test.js
 */

const assert = require('assert');
const fs = require('fs');
const path = require('path');
const { test, describe } = require('../harness');

const ROOT = path.join(__dirname, '..', '..');

async function runTests() {
  describe('Planner agent TDD format');

  const plannerPath = path.join(ROOT, 'agents', 'planner.md');

  await test('agents/planner.md exists', () => {
    assert.ok(fs.existsSync(plannerPath), 'Missing: agents/planner.md');
  });

  await test('planner includes Test Targets subsection in plan format', () => {
    const content = fs.readFileSync(plannerPath, 'utf8');
    assert.ok(content.includes('#### Test Targets'), 'Plan format must include "#### Test Targets" subsections per phase');
  });

  await test('planner Test Targets includes required fields', () => {
    const content = fs.readFileSync(plannerPath, 'utf8');
    assert.ok(content.includes('Interfaces to scaffold'), 'Missing "Interfaces to scaffold" in Test Targets');
    assert.ok(content.includes('Unit tests'), 'Missing "Unit tests" in Test Targets');
    assert.ok(content.includes('Edge cases'), 'Missing "Edge cases" in Test Targets');
    assert.ok(content.includes('Expected test file'), 'Missing "Expected test file" in Test Targets');
  });

  await test('planner includes E2E Assessment section', () => {
    const content = fs.readFileSync(plannerPath, 'utf8');
    assert.ok(content.includes('## E2E Assessment'), 'Plan format must include "## E2E Assessment" section');
  });

  await test('planner E2E Assessment includes scope questions', () => {
    const content = fs.readFileSync(plannerPath, 'utf8');
    assert.ok(content.includes('user-facing flows'), 'E2E Assessment should check user-facing flows');
    assert.ok(content.includes('New E2E tests needed'), 'E2E Assessment should determine if new E2E tests are needed');
  });

  await test('planner commit cadence follows TDD cycle', () => {
    const content = fs.readFileSync(plannerPath, 'utf8');
    assert.ok(content.includes('test: add <phase> tests'), 'Missing RED commit template');
    assert.ok(content.includes('feat: implement <phase>'), 'Missing GREEN commit template');
    assert.ok(content.includes('refactor: improve <phase>'), 'Missing REFACTOR commit template');
  });

  await test('planner worked example includes Test Targets', () => {
    const content = fs.readFileSync(plannerPath, 'utf8');
    // The worked example should have at least one "#### Test Targets for Phase"
    const matches = content.match(/#### Test Targets for Phase \d/g);
    assert.ok(matches && matches.length >= 2, 'Worked example should include Test Targets for at least 2 phases');
  });

  describe('Plan command TDD execution flow');

  const planCmdPath = path.join(ROOT, 'commands', 'plan.md');

  await test('commands/plan.md exists', () => {
    assert.ok(fs.existsSync(planCmdPath), 'Missing: commands/plan.md');
  });

  await test('plan command includes Execution Mode section', () => {
    const content = fs.readFileSync(planCmdPath, 'utf8');
    assert.ok(content.includes('## Execution Mode'), 'Plan command must include "## Execution Mode" section');
  });

  await test('plan command includes TDD cycle keywords', () => {
    const content = fs.readFileSync(planCmdPath, 'utf8');
    assert.ok(content.includes('RED'), 'Missing RED phase in execution flow');
    assert.ok(content.includes('GREEN'), 'Missing GREEN phase in execution flow');
    assert.ok(content.includes('REFACTOR'), 'Missing REFACTOR phase in execution flow');
    assert.ok(content.includes('GATE'), 'Missing GATE step in execution flow');
  });

  await test('plan command includes E2E Testing section', () => {
    const content = fs.readFileSync(planCmdPath, 'utf8');
    assert.ok(content.includes('### E2E Testing'), 'Plan command must include "### E2E Testing" section');
  });

  await test('plan command includes Mandatory Code Review section', () => {
    const content = fs.readFileSync(planCmdPath, 'utf8');
    assert.ok(content.includes('### Mandatory Code Review'), 'Plan command must include "### Mandatory Code Review" section');
  });

  await test('plan command references tdd-guide agent', () => {
    const content = fs.readFileSync(planCmdPath, 'utf8');
    assert.ok(content.includes('tdd-guide'), 'Plan command must reference tdd-guide agent');
  });

  await test('plan command references e2e-runner agent', () => {
    const content = fs.readFileSync(planCmdPath, 'utf8');
    assert.ok(content.includes('e2e-runner'), 'Plan command must reference e2e-runner agent');
  });

  await test('plan command references code-reviewer agent', () => {
    const content = fs.readFileSync(planCmdPath, 'utf8');
    assert.ok(content.includes('code-reviewer'), 'Plan command must reference code-reviewer agent');
  });

  describe('Requirements-analyst agent');

  const analystPath = path.join(ROOT, 'agents', 'requirements-analyst.md');

  await test('agents/requirements-analyst.md exists', () => {
    assert.ok(fs.existsSync(analystPath), 'Missing: agents/requirements-analyst.md');
  });

  await test('requirements-analyst includes User Story format', () => {
    const content = fs.readFileSync(analystPath, 'utf8');
    assert.ok(content.includes('As a'), 'Missing "As a" in User Story format');
    assert.ok(content.includes('Acceptance Criteria'), 'Missing "Acceptance Criteria" in User Story format');
  });

  await test('requirements-analyst includes challenge behavior', () => {
    const content = fs.readFileSync(analystPath, 'utf8');
    assert.ok(content.includes('challenge') || content.includes('Challenge'), 'Missing challenge behavior');
    assert.ok(content.includes('alignment') || content.includes('align'), 'Missing alignment checking');
    assert.ok(content.includes('push') || content.includes('Push'), 'Missing push-the-need behavior');
  });

  await test('requirements-analyst includes codebase validation', () => {
    const content = fs.readFileSync(analystPath, 'utf8');
    assert.ok(content.includes('Explore'), 'Missing Explore agent invocation for codebase validation');
  });

  await test('requirements-analyst includes dependency analysis', () => {
    const content = fs.readFileSync(analystPath, 'utf8');
    assert.ok(content.includes('Dependency') || content.includes('dependency'), 'Missing dependency analysis');
    assert.ok(content.includes('Layer') || content.includes('DAG'), 'Missing Layer/DAG concept');
  });

  describe('Plan command stories mode');

  await test('plan command includes Stories Mode section', () => {
    const content = fs.readFileSync(planCmdPath, 'utf8');
    assert.ok(content.includes('### Stories Mode'), 'Plan command must include "### Stories Mode" section');
  });

  await test('plan command references requirements-analyst agent', () => {
    const content = fs.readFileSync(planCmdPath, 'utf8');
    assert.ok(content.includes('requirements-analyst'), 'Plan command must reference requirements-analyst agent');
  });

  await test('plan command includes Recap Report section', () => {
    const content = fs.readFileSync(planCmdPath, 'utf8');
    assert.ok(content.includes('### Recap Report'), 'Plan command must include "### Recap Report" section');
  });

  describe('Cross-reference consistency');

  await test('development-workflow.md references /plan TDD integration', () => {
    const content = fs.readFileSync(path.join(ROOT, 'rules', 'common', 'development-workflow.md'), 'utf8');
    assert.ok(content.includes('/plan') || content.includes('plan confirmation'), 'development-workflow.md should reference /plan TDD integration');
  });

  // tdd.md and orchestrate.md were archived to commands/_archive/ during
  // the 5-command simplification. TDD is now built into /plan directly.
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
