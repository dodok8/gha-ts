# Configuration

gaji uses TypeScript configuration files for type-safe settings with autocomplete.

## Configuration File

Create `gaji.config.ts` in your project root. `gaji init` generates one automatically.

```typescript
import { defineConfig } from "./generated/index.js";

export default defineConfig({
    workflows: "workflows",
    output: ".github",
    generated: "generated",
    watch: {
        debounce: 300,
        ignore: ["node_modules", ".git", "generated"],
    },
    build: {
        validate: true,
        format: true,
    },
});
```

The `defineConfig` function provides autocomplete and type checking for all fields. At runtime it returns its argument unchanged.

## Configuration Options

### Project Directories

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `workflows` | string | `"workflows"` | Directory containing TypeScript workflows. Used as the default `--input` for `gaji dev` and `gaji build` |
| `output` | string | `".github"` | Base output directory (workflows go to `workflows/`, actions to `actions/`). Used as the default `--output` for `gaji build` |
| `generated` | string | `"generated"` | Directory for generated action types |

**Example:**

```typescript
export default defineConfig({
    workflows: "gha",
    output: ".github",
    generated: "gha-types",
});
```

### `github`

GitHub API settings:

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `token` | string (optional) | None | GitHub personal access token |
| `apiUrl` | string (optional) | `"https://github.com"` | GitHub base URL (for Enterprise) |

**Token Priority:**

1. `GITHUB_TOKEN` environment variable (highest priority)
2. `token` in `gaji.config.local.ts`
3. `token` in `gaji.config.ts`

**Example for GitHub Enterprise:**

```typescript
export default defineConfig({
    github: {
        token: "ghp_your_token_here",
        apiUrl: "https://github.example.com",
    },
});
```

**What you can configure:**

- **GitHub token**: Authenticate API requests (increases rate limits, access private repos)
- **GitHub Enterprise**: Point to your self-hosted GitHub instance
- **Action fetching**: Retrieve `action.yml` from private or enterprise GitHub

**Note:** For security, store tokens in `gaji.config.local.ts` (gitignored) instead of `gaji.config.ts`

### `watch`

File watching settings:

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `debounce` | number | `300` | Debounce delay in milliseconds |
| `ignore` | string[] | `["node_modules", ".git", "generated"]` | Patterns to ignore |

The `ignore` setting is used by both `gaji dev` (watch mode) and `gaji build` commands. Files matching any of these patterns will be excluded from processing. The matching uses simple substring matching - if any pattern appears anywhere in the file path, the file is ignored.

**Example:**

```typescript
export default defineConfig({
    watch: {
        debounce: 500,
        ignore: ["node_modules", ".git", "generated", "dist", "coverage"],
    },
});
```

**Common patterns to ignore:**

- `node_modules` - npm dependencies
- `.git` - Git internals
- `generated` - gaji-generated type files
- `dist` - build output directories
- `coverage` - test coverage reports

### `build`

Build settings:

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `validate` | boolean | `true` | Validate generated YAML |
| `format` | boolean | `true` | Format generated YAML |
| `cacheTtlDays` | number | `30` | Cache TTL in days for action metadata |

**Example:**

```typescript
export default defineConfig({
    build: {
        validate: true,
        format: true,
        cacheTtlDays: 14,
    },
});
```

## Local Configuration

Create `gaji.config.local.ts` for sensitive values like tokens. This file should be gitignored.

```typescript
import { defineConfig } from "./generated/index.js";

export default defineConfig({
    github: {
        token: "ghp_your_token_here",
        apiUrl: "https://github.example.com",
    },
});
```

## TypeScript Configuration

gaji works with standard TypeScript configuration. Make sure your `tsconfig.json` includes the generated types.

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
      "./generated"
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
gaji.config.local.ts
```

**Note:** Do NOT ignore `.github/workflows/` since those are the actual workflow files GitHub Actions uses.

## Backward Compatibility

gaji still reads `.gaji.toml` and `.gaji.local.toml` if `gaji.config.ts` is not found. Existing projects using TOML configuration continue to work without changes.

## Cache

gaji uses a cache file (`.gaji-cache.json`) to avoid re-fetching action definitions. This file is automatically managed and should be gitignored.

To clear the cache, use the `clean` command.

```bash
gaji clean --cache
```

## Next Steps

- Learn about [Migration](./migration.md)
- Check the [CLI Reference](/reference/cli)
