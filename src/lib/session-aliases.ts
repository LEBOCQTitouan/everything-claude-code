/**
 * Session Aliases Library for Claude Code
 * Manages session aliases stored in ~/.claude/session-aliases.json
 */

import fs from 'fs';
import path from 'path';
import { getClaudeDir, ensureDir, readFile, log } from './utils';

export function getAliasesPath(): string {
  return path.join(getClaudeDir(), 'session-aliases.json');
}

const ALIAS_VERSION = '1.0';

interface AliasEntry {
  sessionPath: string;
  createdAt: string;
  updatedAt?: string;
  title: string | null;
}

interface AliasMetadata {
  totalCount: number;
  lastUpdated: string;
}

interface AliasesData {
  version: string;
  aliases: Record<string, AliasEntry>;
  metadata: AliasMetadata;
}

function getDefaultAliases(): AliasesData {
  return {
    version: ALIAS_VERSION,
    aliases: {},
    metadata: {
      totalCount: 0,
      lastUpdated: new Date().toISOString()
    }
  };
}

/**
 * Load aliases from file
 */
export function loadAliases(): AliasesData {
  const aliasesPath = getAliasesPath();

  if (!fs.existsSync(aliasesPath)) {
    return getDefaultAliases();
  }

  const content = readFile(aliasesPath);
  if (!content) {
    return getDefaultAliases();
  }

  try {
    const data = JSON.parse(content) as AliasesData;

    if (!data.aliases || typeof data.aliases !== 'object') {
      log('[Aliases] Invalid aliases file structure, resetting');
      return getDefaultAliases();
    }

    if (!data.version) {
      data.version = ALIAS_VERSION;
    }

    if (!data.metadata) {
      data.metadata = {
        totalCount: Object.keys(data.aliases).length,
        lastUpdated: new Date().toISOString()
      };
    }

    return data;
  } catch (err: unknown) {
    log(`[Aliases] Error parsing aliases file: ${(err as Error).message}`);
    return getDefaultAliases();
  }
}

/**
 * Save aliases to file with atomic write
 */
export function saveAliases(aliases: AliasesData): boolean {
  const aliasesPath = getAliasesPath();
  const tempPath = aliasesPath + '.tmp';
  const backupPath = aliasesPath + '.bak';

  try {
    aliases.metadata = {
      totalCount: Object.keys(aliases.aliases).length,
      lastUpdated: new Date().toISOString()
    };

    const content = JSON.stringify(aliases, null, 2);
    ensureDir(path.dirname(aliasesPath));

    if (fs.existsSync(aliasesPath)) {
      fs.copyFileSync(aliasesPath, backupPath);
    }

    fs.writeFileSync(tempPath, content, 'utf8');

    if (process.platform === 'win32' && fs.existsSync(aliasesPath)) {
      fs.unlinkSync(aliasesPath);
    }
    fs.renameSync(tempPath, aliasesPath);

    if (fs.existsSync(backupPath)) {
      fs.unlinkSync(backupPath);
    }

    return true;
  } catch (err: unknown) {
    log(`[Aliases] Error saving aliases: ${(err as Error).message}`);

    if (fs.existsSync(backupPath)) {
      try {
        fs.copyFileSync(backupPath, aliasesPath);
        log('[Aliases] Restored from backup');
      } catch (restoreErr: unknown) {
        log(`[Aliases] Failed to restore backup: ${(restoreErr as Error).message}`);
      }
    }

    try {
      if (fs.existsSync(tempPath)) {
        fs.unlinkSync(tempPath);
      }
    } catch {
      // Non-critical
    }

    return false;
  }
}

export interface ResolvedAlias {
  alias: string;
  sessionPath: string;
  createdAt: string;
  title: string | null;
}

/**
 * Resolve an alias to get session path
 */
export function resolveAlias(alias: string): ResolvedAlias | null {
  if (!alias) return null;

  if (!/^[a-zA-Z0-9_-]+$/.test(alias)) {
    return null;
  }

  const data = loadAliases();
  const aliasData = data.aliases[alias];

  if (!aliasData) {
    return null;
  }

  return {
    alias,
    sessionPath: aliasData.sessionPath,
    createdAt: aliasData.createdAt,
    title: aliasData.title || null
  };
}

