/**
 * Session Manager Library for Claude Code
 * Provides core session CRUD operations for listing, loading, and managing sessions
 *
 * Sessions are stored as markdown files in ~/.claude/sessions/ with format:
 * - YYYY-MM-DD-session.tmp (old format)
 * - YYYY-MM-DD-<short-id>-session.tmp (new format)
 */

import fs from 'fs';
import path from 'path';
import { getSessionsDir, readFile, log } from './utils';

// Session filename pattern
const SESSION_FILENAME_REGEX = /^(\d{4}-\d{2}-\d{2})(?:-([a-z0-9]{8,}))?-session\.tmp$/;

export interface SessionFilename {
  filename: string;
  shortId: string;
  date: string;
  datetime: Date;
}

/**
 * Parse session filename to extract metadata
 */
export function parseSessionFilename(filename: string): SessionFilename | null {
  const match = filename.match(SESSION_FILENAME_REGEX);
  if (!match) return null;

  const dateStr = match[1];
  const [year, month, day] = dateStr.split('-').map(Number);
  if (month < 1 || month > 12 || day < 1 || day > 31) return null;
  const d = new Date(year, month - 1, day);
  if (d.getMonth() !== month - 1 || d.getDate() !== day) return null;

  const shortId = match[2] || 'no-id';

  return {
    filename,
    shortId,
    date: dateStr,
    datetime: new Date(year, month - 1, day)
  };
}

/**
 * Get the full path to a session file
 */
export function getSessionPath(filename: string): string {
  return path.join(getSessionsDir(), filename);
}

/**
 * Read and parse session markdown content
 */
export function getSessionContent(sessionPath: string): string | null {
  return readFile(sessionPath);
}

export interface SessionMetadata {
  title: string | null;
  date: string | null;
  started: string | null;
  lastUpdated: string | null;
  completed: string[];
  inProgress: string[];
  notes: string;
  context: string;
}

/**
 * Parse session metadata from markdown content
 */
