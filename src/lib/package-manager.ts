/**
 * Package Manager Detection and Selection
 * Automatically detects the preferred package manager or lets user choose
 *
 * Supports: npm, pnpm, yarn, bun
 */

import fs from 'fs';
import path from 'path';
import { commandExists, getClaudeDir, readFile, writeFile } from './utils';

/** Configuration for a package manager — commands for install, run, exec, test, build, and dev. */
export interface PackageManagerConfig {
  name: string;
  lockFile: string;
  installCmd: string;
  runCmd: string;
  execCmd: string;
  testCmd: string;
  buildCmd: string;
  devCmd: string;
}

/** Supported package managers with their command configurations. */
export const PACKAGE_MANAGERS: Record<string, PackageManagerConfig> = {
  npm: {
    name: 'npm',
    lockFile: 'package-lock.json',
    installCmd: 'npm install',
    runCmd: 'npm run',
    execCmd: 'npx',
    testCmd: 'npm test',
    buildCmd: 'npm run build',
    devCmd: 'npm run dev'
  },
  pnpm: {
    name: 'pnpm',
    lockFile: 'pnpm-lock.yaml',
    installCmd: 'pnpm install',
    runCmd: 'pnpm',
    execCmd: 'pnpm dlx',
    testCmd: 'pnpm test',
    buildCmd: 'pnpm build',
    devCmd: 'pnpm dev'
  },
  yarn: {
    name: 'yarn',
    lockFile: 'yarn.lock',
    installCmd: 'yarn',
    runCmd: 'yarn',
    execCmd: 'yarn dlx',
    testCmd: 'yarn test',
    buildCmd: 'yarn build',
    devCmd: 'yarn dev'
  },
  bun: {
    name: 'bun',
    lockFile: 'bun.lockb',
    installCmd: 'bun install',
    runCmd: 'bun run',
    execCmd: 'bunx',
    testCmd: 'bun test',
    buildCmd: 'bun run build',
    devCmd: 'bun run dev'
  }
};

/** Priority order for detecting package managers from lock files. */
export const DETECTION_PRIORITY = ['pnpm', 'bun', 'yarn', 'npm'] as const;

function getConfigPath(): string {
  return path.join(getClaudeDir(), 'package-manager.json');
}

interface PackageManagerSavedConfig {
  packageManager?: string;
  setAt?: string;
}

/**
 * Load saved package manager configuration
 */
function loadConfig(): PackageManagerSavedConfig | null {
  const configPath = getConfigPath();
  const content = readFile(configPath);

  if (content) {
    try {
      return JSON.parse(content) as PackageManagerSavedConfig;
    } catch {
      return null;
    }
  }
  return null;
}

/**
 * Save package manager configuration
 */
function saveConfig(config: PackageManagerSavedConfig): void {
  const configPath = getConfigPath();
  writeFile(configPath, JSON.stringify(config, null, 2));
}

/**
 * Detect package manager from lock file in project directory
 */
export function detectFromLockFile(projectDir: string = process.cwd()): string | null {
  for (const pmName of DETECTION_PRIORITY) {
    const pm = PACKAGE_MANAGERS[pmName];
    const lockFilePath = path.join(projectDir, pm.lockFile);

    if (fs.existsSync(lockFilePath)) {
      return pmName;
    }
  }
  return null;
}

/**
 * Detect package manager from package.json packageManager field
 */
export function detectFromPackageJson(projectDir: string = process.cwd()): string | null {
  const packageJsonPath = path.join(projectDir, 'package.json');
  const content = readFile(packageJsonPath);

  if (content) {
    try {
      const pkg = JSON.parse(content) as { packageManager?: string };
      if (pkg.packageManager) {
        const pmName = pkg.packageManager.split('@')[0];
        if (PACKAGE_MANAGERS[pmName]) {
          return pmName;
        }
      }
    } catch {
      // Invalid package.json
    }
  }
  return null;
}

