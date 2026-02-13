# CLI Reference

Complete reference for all gaji CLI commands.

## Commands

### `gaji init`

Initialize a new gaji project.

```bash
gaji init [OPTIONS]
```

**Options:**

| Option | Description |
|--------|-------------|
| `--force` | Overwrite existing files |
| `--skip-examples` | Skip example workflow creation |
| `--migrate` | Migrate existing YAML workflows to TypeScript |
| `-i, --interactive` | Interactive mode with prompts |

**Examples:**

```bash
# Basic initialization
gaji init

# With migration
gaji init --migrate

# Interactive mode
gaji init --interactive

# Force overwrite
gaji init --force
```

**What it does:**

- Creates `workflows/` directory
- Creates `generated/` directory
- Creates `.github/workflows/` directory
- Updates `.gitignore`
- Creates example workflow (unless `--skip-examples`)
- Migrates existing workflows (if `--migrate`)

---

### `gaji dev`

Analyze workflow files and generate types for actions.

```bash
gaji dev [OPTIONS]
```

**Options:**

| Option | Description |
|--------|-------------|
| `-d, --dir <DIR>` | Directory to scan (default: `workflows`) |
| `--watch` | Keep watching for changes after initial scan |

**Examples:**

```bash
# One-time scan
gaji dev

# Watch mode (recommended for development)
gaji dev --watch

# Scan a custom directory
gaji dev --dir src/workflows
```

**What it does:**

- Scans all `.ts` files in `workflows/`
- Extracts `getAction()` calls
- Fetches `action.yml` from GitHub
- Generates TypeScript types in `generated/`
- Updates cache (`.gaji-cache.json`)

**Watch Mode:**

In watch mode, gaji continuously monitors your workflow files. When you add a new action with `getAction()`, types are automatically generated.

---

### `gaji build`

Build TypeScript workflows to YAML.

```bash
gaji build [OPTIONS]
```

**Options:**

| Option | Description |
|--------|-------------|
| `-i, --input <DIR>` | Input directory containing TypeScript workflows (default: `workflows`) |
| `-o, --output <DIR>` | Output directory for YAML files (default: `.github`) |
| `--dry-run` | Preview YAML output without writing files |

**Examples:**

```bash
# Build all workflows
gaji build

# Preview without writing
gaji build --dry-run

# Custom input/output directories
gaji build --input src/workflows --output .github
```

::: tip
Validation and formatting options are configured via `.gaji.toml`, not CLI flags. See [Configuration](../guide/configuration.md).
:::

**What it does:**

- Finds all `.ts` files in `workflows/`
- Executes them with the built-in QuickJS engine (falls back to `npx tsx`)
- Converts output to YAML
- Writes workflows to `.github/workflows/`
- Writes composite actions to `.github/actions/<name>/action.yml`

---

### `gaji add`

Add a GitHub Action and generate types.

```bash
gaji add <ACTION_REF>
```

**Arguments:**

| Argument | Description |
|----------|-------------|
| `<ACTION_REF>` | GitHub action reference (e.g., `actions/checkout@v4`) |

**Examples:**

```bash
# Add common actions
gaji add actions/checkout@v4
gaji add actions/setup-node@v4
gaji add actions/cache@v4

# Add third-party action
gaji add softprops/action-gh-release@v1

# Add action from subdirectory
gaji add docker/setup-buildx-action@v3
```

**What it does:**

- Fetches `action.yml` from GitHub
- Parses inputs, outputs, and metadata
- Generates TypeScript types
- Saves to `generated/`
- Updates cache

---

### `gaji clean`

Clean generated files and optionally clean cache.

```bash
gaji clean [OPTIONS]
```

**Options:**

| Option | Description |
|--------|-------------|
| `--cache` | Also clean cache |

**Examples:**

```bash
# Clean generated files
gaji clean

# Also clean cache
gaji clean --cache
```

**What it does:**

- Removes `generated/` directory
- With `--cache`: also removes `.gaji-cache.json`

Use this when you want to regenerate all types from scratch.

---

### `gaji --version`

Show gaji version.

```bash
gaji --version
```

---

### `gaji --help`

Show help message.

```bash
gaji --help

# Show help for specific command
gaji init --help
gaji dev --help
```

## Common Workflows

### Initial Setup

```bash
# Install
npm install -D gaji

# Initialize
gaji init

# Add actions
gaji add actions/checkout@v4
gaji add actions/setup-node@v4

# Generate types
gaji dev
```

### Development

```bash
# Terminal 1: Watch mode
gaji dev --watch

# Terminal 2: Edit workflows
# (edit workflows/ci.ts)

# Terminal 2: Build
gaji build
```

### Clean Build

```bash
# Remove all generated files
gaji clean

# Regenerate types
gaji dev

# Build workflows
gaji build
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Parsing error |
| 3 | Network error |
| 4 | Validation error |

## Environment Variables

### `GITHUB_TOKEN`

Set a GitHub token for authenticated API requests (increases rate limits):

```bash
export GITHUB_TOKEN=ghp_your_token_here
gaji dev
```

## Configuration File

Commands respect settings in `.gaji.toml`. See [Configuration](../guide/configuration.md) for details.

## Troubleshooting

### "Action not found"

Make sure the action reference is correct:

```bash
# ✅ Correct
gaji add actions/checkout@v4

# ❌ Wrong
gaji add checkout  # Missing owner and version
```

### "Network error"

Check your internet connection. If you're behind a proxy, configure it:

```bash
export HTTP_PROXY=http://proxy.example.com:8080
export HTTPS_PROXY=http://proxy.example.com:8080
```

### "Types not generated"

Make sure you've run `gaji dev` after adding actions:

```bash
gaji add actions/checkout@v4
gaji dev  # Don't forget this!
```

## Next Steps

- Learn about the [TypeScript API](./api.md)
- See [Configuration](../guide/configuration.md)
