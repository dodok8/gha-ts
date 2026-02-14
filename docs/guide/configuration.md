# Configuration

gaji can be configured using a `.gaji.toml` file in your project root.

## Configuration File

Create `.gaji.toml` in your project root:

```toml
[project]
workflows_dir = "workflows"
output_dir = ".github"
generated_dir = "generated"

[github]
# Optional: GitHub token (can also use GITHUB_TOKEN env var)
token = "ghp_your_token_here"
# For GitHub Enterprise users
api_url = "https://github.example.com"

[watch]
debounce_ms = 300
ignored_patterns = ["node_modules", ".git", "generated"]

[build]
validate = true
format = true
```

## Configuration Options

### `[project]`

Project-level settings:

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `workflows_dir` | string | `"workflows"` | Directory containing TypeScript workflows |
| `output_dir` | string | `".github"` | Base output directory (workflows go to `workflows/`, actions to `actions/`) |
| `generated_dir` | string | `"generated"` | Directory for generated action types |

**Example:**

```toml
[project]
workflows_dir = "gha"
output_dir = ".github"
generated_dir = "gha-types"
```

### `[github]`

GitHub API settings:

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `token` | string (optional) | None | GitHub personal access token |
| `api_url` | string (optional) | `"https://github.com"` | GitHub base URL (for Enterprise) |

**Token Priority:**
1. `GITHUB_TOKEN` environment variable (highest priority)
2. `token` in `.gaji.local.toml`
3. `token` in `.gaji.toml`

**Example for GitHub Enterprise:**

```toml
[github]
token = "ghp_your_token_here"
api_url = "https://github.example.com"
```

**What you can configure:**
- **GitHub token**: Authenticate API requests (increases rate limits, access private repos)
- **GitHub Enterprise**: Point to your self-hosted GitHub instance
- **Action fetching**: Retrieve `action.yml` from private or enterprise GitHub

**Note:** For security, store tokens in `.gaji.local.toml` (gitignored) instead of `.gaji.toml`

### `[watch]`

File watching settings:

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `debounce_ms` | integer | `300` | Debounce delay in milliseconds |
| `ignored_patterns` | array | `["node_modules", ".git", "generated"]` | Patterns to ignore |

The `ignored_patterns` setting is used by both `gaji dev` (watch mode) and `gaji build` commands. Files matching any of these patterns will be excluded from processing. The matching uses simple substring matching - if any pattern appears anywhere in the file path, the file is ignored.

**Example:**

```toml
[watch]
debounce_ms = 500
ignored_patterns = ["node_modules", ".git", "generated", "dist", "coverage"]
```

**Common patterns to ignore:**

- `node_modules` - npm dependencies
- `.git` - Git internals
- `generated` - gaji-generated type files
- `dist` - build output directories
- `coverage` - test coverage reports

### `[build]`

Build settings:

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `validate` | boolean | `true` | Validate generated YAML |
| `format` | boolean | `true` | Format generated YAML |

**Example:**

```toml
[build]
validate = true
format = true
```

## TypeScript Configuration

gaji works with standard TypeScript configuration. Make sure your `tsconfig.json` includes the generated types:

```json
{
  "compilerOptions": {
    "target": "ES2020",
    "module": "commonjs",
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "typeRoots": [
      "./node_modules/@types",
      "./generated"  // Include gaji-generated types
    ]
  },
  "include": ["workflows/**/*"],
  "exclude": ["node_modules", "dist", "generated"]
}
```

## `.gitignore`

Add gaji-generated files to your `.gitignore`:

```gitignore
# gaji
generated/
.gaji-cache.json
```

**Note:** Do NOT ignore `.github/workflows/` since those are the actual workflow files GitHub Actions uses.

## Cache

gaji uses a cache file (`.gaji-cache.json`) to avoid re-fetching action definitions. This file is automatically managed and should be gitignored.

To clear the cache:

```bash
gaji clean --cache
```

## Environment Variables

### `GITHUB_TOKEN`

Set a GitHub token for authenticated requests (increases rate limits):

```bash
export GITHUB_TOKEN=ghp_your_token_here
gaji dev
```

## Next Steps

- Learn about [Migration](./migration.md)
- Check the [CLI Reference](/reference/cli)
