/**
 * Project type and framework detection
 *
 * Cross-platform (Windows, macOS, Linux) project type detection
 * by inspecting files in the working directory.
 */

import fs from 'fs';
import path from 'path';

interface LanguageRule {
  type: string;
  markers: string[];
  extensions: string[];
}

/**
 * Language detection rules.
 */
export const LANGUAGE_RULES: readonly LanguageRule[] = [
  { type: 'python', markers: ['requirements.txt', 'pyproject.toml', 'setup.py', 'setup.cfg', 'Pipfile', 'poetry.lock'], extensions: ['.py'] },
  { type: 'typescript', markers: ['tsconfig.json', 'tsconfig.build.json'], extensions: ['.ts', '.tsx'] },
  { type: 'javascript', markers: ['package.json', 'jsconfig.json'], extensions: ['.js', '.jsx', '.mjs'] },
  { type: 'golang', markers: ['go.mod', 'go.sum'], extensions: ['.go'] },
  { type: 'rust', markers: ['Cargo.toml', 'Cargo.lock'], extensions: ['.rs'] },
  { type: 'ruby', markers: ['Gemfile', 'Gemfile.lock', 'Rakefile'], extensions: ['.rb'] },
  { type: 'java', markers: ['pom.xml', 'build.gradle', 'build.gradle.kts'], extensions: ['.java'] },
  { type: 'csharp', markers: [], extensions: ['.cs', '.csproj', '.sln'] },
  { type: 'swift', markers: ['Package.swift'], extensions: ['.swift'] },
  { type: 'kotlin', markers: [], extensions: ['.kt', '.kts'] },
  { type: 'elixir', markers: ['mix.exs'], extensions: ['.ex', '.exs'] },
  { type: 'php', markers: ['composer.json', 'composer.lock'], extensions: ['.php'] }
];

interface FrameworkRule {
  framework: string;
  language: string;
  markers: string[];
  packageKeys: string[];
}

/**
 * Framework detection rules.
 */
export const FRAMEWORK_RULES: readonly FrameworkRule[] = [
  // Python frameworks
  { framework: 'django', language: 'python', markers: ['manage.py'], packageKeys: ['django'] },
  { framework: 'fastapi', language: 'python', markers: [], packageKeys: ['fastapi'] },
  { framework: 'flask', language: 'python', markers: [], packageKeys: ['flask'] },

  // JavaScript/TypeScript frameworks
  { framework: 'nextjs', language: 'typescript', markers: ['next.config.js', 'next.config.mjs', 'next.config.ts'], packageKeys: ['next'] },
  { framework: 'react', language: 'typescript', markers: [], packageKeys: ['react'] },
  { framework: 'vue', language: 'typescript', markers: ['vue.config.js'], packageKeys: ['vue'] },
  { framework: 'angular', language: 'typescript', markers: ['angular.json'], packageKeys: ['@angular/core'] },
  { framework: 'svelte', language: 'typescript', markers: ['svelte.config.js'], packageKeys: ['svelte'] },
  { framework: 'express', language: 'javascript', markers: [], packageKeys: ['express'] },
  { framework: 'nestjs', language: 'typescript', markers: ['nest-cli.json'], packageKeys: ['@nestjs/core'] },
  { framework: 'remix', language: 'typescript', markers: [], packageKeys: ['@remix-run/node', '@remix-run/react'] },
  { framework: 'astro', language: 'typescript', markers: ['astro.config.mjs', 'astro.config.ts'], packageKeys: ['astro'] },
  { framework: 'nuxt', language: 'typescript', markers: ['nuxt.config.js', 'nuxt.config.ts'], packageKeys: ['nuxt'] },
  { framework: 'electron', language: 'typescript', markers: [], packageKeys: ['electron'] },

  // Ruby frameworks
  { framework: 'rails', language: 'ruby', markers: ['config/routes.rb', 'bin/rails'], packageKeys: [] },

  // Go frameworks
  { framework: 'gin', language: 'golang', markers: [], packageKeys: ['github.com/gin-gonic/gin'] },
  { framework: 'echo', language: 'golang', markers: [], packageKeys: ['github.com/labstack/echo'] },

  // Rust frameworks
  { framework: 'actix', language: 'rust', markers: [], packageKeys: ['actix-web'] },
  { framework: 'axum', language: 'rust', markers: [], packageKeys: ['axum'] },

  // Java frameworks
  { framework: 'spring', language: 'java', markers: [], packageKeys: ['spring-boot', 'org.springframework'] },

  // PHP frameworks
  { framework: 'laravel', language: 'php', markers: ['artisan'], packageKeys: ['laravel/framework'] },
  { framework: 'symfony', language: 'php', markers: ['symfony.lock'], packageKeys: ['symfony/framework-bundle'] },

  // Elixir frameworks
  { framework: 'phoenix', language: 'elixir', markers: [], packageKeys: ['phoenix'] }
];