export interface SetAliasResult {
  success: boolean;
  error?: string;
  isNew?: boolean;
  alias?: string;
  sessionPath?: string;
  title?: string | null;
}

/**
 * Set or update an alias for a session
 */
export function setAlias(alias: string, sessionPath: string, title: string | null = null): SetAliasResult {
  if (!alias || alias.length === 0) {
    return { success: false, error: 'Alias name cannot be empty' };
  }

  if (!sessionPath || typeof sessionPath !== 'string' || sessionPath.trim().length === 0) {
    return { success: false, error: 'Session path cannot be empty' };
  }

  if (alias.length > 128) {
    return { success: false, error: 'Alias name cannot exceed 128 characters' };
  }

  if (!/^[a-zA-Z0-9_-]+$/.test(alias)) {
    return { success: false, error: 'Alias name must contain only letters, numbers, dashes, and underscores' };
  }

  const reserved = ['list', 'help', 'remove', 'delete', 'create', 'set'];
  if (reserved.includes(alias.toLowerCase())) {
    return { success: false, error: `'${alias}' is a reserved alias name` };
  }

  const data = loadAliases();
  const existing = data.aliases[alias];
  const isNew = !existing;

  data.aliases[alias] = {
    sessionPath,
    createdAt: existing ? existing.createdAt : new Date().toISOString(),
    updatedAt: new Date().toISOString(),
    title: title || null
  };

  if (saveAliases(data)) {
    return {
      success: true,
      isNew,
      alias,
      sessionPath,
      title: data.aliases[alias].title
    };
  }

  return { success: false, error: 'Failed to save alias' };
}

export interface AliasInfo {
  name: string;
  sessionPath: string;
  createdAt: string;
  updatedAt?: string;
  title: string | null;
}

export interface ListAliasesOptions {
  search?: string | null;
  limit?: number | null;
}

/**
 * List all aliases
 */
export function listAliases(options: ListAliasesOptions = {}): AliasInfo[] {
  const { search = null, limit = null } = options;
  const data = loadAliases();

  let aliases: AliasInfo[] = Object.entries(data.aliases).map(([name, info]) => ({
    name,
    sessionPath: info.sessionPath,
    createdAt: info.createdAt,
    updatedAt: info.updatedAt,
    title: info.title
  }));

  aliases.sort((a, b) => (new Date(b.updatedAt || b.createdAt || '').getTime() || 0) - (new Date(a.updatedAt || a.createdAt || '').getTime() || 0));

  if (search) {
    const searchLower = search.toLowerCase();
    aliases = aliases.filter(a => a.name.toLowerCase().includes(searchLower) || (a.title && a.title.toLowerCase().includes(searchLower)));
  }

  if (limit && limit > 0) {
    aliases = aliases.slice(0, limit);
  }

  return aliases;
}

export interface DeleteAliasResult {
  success: boolean;
  error?: string;
  alias?: string;
  deletedSessionPath?: string;
}

/**
 * Delete an alias
 */
export function deleteAlias(alias: string): DeleteAliasResult {
  const data = loadAliases();

  if (!data.aliases[alias]) {
    return { success: false, error: `Alias '${alias}' not found` };
  }

  const deleted = data.aliases[alias];
  delete data.aliases[alias];

  if (saveAliases(data)) {
    return { success: true, alias, deletedSessionPath: deleted.sessionPath };
  }

  return { success: false, error: 'Failed to delete alias' };
}

export interface RenameAliasResult {
  success: boolean;
  error?: string;
  oldAlias?: string;
  newAlias?: string;
  sessionPath?: string;
}

/**
 * Rename an alias
 */