/**
 * Get available package managers (installed on system)
 *
 * WARNING: This spawns child processes for each package manager.
 * Do NOT call during session startup hooks.
 */
export function getAvailablePackageManagers(): string[] {
  const available: string[] = [];

  for (const pmName of Object.keys(PACKAGE_MANAGERS)) {
    if (commandExists(pmName)) {
      available.push(pmName);
    }
  }

  return available;
}

/** Detected or configured package manager with its config and detection source. */
export interface PackageManagerResult {
  name: string;
  config: PackageManagerConfig;
  source: string;
}

/** Options for package manager detection — optional project directory override. */
export interface GetPackageManagerOptions {
  projectDir?: string;
}

/**
 * Get the package manager to use for current project
 *
 * Detection priority:
 * 1. Environment variable CLAUDE_PACKAGE_MANAGER
 * 2. Project-specific config (in .claude/package-manager.json)
 * 3. package.json packageManager field
 * 4. Lock file detection
 * 5. Global user preference (in ~/.claude/package-manager.json)
 * 6. Default to npm
 */
export function getPackageManager(options: GetPackageManagerOptions = {}): PackageManagerResult {
  const { projectDir = process.cwd() } = options;

  // 1. Check environment variable
  const envPm = process.env.CLAUDE_PACKAGE_MANAGER;
  if (envPm && PACKAGE_MANAGERS[envPm]) {
    return { name: envPm, config: PACKAGE_MANAGERS[envPm], source: 'environment' };
  }

  // 2. Check project-specific config
  const projectConfigPath = path.join(projectDir, '.claude', 'package-manager.json');
  const projectConfig = readFile(projectConfigPath);
  if (projectConfig) {
    try {
      const config = JSON.parse(projectConfig) as PackageManagerSavedConfig;
      if (config.packageManager && PACKAGE_MANAGERS[config.packageManager]) {
        return { name: config.packageManager, config: PACKAGE_MANAGERS[config.packageManager], source: 'project-config' };
      }
    } catch {
      // Invalid config
    }
  }

  // 3. Check package.json packageManager field
  const fromPackageJson = detectFromPackageJson(projectDir);
  if (fromPackageJson) {
    return { name: fromPackageJson, config: PACKAGE_MANAGERS[fromPackageJson], source: 'package.json' };
  }

  // 4. Check lock file
  const fromLockFile = detectFromLockFile(projectDir);
  if (fromLockFile) {
    return { name: fromLockFile, config: PACKAGE_MANAGERS[fromLockFile], source: 'lock-file' };
  }

  // 5. Check global user preference
  const globalConfig = loadConfig();
  if (globalConfig?.packageManager && PACKAGE_MANAGERS[globalConfig.packageManager]) {
    return { name: globalConfig.packageManager, config: PACKAGE_MANAGERS[globalConfig.packageManager], source: 'global-config' };
  }

  // 6. Default to npm
  return { name: 'npm', config: PACKAGE_MANAGERS.npm, source: 'default' };
}

/**
 * Set user's preferred package manager (global)
 */
export function setPreferredPackageManager(pmName: string): PackageManagerSavedConfig {
  if (!PACKAGE_MANAGERS[pmName]) {
    throw new Error(`Unknown package manager: ${pmName}`);
  }

  const config: PackageManagerSavedConfig = loadConfig() || {};
  config.packageManager = pmName;
  config.setAt = new Date().toISOString();

  try {
    saveConfig(config);
  } catch (err: unknown) {
    throw new Error(`Failed to save package manager preference: ${(err as Error).message}`);
  }

  return config;
}

/**
 * Set project's preferred package manager
 */
