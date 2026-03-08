/**
 * ECC manifest tracking: records what ECC installed for diffing and updates.
 * Tracks ownership so we can distinguish ECC-managed files from user-custom files.
 */

import fs from 'fs';
import path from 'path';

export interface EccManifest {
  version: string;
  installedAt: string;
  updatedAt: string;
  languages: string[];
  artifacts: {
    agents: string[];
    commands: string[];
    skills: string[];
    rules: Record<string, string[]>;
    hookDescriptions: string[];
  };
}

export interface ManifestDiff {
  added: string[];
  updated: string[];
  removed: string[];
}

const MANIFEST_FILENAME = '.ecc-manifest.json';

/**
 * Read an existing manifest from a directory.
 * Returns null if not found or corrupted.
 */
export function readManifest(dir: string): EccManifest | null {
  const manifestPath = path.join(dir, MANIFEST_FILENAME);
  try {
    if (!fs.existsSync(manifestPath)) return null;
    const raw = fs.readFileSync(manifestPath, 'utf8');
    const parsed = JSON.parse(raw);
    // Basic validation
    if (!parsed.version || !parsed.artifacts) return null;
    return parsed as EccManifest;
  } catch {
    return null;
  }
}

/**
 * Write a manifest to a directory (immutable — creates new object).
 */
export function writeManifest(dir: string, manifest: EccManifest): void {
  const manifestPath = path.join(dir, MANIFEST_FILENAME);
  fs.mkdirSync(dir, { recursive: true });
  fs.writeFileSync(manifestPath, JSON.stringify(manifest, null, 2) + '\n', 'utf8');
}

/**
 * Create a fresh manifest for a new installation.
 */
export function createManifest(
  version: string,
  languages: string[],
  artifacts: EccManifest['artifacts'],
): EccManifest {
  const now = new Date().toISOString();
  return {
    version,
    installedAt: now,
    updatedAt: now,
    languages: [...languages],
    artifacts: {
      agents: [...artifacts.agents],
      commands: [...artifacts.commands],
      skills: [...artifacts.skills],
      rules: Object.fromEntries(
        Object.entries(artifacts.rules).map(([k, v]) => [k, [...v]]),
      ),
      hookDescriptions: [...artifacts.hookDescriptions],
    },
  };
}

/**
 * Update an existing manifest with new data (returns new object, does not mutate).
 */
export function updateManifest(
  existing: EccManifest,
  version: string,
  languages: string[],
  artifacts: EccManifest['artifacts'],
): EccManifest {
  return {
    ...existing,
    version,
    updatedAt: new Date().toISOString(),
    languages: [...new Set([...existing.languages, ...languages])],
    artifacts: {
      agents: [...artifacts.agents],
      commands: [...artifacts.commands],
      skills: [...artifacts.skills],
      rules: Object.fromEntries(
        Object.entries(artifacts.rules).map(([k, v]) => [k, [...v]]),
      ),
      hookDescriptions: [...artifacts.hookDescriptions],
    },
  };
}

/**
 * Check if a specific artifact is managed by ECC.
 */
export function isEccManaged(
  manifest: EccManifest | null,
  artifactType: 'agents' | 'commands' | 'skills',
  filename: string,
): boolean {
  if (!manifest) return false;
  return manifest.artifacts[artifactType].includes(filename);
}

/**
 * Check if a rule file is managed by ECC.
 */
export function isEccManagedRule(
  manifest: EccManifest | null,
  group: string,
  filename: string,
): boolean {
  if (!manifest) return false;
  const groupRules = manifest.artifacts.rules[group];
  if (!groupRules) return false;
  return groupRules.includes(filename);
}

/**
 * Diff two lists of filenames to compute what changed.
 */
export function diffFileList(existing: string[], incoming: string[]): ManifestDiff {
  const existingSet = new Set(existing);
  const incomingSet = new Set(incoming);

  return {
    added: incoming.filter(f => !existingSet.has(f)),
    updated: incoming.filter(f => existingSet.has(f)),
    removed: existing.filter(f => !incomingSet.has(f)),
  };
}

/**
 * Get the manifest filename (for gitignore entries etc.).
 */
export function getManifestFilename(): string {
  return MANIFEST_FILENAME;
}
