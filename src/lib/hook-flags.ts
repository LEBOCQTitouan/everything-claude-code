/**
 * Shared hook enable/disable controls.
 *
 * Controls:
 * - ECC_HOOK_PROFILE=minimal|standard|strict (default: standard)
 * - ECC_DISABLED_HOOKS=comma,separated,hook,ids
 */

export type HookProfile = 'minimal' | 'standard' | 'strict';

export const VALID_PROFILES: ReadonlySet<string> = new Set(['minimal', 'standard', 'strict']);

export function normalizeId(value: unknown): string {
  return String(value || '').trim().toLowerCase();
}

export function getHookProfile(): HookProfile {
  const raw = String(process.env.ECC_HOOK_PROFILE || 'standard').trim().toLowerCase();
  return VALID_PROFILES.has(raw) ? (raw as HookProfile) : 'standard';
}

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

export function parseProfiles(rawProfiles?: string | string[] | null, fallback: string[] = ['standard', 'strict']): string[] {
  if (!rawProfiles) return [...fallback];

  if (Array.isArray(rawProfiles)) {
    const parsed = rawProfiles
      .map(v => String(v || '').trim().toLowerCase())
      .filter(v => VALID_PROFILES.has(v));
    return parsed.length > 0 ? parsed : [...fallback];
  }

  const parsed = String(rawProfiles)
    .split(',')
    .map(v => v.trim().toLowerCase())
    .filter(v => VALID_PROFILES.has(v));

  return parsed.length > 0 ? parsed : [...fallback];
}

export interface IsHookEnabledOptions {
  profiles?: string | string[];
}

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