function fileExists(projectDir: string, filePath: string): boolean {
  try {
    return fs.existsSync(path.join(projectDir, filePath));
  } catch {
    return false;
  }
}

function hasFileWithExtension(projectDir: string, extensions: string[]): boolean {
  try {
    const entries = fs.readdirSync(projectDir, { withFileTypes: true });
    return entries.some(entry => {
      if (!entry.isFile()) return false;
      const ext = path.extname(entry.name);
      return extensions.includes(ext);
    });
  } catch {
    return false;
  }
}

/** Extract all dependency names from package.json (dependencies + devDependencies). */
export function getPackageJsonDeps(projectDir: string): string[] {
  try {
    const pkgPath = path.join(projectDir, 'package.json');
    if (!fs.existsSync(pkgPath)) return [];
    const pkg = JSON.parse(fs.readFileSync(pkgPath, 'utf8')) as {
      dependencies?: Record<string, string>;
      devDependencies?: Record<string, string>;
    };
    return [...Object.keys(pkg.dependencies || {}), ...Object.keys(pkg.devDependencies || {})];
  } catch {
    return [];
  }
}

/** Extract Python dependency names from requirements.txt and pyproject.toml. */
export function getPythonDeps(projectDir: string): string[] {
  const deps: string[] = [];

  // requirements.txt
  try {
    const reqPath = path.join(projectDir, 'requirements.txt');
    if (fs.existsSync(reqPath)) {
      const content = fs.readFileSync(reqPath, 'utf8');
      content.split('\n').forEach(line => {
        const trimmed = line.trim();
        if (trimmed && !trimmed.startsWith('#') && !trimmed.startsWith('-')) {
          const name = trimmed
            .split(/[>=<![;]/)[0]
            .trim()
            .toLowerCase();
          if (name) deps.push(name);
        }
      });
    }
  } catch {
    /* ignore */
  }

  // pyproject.toml
  try {
    const tomlPath = path.join(projectDir, 'pyproject.toml');
    if (fs.existsSync(tomlPath)) {
      const content = fs.readFileSync(tomlPath, 'utf8');
      const depMatches = content.match(/dependencies\s*=\s*\[([\s\S]*?)\]/);
      if (depMatches) {
        const block = depMatches[1];
        block.match(/"([^"]+)"/g)?.forEach(m => {
          const name = m
            .replace(/"/g, '')
            .split(/[>=<![;]/)[0]
            .trim()
            .toLowerCase();
          if (name) deps.push(name);
        });
      }
    }
  } catch {
    /* ignore */
  }

  return deps;
}

/** Extract Go module dependency paths from go.mod require block. */
export function getGoDeps(projectDir: string): string[] {
  try {
    const modPath = path.join(projectDir, 'go.mod');
    if (!fs.existsSync(modPath)) return [];
    const content = fs.readFileSync(modPath, 'utf8');
    const deps: string[] = [];
    const requireBlock = content.match(/require\s*\(([\s\S]*?)\)/);
    if (requireBlock) {
      requireBlock[1].split('\n').forEach(line => {
        const trimmed = line.trim();
        if (trimmed && !trimmed.startsWith('//')) {
          const parts = trimmed.split(/\s+/);
          if (parts[0]) deps.push(parts[0]);
        }
      });
    }
    return deps;
  } catch {
    return [];
  }
}

