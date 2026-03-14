---
name: config-extraction
description: Extract environment variables, config files, CLI flags, defaults, and sensitivity markers for documentation and runbook generation.
origin: ECC
---

# Config Extraction

Atomic extraction skill for cataloguing all configuration surfaces in a codebase — environment variables, config files, CLI flags, feature flags, and their defaults, types, and sensitivity levels.

## When to Activate

- When documenting environment variables and setup requirements
- Before generating runbooks (need to know all config knobs)
- When auditing configuration for security (sensitive values)
- During onboarding documentation (what needs to be set up?)

## Methodology

### 1. Environment Variable Extraction

Scan source code for environment variable access:

| Language | Pattern |
|----------|---------|
| TypeScript/JS | `process.env.NAME`, `process.env['NAME']` |
| Python | `os.environ['NAME']`, `os.environ.get('NAME')`, `os.getenv('NAME')` |
| Go | `os.Getenv("NAME")`, `os.LookupEnv("NAME")` |
| Rust | `std::env::var("NAME")`, `env::var_os("NAME")` |
| Java | `System.getenv("NAME")`, `System.getProperty("NAME")` |

For each variable, record:

| Field | Source |
|-------|--------|
| Name | The string literal in the env access call |
| Default | Second argument to `get()`/`getenv()`, or fallback in `\|\|`/`??` |
| Required | No default provided, or explicit validation/throw on missing |
| Type | Inferred from usage (parsed as int, used as URL, compared to boolean) |
| Sensitivity | See sensitivity detection below |
| Used in | File:line references |

### 2. Config File Detection

Identify configuration files and their schemas:

| File Pattern | Type |
|-------------|------|
| `.env`, `.env.*` | dotenv |
| `config.*`, `*.config.*` | Application config |
| `docker-compose.yml` | Container orchestration |
| `Dockerfile` | Build config |
| `tsconfig.json`, `pyproject.toml`, `go.mod` | Language tooling |
| `*.yaml`, `*.yml` in config dirs | Structured config |

For each config file:
- List all keys/settings
- Note which are referenced in source code
- Flag unused config keys
- Flag config keys referenced in code but missing from config files

### 3. CLI Flag Extraction

Scan for command-line argument parsing:

| Library | Pattern |
|---------|---------|
| Node.js `yargs`/`commander` | `.option('--name')`, `.argument('<name>')` |
| Python `argparse` | `parser.add_argument('--name')` |
| Go `flag` | `flag.String("name", ...)`, `pflag.StringP(...)` |
| Rust `clap` | `Arg::new("name")`, `#[arg(long)]` |

Record: name, type, default, description (from help text), required status.

### 4. Sensitivity Detection

Classify config values by sensitivity:

| Level | Indicators |
|-------|-----------|
| **SECRET** | Name contains: `KEY`, `SECRET`, `TOKEN`, `PASSWORD`, `CREDENTIAL`, `PRIVATE` |
| **SENSITIVE** | Name contains: `DATABASE_URL`, `REDIS_URL`, `API_URL` (may expose infra) |
| **PUBLIC** | Everything else |

Additional checks:
- Is the value in `.gitignore`?
- Is there a `.env.example` that redacts it?
- Is it loaded from a secrets manager (AWS SSM, Vault, etc.)?

### 5. Default Value Mapping

Build a complete map of what happens with zero configuration:

```
Config: DATABASE_URL
  Default: none (REQUIRED)
  Sensitivity: SENSITIVE
  Used in: src/db/connection.ts:12
  Behaviour without: throws "DATABASE_URL must be set" at startup

Config: LOG_LEVEL
  Default: "info"
  Sensitivity: PUBLIC
  Used in: src/lib/logger.ts:5
  Behaviour without: defaults to info-level logging
```

## Output Format

Structured configuration inventory:

```
# Configuration Surface

## Environment Variables (12 total, 4 required, 3 secret)

| Variable | Required | Default | Type | Sensitivity | Used In |
|----------|----------|---------|------|-------------|---------|
| DATABASE_URL | yes | — | URL | SENSITIVE | src/db/connection.ts:12 |
| API_KEY | yes | — | string | SECRET | src/api/client.ts:8 |
| LOG_LEVEL | no | "info" | enum | PUBLIC | src/lib/logger.ts:5 |
| PORT | no | 3000 | number | PUBLIC | src/server.ts:15 |

## Config Files (3)

| File | Keys | Referenced | Unreferenced |
|------|------|-----------|-------------|
| .env.example | 8 | 7 | 1 (LEGACY_MODE) |
| tsconfig.json | 12 | — | — |

## CLI Flags (5)

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| --port | number | 3000 | Server port |
| --verbose | boolean | false | Enable verbose logging |
```

## Related

- Runbook generation: `skills/runbook-gen/SKILL.md`
- Failure modes: `skills/failure-modes/SKILL.md`
- Doc analysis skill: `skills/doc-analysis/SKILL.md`
- Security review: `agents/security-reviewer.md`