export function renameAlias(oldAlias: string, newAlias: string): RenameAliasResult {
  const data = loadAliases();

  if (!data.aliases[oldAlias]) {
    return { success: false, error: `Alias '${oldAlias}' not found` };
  }

  if (!newAlias || newAlias.length === 0) {
    return { success: false, error: 'New alias name cannot be empty' };
  }

  if (newAlias.length > 128) {
    return { success: false, error: 'New alias name cannot exceed 128 characters' };
  }

  if (!/^[a-zA-Z0-9_-]+$/.test(newAlias)) {
    return { success: false, error: 'New alias name must contain only letters, numbers, dashes, and underscores' };
  }

  const reserved = ['list', 'help', 'remove', 'delete', 'create', 'set'];
  if (reserved.includes(newAlias.toLowerCase())) {
    return { success: false, error: `'${newAlias}' is a reserved alias name` };
  }

  if (data.aliases[newAlias]) {
    return { success: false, error: `Alias '${newAlias}' already exists` };
  }

  const aliasData = data.aliases[oldAlias];
  delete data.aliases[oldAlias];

  aliasData.updatedAt = new Date().toISOString();
  data.aliases[newAlias] = aliasData;

  if (saveAliases(data)) {
    return { success: true, oldAlias, newAlias, sessionPath: aliasData.sessionPath };
  }

  // Rollback
  data.aliases[oldAlias] = aliasData;
  delete data.aliases[newAlias];
  saveAliases(data);
  return { success: false, error: 'Failed to save renamed alias — rolled back to original' };
}

/**
 * Get session path by alias (convenience function)
 */
export function resolveSessionAlias(aliasOrId: string): string {
  const resolved = resolveAlias(aliasOrId);
  if (resolved) {
    return resolved.sessionPath;
  }
  return aliasOrId;
}

export interface UpdateTitleResult {
  success: boolean;
  error?: string;
  alias?: string;
  title?: string | null;
}

/**
 * Update alias title
 */
export function updateAliasTitle(alias: string, title: string | null): UpdateTitleResult {
  if (title !== null && typeof title !== 'string') {
    return { success: false, error: 'Title must be a string or null' };
  }

  const data = loadAliases();

  if (!data.aliases[alias]) {
    return { success: false, error: `Alias '${alias}' not found` };
  }

  data.aliases[alias].title = title || null;
  data.aliases[alias].updatedAt = new Date().toISOString();

  if (saveAliases(data)) {
    return { success: true, alias, title };
  }

  return { success: false, error: 'Failed to update alias title' };
}

export interface SessionAliasInfo {
  name: string;
  createdAt: string;
  title: string | null;
}

/**
 * Get all aliases for a specific session
 */
export function getAliasesForSession(sessionPath: string): SessionAliasInfo[] {
  const data = loadAliases();
  const aliases: SessionAliasInfo[] = [];

  for (const [name, info] of Object.entries(data.aliases)) {
    if (info.sessionPath === sessionPath) {
      aliases.push({
        name,
        createdAt: info.createdAt,
        title: info.title
      });
    }
  }

  return aliases;
}

export interface CleanupResult {
  success?: boolean;
  totalChecked: number;
  removed: number;
  removedAliases: Array<{ name: string; sessionPath: string }>;
  error?: string;
}

/**
 * Clean up aliases for non-existent sessions
 */
export function cleanupAliases(sessionExistsFn: (path: string) => boolean): CleanupResult {
  if (typeof sessionExistsFn !== 'function') {
    return { totalChecked: 0, removed: 0, removedAliases: [], error: 'sessionExists must be a function' };
  }

  const data = loadAliases();
  const removed: Array<{ name: string; sessionPath: string }> = [];

  for (const [name, info] of Object.entries(data.aliases)) {
    if (!sessionExistsFn(info.sessionPath)) {
      removed.push({ name, sessionPath: info.sessionPath });
      delete data.aliases[name];
    }
  }

  if (removed.length > 0 && !saveAliases(data)) {
    log('[Aliases] Failed to save after cleanup');
    return {
      success: false,
      totalChecked: Object.keys(data.aliases).length + removed.length,
      removed: removed.length,
      removedAliases: removed,
      error: 'Failed to save after cleanup'
    };
  }

  return {
    success: true,
    totalChecked: Object.keys(data.aliases).length + removed.length,
    removed: removed.length,
    removedAliases: removed
  };
}