/** Extract Rust crate names from Cargo.toml [dependencies] sections. */
export function getRustDeps(projectDir: string): string[] {
  try {
    const cargoPath = path.join(projectDir, 'Cargo.toml');
    if (!fs.existsSync(cargoPath)) return [];
    const content = fs.readFileSync(cargoPath, 'utf8');
    const deps: string[] = [];
    const sections = content.match(/\[(dev-)?dependencies\]([\s\S]*?)(?=\n\[|$)/g);
    if (sections) {
      sections.forEach(section => {
        section.split('\n').forEach(line => {
          const match = line.match(/^([a-zA-Z0-9_-]+)\s*=/);
          if (match && !line.startsWith('[')) {
            deps.push(match[1]);
          }
        });
      });
    }
    return deps;
  } catch {
    return [];
  }
}

/** Extract PHP dependency names from composer.json (require + require-dev). */
export function getComposerDeps(projectDir: string): string[] {
  try {
    const composerPath = path.join(projectDir, 'composer.json');
    if (!fs.existsSync(composerPath)) return [];
    const composer = JSON.parse(fs.readFileSync(composerPath, 'utf8')) as {
      require?: Record<string, string>;
      'require-dev'?: Record<string, string>;
    };
    return [...Object.keys(composer.require || {}), ...Object.keys(composer['require-dev'] || {})];
  } catch {
    return [];
  }
}

/** Extract Elixir dependency names from mix.exs deps block. */
export function getElixirDeps(projectDir: string): string[] {
  try {
    const mixPath = path.join(projectDir, 'mix.exs');
    if (!fs.existsSync(mixPath)) return [];
    const content = fs.readFileSync(mixPath, 'utf8');
    const deps: string[] = [];
    const matches = content.match(/\{:(\w+)/g);
    if (matches) {
      matches.forEach(m => deps.push(m.replace('{:', '')));
    }
    return deps;
  } catch {
    return [];
  }
}

/** Result of project type detection — detected languages, frameworks, and primary type. */
export interface ProjectType {
  languages: string[];
  frameworks: string[];
  primary: string;
  projectDir: string;
}

/**
 * Detect project languages and frameworks
 */
export function detectProjectType(projectDir?: string): ProjectType {
  const dir = projectDir || process.cwd();
  const languages: string[] = [];
  const frameworks: string[] = [];

  // Step 1: Detect languages
  for (const rule of LANGUAGE_RULES) {
    const hasMarker = rule.markers.some(m => fileExists(dir, m));
    const hasExt = rule.extensions.length > 0 && hasFileWithExtension(dir, rule.extensions);

    if (hasMarker || hasExt) {
      languages.push(rule.type);
    }
  }

  // Deduplicate: if both typescript and javascript detected, keep typescript
  if (languages.includes('typescript') && languages.includes('javascript')) {
    const idx = languages.indexOf('javascript');
    if (idx !== -1) languages.splice(idx, 1);
  }

  // Step 2: Detect frameworks based on markers and dependencies
  const npmDeps = getPackageJsonDeps(dir);
  const pyDeps = getPythonDeps(dir);
  const goDeps = getGoDeps(dir);
  const rustDeps = getRustDeps(dir);
  const composerDeps = getComposerDeps(dir);
  const elixirDeps = getElixirDeps(dir);

  for (const rule of FRAMEWORK_RULES) {
    const hasMarker = rule.markers.some(m => fileExists(dir, m));

    let hasDep = false;
    if (rule.packageKeys.length > 0) {
      let depList: string[] = [];
      switch (rule.language) {
        case 'python':
          depList = pyDeps;
          break;
        case 'typescript':
        case 'javascript':
          depList = npmDeps;
          break;
        case 'golang':
          depList = goDeps;
          break;
        case 'rust':
          depList = rustDeps;
          break;
        case 'php':
          depList = composerDeps;
          break;
        case 'elixir':
          depList = elixirDeps;
          break;
      }
      hasDep = rule.packageKeys.some(key => depList.some(dep => dep.toLowerCase().includes(key.toLowerCase())));
    }

    if (hasMarker || hasDep) {
      frameworks.push(rule.framework);
    }
  }

  // Step 3: Determine primary type
  let primary = 'unknown';
  if (frameworks.length > 0) {
    primary = frameworks[0];
  } else if (languages.length > 0) {
    primary = languages[0];
  }

  // Determine if fullstack
  const frontendSignals = ['react', 'vue', 'angular', 'svelte', 'nextjs', 'nuxt', 'astro', 'remix'];
  const backendSignals = ['django', 'fastapi', 'flask', 'express', 'nestjs', 'rails', 'spring', 'laravel', 'phoenix', 'gin', 'echo', 'actix', 'axum'];
  const hasFrontend = frameworks.some(f => frontendSignals.includes(f));
  const hasBackend = frameworks.some(f => backendSignals.includes(f));

  if (hasFrontend && hasBackend) {
    primary = 'fullstack';
  }

  return { languages, frameworks, primary, projectDir: dir };
}