export function setProjectPackageManager(pmName: string, projectDir: string = process.cwd()): PackageManagerSavedConfig {
  if (!PACKAGE_MANAGERS[pmName]) {
    throw new Error(`Unknown package manager: ${pmName}`);
  }

  const configPath = path.join(projectDir, '.claude', 'package-manager.json');

  const config: PackageManagerSavedConfig = {
    packageManager: pmName,
    setAt: new Date().toISOString()
  };

  try {
    writeFile(configPath, JSON.stringify(config, null, 2));
  } catch (err: unknown) {
    throw new Error(`Failed to save package manager config to ${configPath}: ${(err as Error).message}`);
  }
  return config;
}

// Allowed characters in script/binary names
const SAFE_NAME_REGEX = /^[@a-zA-Z0-9_./-]+$/;

/**
 * Get the command to run a script
 */
export function getRunCommand(script: string, options: GetPackageManagerOptions = {}): string {
  if (!script || typeof script !== 'string') {
    throw new Error('Script name must be a non-empty string');
  }
  if (!SAFE_NAME_REGEX.test(script)) {
    throw new Error(`Script name contains unsafe characters: ${script}`);
  }

  const pm = getPackageManager(options);

  switch (script) {
    case 'install':
      return pm.config.installCmd;
    case 'test':
      return pm.config.testCmd;
    case 'build':
      return pm.config.buildCmd;
    case 'dev':
      return pm.config.devCmd;
    default:
      return `${pm.config.runCmd} ${script}`;
  }
}

// Allowed characters in arguments
const SAFE_ARGS_REGEX = /^[@a-zA-Z0-9\s_./:=,'"*+-]+$/;

/**
 * Get the command to execute a package binary
 */
export function getExecCommand(binary: string, args: string = '', options: GetPackageManagerOptions = {}): string {
  if (!binary || typeof binary !== 'string') {
    throw new Error('Binary name must be a non-empty string');
  }
  if (!SAFE_NAME_REGEX.test(binary)) {
    throw new Error(`Binary name contains unsafe characters: ${binary}`);
  }
  if (args && typeof args === 'string' && !SAFE_ARGS_REGEX.test(args)) {
    throw new Error(`Arguments contain unsafe characters: ${args}`);
  }

  const pm = getPackageManager(options);
  return `${pm.config.execCmd} ${binary}${args ? ' ' + args : ''}`;
}

/**
 * Interactive prompt for package manager selection
 */
export function getSelectionPrompt(): string {
  let message = '[PackageManager] No package manager preference detected.\n';
  message += 'Supported package managers: ' + Object.keys(PACKAGE_MANAGERS).join(', ') + '\n';
  message += '\nTo set your preferred package manager:\n';
  message += '  - Global: Set CLAUDE_PACKAGE_MANAGER environment variable\n';
  message += '  - Or add to ~/.claude/package-manager.json: {"packageManager": "pnpm"}\n';
  message += '  - Or add to package.json: {"packageManager": "pnpm@8"}\n';
  message += '  - Or add a lock file to your project (e.g., pnpm-lock.yaml)\n';

  return message;
}

function escapeRegex(str: string): string {
  return str.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

/**
 * Generate a regex pattern that matches commands for all package managers
 */
export function getCommandPattern(action: string): string {
  const patterns: string[] = [];
  const trimmedAction = action.trim();

  if (trimmedAction === 'dev') {
    patterns.push('npm run dev', 'pnpm( run)? dev', 'yarn dev', 'bun run dev');
  } else if (trimmedAction === 'install') {
    patterns.push('npm install', 'pnpm install', 'yarn( install)?', 'bun install');
  } else if (trimmedAction === 'test') {
    patterns.push('npm test', 'pnpm test', 'yarn test', 'bun test');
  } else if (trimmedAction === 'build') {
    patterns.push('npm run build', 'pnpm( run)? build', 'yarn build', 'bun run build');
  } else {
    const escaped = escapeRegex(trimmedAction);
    patterns.push(`npm run ${escaped}`, `pnpm( run)? ${escaped}`, `yarn ${escaped}`, `bun run ${escaped}`);
  }

  return `(${patterns.join('|')})`;
}