export function parseSessionMetadata(content: string | null): SessionMetadata {
  const metadata: SessionMetadata = {
    title: null,
    date: null,
    started: null,
    lastUpdated: null,
    completed: [],
    inProgress: [],
    notes: '',
    context: ''
  };

  if (!content) return metadata;

  const titleMatch = content.match(/^#\s+(.+)$/m);
  if (titleMatch) metadata.title = titleMatch[1].trim();

  const dateMatch = content.match(/\*\*Date:\*\*\s*(\d{4}-\d{2}-\d{2})/);
  if (dateMatch) metadata.date = dateMatch[1];

  const startedMatch = content.match(/\*\*Started:\*\*\s*([\d:]+)/);
  if (startedMatch) metadata.started = startedMatch[1];

  const updatedMatch = content.match(/\*\*Last Updated:\*\*\s*([\d:]+)/);
  if (updatedMatch) metadata.lastUpdated = updatedMatch[1];

  const completedSection = content.match(/### Completed\s*\n([\s\S]*?)(?=###|\n\n|$)/);
  if (completedSection) {
    const items = completedSection[1].match(/- \[x\]\s*(.+)/g);
    if (items) {
      metadata.completed = items.map(item => item.replace(/- \[x\]\s*/, '').trim());
    }
  }

  const progressSection = content.match(/### In Progress\s*\n([\s\S]*?)(?=###|\n\n|$)/);
  if (progressSection) {
    const items = progressSection[1].match(/- \[ \]\s*(.+)/g);
    if (items) {
      metadata.inProgress = items.map(item => item.replace(/- \[ \]\s*/, '').trim());
    }
  }

  const notesSection = content.match(/### Notes for Next Session\s*\n([\s\S]*?)(?=###|\n\n|$)/);
  if (notesSection) metadata.notes = notesSection[1].trim();

  const contextSection = content.match(/### Context to Load\s*\n```\n([\s\S]*?)```/);
  if (contextSection) metadata.context = contextSection[1].trim();

  return metadata;
}

export interface SessionStats {
  totalItems: number;
  completedItems: number;
  inProgressItems: number;
  lineCount: number;
  hasNotes: boolean;
  hasContext: boolean;
}

/**
 * Calculate statistics for a session
 */
export function getSessionStats(sessionPathOrContent: string): SessionStats {
  const looksLikePath = typeof sessionPathOrContent === 'string' &&
    !sessionPathOrContent.includes('\n') &&
    sessionPathOrContent.endsWith('.tmp') &&
    (sessionPathOrContent.startsWith('/') || /^[A-Za-z]:[/\\]/.test(sessionPathOrContent));
  const content = looksLikePath
    ? getSessionContent(sessionPathOrContent)
    : sessionPathOrContent;

  const metadata = parseSessionMetadata(content);

  return {
    totalItems: metadata.completed.length + metadata.inProgress.length,
    completedItems: metadata.completed.length,
    inProgressItems: metadata.inProgress.length,
    lineCount: content ? content.split('\n').length : 0,
    hasNotes: !!metadata.notes,
    hasContext: !!metadata.context
  };
}

export interface SessionListItem extends SessionFilename {
  sessionPath: string;
  hasContent: boolean;
  size: number;
  modifiedTime: Date;
  createdTime: Date;
}

export interface GetAllSessionsOptions {
  limit?: number;
  offset?: number;
  date?: string | null;
  search?: string | null;
}

export interface SessionListResult {
  sessions: SessionListItem[];
  total: number;
  offset: number;
  limit: number;
  hasMore: boolean;
}

/**
 * Get all sessions with optional filtering and pagination
 */
export function getAllSessions(options: GetAllSessionsOptions = {}): SessionListResult {
  const {
    limit: rawLimit = 50,
    offset: rawOffset = 0,
    date = null,
    search = null
  } = options;

  const offsetNum = Number(rawOffset);
  const offset = Number.isNaN(offsetNum) ? 0 : Math.max(0, Math.floor(offsetNum));
  const limitNum = Number(rawLimit);
  const limit = Number.isNaN(limitNum) ? 50 : Math.max(1, Math.floor(limitNum));

  const sessionsDir = getSessionsDir();

  if (!fs.existsSync(sessionsDir)) {
    return { sessions: [], total: 0, offset, limit, hasMore: false };
  }

  const entries = fs.readdirSync(sessionsDir, { withFileTypes: true });
  const sessions: SessionListItem[] = [];

  for (const entry of entries) {
    if (!entry.isFile() || !entry.name.endsWith('.tmp')) continue;

    const filename = entry.name;
    const metadata = parseSessionFilename(filename);
    if (!metadata) continue;

    if (date && metadata.date !== date) continue;
    if (search && !metadata.shortId.includes(search)) continue;

    const sessionPath = path.join(sessionsDir, filename);

    let stats: fs.Stats;
    try {
      stats = fs.statSync(sessionPath);
    } catch {
      continue;
    }

    sessions.push({
      ...metadata,
      sessionPath,
      hasContent: stats.size > 0,
      size: stats.size,
      modifiedTime: stats.mtime,
      createdTime: stats.birthtime || stats.ctime
    });
  }

  sessions.sort((a, b) => b.modifiedTime.getTime() - a.modifiedTime.getTime());
  const paginatedSessions = sessions.slice(offset, offset + limit);

  return {
    sessions: paginatedSessions,
    total: sessions.length,
    offset,
    limit,
    hasMore: offset + limit < sessions.length
  };
}

export interface SessionDetail extends SessionFilename {
  sessionPath: string;
  size: number;
  modifiedTime: Date;
  createdTime: Date;
  content?: string | null;
  metadata?: SessionMetadata;
  stats?: SessionStats;
}

/**
 * Get a single session by ID
 */
export function getSessionById(sessionId: string, includeContent = false): SessionDetail | null {
  const sessionsDir = getSessionsDir();

  if (!fs.existsSync(sessionsDir)) {
    return null;
  }

  const entries = fs.readdirSync(sessionsDir, { withFileTypes: true });

  for (const entry of entries) {
    if (!entry.isFile() || !entry.name.endsWith('.tmp')) continue;

    const filename = entry.name;
    const metadata = parseSessionFilename(filename);
    if (!metadata) continue;

    const shortIdMatch = sessionId.length > 0 && metadata.shortId !== 'no-id' && metadata.shortId.startsWith(sessionId);
    const filenameMatch = filename === sessionId || filename === `${sessionId}.tmp`;
    const noIdMatch = metadata.shortId === 'no-id' && filename === `${sessionId}-session.tmp`;

    if (!shortIdMatch && !filenameMatch && !noIdMatch) continue;

    const sessionPath = path.join(sessionsDir, filename);
    let stats: fs.Stats;
    try {
      stats = fs.statSync(sessionPath);
    } catch {
      return null;
    }

    const session: SessionDetail = {
      ...metadata,
      sessionPath,
      size: stats.size,
      modifiedTime: stats.mtime,
      createdTime: stats.birthtime || stats.ctime
    };

    if (includeContent) {
      session.content = getSessionContent(sessionPath);
      session.metadata = parseSessionMetadata(session.content ?? null);
      session.stats = getSessionStats(session.content || '');
    }

    return session;
  }

  return null;
}

/**
 * Get session title from content
 */
export function getSessionTitle(sessionPath: string): string {
  const content = getSessionContent(sessionPath);
  const metadata = parseSessionMetadata(content);
  return metadata.title || 'Untitled Session';
}

/**
 * Format session size in human-readable format
 */
export function getSessionSize(sessionPath: string): string {
  let stats: fs.Stats;
  try {
    stats = fs.statSync(sessionPath);
  } catch {
    return '0 B';
  }
  const size = stats.size;

  if (size < 1024) return `${size} B`;
  if (size < 1024 * 1024) return `${(size / 1024).toFixed(1)} KB`;
  return `${(size / (1024 * 1024)).toFixed(1)} MB`;
}

/**
 * Write session content to file
 */
export function writeSessionContent(sessionPath: string, content: string): boolean {
  try {
    fs.writeFileSync(sessionPath, content, 'utf8');
    return true;
  } catch (err: unknown) {
    log(`[SessionManager] Error writing session: ${(err as Error).message}`);
    return false;
  }
}

/**
 * Append content to a session
 */
export function appendSessionContent(sessionPath: string, content: string): boolean {
  try {
    fs.appendFileSync(sessionPath, content, 'utf8');
    return true;
  } catch (err: unknown) {
    log(`[SessionManager] Error appending to session: ${(err as Error).message}`);
    return false;
  }
}

/**
 * Delete a session file
 */
export function deleteSession(sessionPath: string): boolean {
  try {
    if (fs.existsSync(sessionPath)) {
      fs.unlinkSync(sessionPath);
      return true;
    }
    return false;
  } catch (err: unknown) {
    log(`[SessionManager] Error deleting session: ${(err as Error).message}`);
    return false;
  }
}

/**
 * Check if a session exists
 */
export function sessionExists(sessionPath: string): boolean {
  try {
    return fs.statSync(sessionPath).isFile();
  } catch {
    return false;
  }
}
