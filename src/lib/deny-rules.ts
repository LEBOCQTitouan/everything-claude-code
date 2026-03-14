/**
 * Deny rules management: ensures sensitive path protections are in settings.json.
 * Append-only — never removes existing deny rules.
 */

import fs from 'fs';

/**
 * Default deny rules ECC should add to ~/.claude/settings.json.
 * Protects against reading/writing secrets, destructive git commands, and shell exploits.
 */
export const ECC_DENY_RULES: ReadonlyArray<string> = [
  'Read(//**/.env)',
  'Read(//**/.env.*)',
  'Write(//**/.env)',
  'Write(//**/.env.*)',
  'Read(//Users/*/.ssh/**)',
  'Read(//Users/*/.aws/**)',
  'Read(//Users/*/.gnupg/**)',
  'Read(//**/*.pem)',
  'Read(//**/*.key)',
  'Write(//**/*.pem)',
  'Write(//**/*.key)',
  'Bash(rm -rf:*)',
  'Bash(chmod 777:*)',
  'Bash(git push*--force*)'
];

/** Result of ensuring deny rules. */
export interface DenyRulesResult {
  added: number;
  existing: number;
}

/**
 * Ensure deny rules are present in settings.json.
 * Non-destructive: only adds rules that don't already exist.
 */
export function ensureDenyRules(settingsJsonPath: string): DenyRulesResult {
  const settings = fs.existsSync(settingsJsonPath)
    ? JSON.parse(fs.readFileSync(settingsJsonPath, 'utf8'))
    : {};

  const permissions = settings.permissions || {};
  const existingDeny: string[] = permissions.deny || [];
  const existingSet = new Set(existingDeny);

  let added = 0;
  let existing = 0;
  const newDeny = [...existingDeny];

  for (const rule of ECC_DENY_RULES) {
    if (existingSet.has(rule)) {
      existing++;
    } else {
      newDeny.push(rule);
      added++;
    }
  }

  if (added > 0) {
    const updated = {
      ...settings,
      permissions: { ...permissions, deny: newDeny }
    };
    fs.writeFileSync(settingsJsonPath, JSON.stringify(updated, null, 2) + '\n');
  }

  return { added, existing };
}
