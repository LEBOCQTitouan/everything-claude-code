/**
 * Shared hook enable/disable controls.
 *
 * Controls:
 * - ECC_HOOK_PROFILE=minimal|standard|strict (default: standard)
 * - ECC_DISABLED_HOOKS=comma,separated,hook,ids
 */

/** Hook execution profile — controls which hooks run. */
export type HookProfile = 'minimal' | 'standard' | 'strict';

/** Set of valid hook profile names. */
export const VALID_PROFILES: ReadonlySet<string> = new Set(['minimal', 'standard', 'strict']);

/** Normalize a hook ID to lowercase trimmed string. */
export function normalizeId(value: unknown): string {
  return String(value || '')
    .trim()
    .toLowerCase();
}

/** Read the active hook profile from ECC_HOOK_PROFILE env var, defaulting to 'standard'. */
export function getHookProfile(): HookProfile {
  const raw = String(process.env.ECC_HOOK_PROFILE || 'standard')
    .trim()
    .toLowerCase();
  return VALID_PROFILES.has(raw) ? (raw as HookProfile) : 'standard';
}

/** Parse ECC_DISABLED_HOOKS env var into a set of disabled hook IDs. */
export function getDisabledHookIds(): Set<string> {
  const raw = String(process.env.ECC_DISABLED_HOOKS || '');
  if (!raw.trim()) return new Set();

  return new Set(
    raw
      .split(',')
      .map(v => normalizeId(v))
      .filter(Boolean)
  );
}

/** Parse a comma-separated or array of profile names into validated profile list. */
export function parseProfiles(rawProfiles?: string | string[] | null, fallback: string[] = ['standard', 'strict']): string[] {
  if (!rawProfiles) return [...fallback];

  if (Array.isArray(rawProfiles)) {
    const parsed = rawProfiles
      .map(v =>
        String(v || '')
          .trim()
          .toLowerCase()
      )
      .filter(v => VALID_PROFILES.has(v));
    return parsed.length > 0 ? parsed : [...fallback];
  }

  const parsed = String(rawProfiles)
    .split(',')
    .map(v => v.trim().toLowerCase())
    .filter(v => VALID_PROFILES.has(v));

  return parsed.length > 0 ? parsed : [...fallback];
}

/** Options for checking if a hook should execute. */
export interface IsHookEnabledOptions {
  profiles?: string | string[];
}

/** Check if a hook is enabled based on its ID, disabled list, and active profile. */
export function isHookEnabled(hookId: string, options: IsHookEnabledOptions = {}): boolean {
  const id = normalizeId(hookId);
  if (!id) return true;

  const disabled = getDisabledHookIds();
  if (disabled.has(id)) {
    return false;
  }

  const profile = getHookProfile();
  const allowedProfiles = parseProfiles(options.profiles);
  return allowedProfiles.includes(profile);
}
