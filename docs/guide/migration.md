# Migration

This guide helps you migrate existing YAML workflows to TypeScript with gaji.

## Automatic Migration

gaji can automatically convert existing YAML workflows to TypeScript.

```bash
gaji init --migrate
```

The process follows this order:

1. Detect existing YAML workflows in `.github/workflows/`
2. Convert them to TypeScript in `workflows/`
3. Backup original YAML files (`.yml.backup`)
4. Generate types for all actions used

## Action Migration

gaji also migrates existing local actions (`.github/actions/*/action.yml`) to TypeScript.

```bash
gaji init --migrate
```

This detects both workflows and actions automatically. Actions are converted to `Action`, `NodeAction`, or `DockerAction` classes depending on the `runs.using` field.

### Composite Action

**Before** (`.github/actions/setup-env/action.yml`):

```yaml
name: Setup Environment
description: Setup Node.js and install dependencies
inputs:
  node-version:
    description: Node.js version to use
    required: false
    default: "20"
outputs:
  cache-hit:
    description: Whether cache was hit
    value: ${{ steps.cache.outputs.cache-hit }}
runs:
  using: composite
  steps:
    - uses: actions/checkout@v5
    - name: Install dependencies
      run: npm ci
      shell: bash
    - name: Cache node_modules
      id: cache
      uses: actions/cache@v4
      with:
        path: node_modules
        key: ${{ runner.os }}-node-${{ hashFiles('**/package-lock.json') }}
```

**After** (`workflows/action-setup-env.ts`):

```typescript
import { getAction, Action } from "../generated/index.js";

const checkout = getAction("actions/checkout@v5");
const cache = getAction("actions/cache@v4");

const action = new Action({
    name: "Setup Environment",
    description: "Setup Node.js and install dependencies",
    inputs: {
        "node-version": {
            description: "Node.js version to use",
            required: false,
            default: "20",
        },
    },
    outputs: {
        "cache-hit": {
            description: "Whether cache was hit",
            value: "${{ steps.cache.outputs.cache-hit }}",
        },
    },
});

action
    .steps(s => s
        .add(checkout({}))
        .add({
            name: "Install dependencies",
            run: "npm ci",
            shell: "bash",
        })
        .add(cache({
            id: "cache",
            name: "Cache node_modules",
            with: {
                path: "node_modules",
                key: "${{ runner.os }}-node-${{ hashFiles('**/package-lock.json') }}",
            },
        }))
    )
    .build("setup-env");
```

### JavaScript Action

**Before** (`.github/actions/notify/action.yml`):

```yaml
name: Send Notification
description: Send a Slack notification
inputs:
  webhook-url:
    description: Slack webhook URL
    required: true
  message:
    description: Message to send
    required: true
runs:
  using: node20
  main: dist/index.js
  post: dist/cleanup.js
```

**After** (`workflows/action-notify.ts`):

```typescript
import { NodeAction } from "../generated/index.js";

const action = new NodeAction(
    {
        name: "Send Notification",
        description: "Send a Slack notification",
        inputs: {
            "webhook-url": {
                description: "Slack webhook URL",
                required: true,
            },
            message: {
                description: "Message to send",
                required: true,
            },
        },
    },
    {
        using: "node20",
        main: "dist/index.js",
        post: "dist/cleanup.js",
    },
);

action.build("notify");
```

### Docker Action

**Before** (`.github/actions/lint/action.yml`):

```yaml
name: Lint
description: Run linter in Docker
inputs:
  config:
    description: Config file path
    required: false
    default: ".lintrc"
runs:
  using: docker
  image: Dockerfile
  entrypoint: entrypoint.sh
  args:
    - --config
    - ${{ inputs.config }}
```

**After** (`workflows/action-lint.ts`):

```typescript
import { DockerAction } from "../generated/index.js";

const action = new DockerAction(
    {
        name: "Lint",
        description: "Run linter in Docker",
        inputs: {
            config: {
                description: "Config file path",
                required: false,
                default: ".lintrc",
            },
        },
    },
    {
        using: "docker",
        image: "Dockerfile",
        entrypoint: "entrypoint.sh",
        args: ["--config", "${{ inputs.config }}"],
    },
);

action.build("lint");
```

### Supported Action Types

| Type       | `runs.using`                 | Migrated To        |
| ---------- | ---------------------------- | ------------------ |
| Composite  | `composite`                  | `Action`           |
| JavaScript | `node12`, `node16`, `node20` | `NodeAction`       |
| Docker     | `docker`                     | `DockerAction`     |

## Manual Migration

If you prefer to migrate manually, follow this process.

### Step 1: Analyze Your YAML

Here is a simple YAML workflow as an example.

```yaml
name: CI
on:
  push:
    branches: [main]
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v5
      - uses: actions/setup-node@v4
        with:
          node-version: '20'
      - run: npm ci
      - run: npm test
```

### Step 2: Add Required Actions

```bash
gaji add actions/checkout@v5
gaji add actions/setup-node@v4
```

### Step 3: Convert to TypeScript

Create `workflows/ci.ts`:

```ts twoslash
// @filename: workflows/example.ts
// ---cut---
import { getAction, Job, Workflow } from "../generated/index.js";

// Import actions
const checkout = getAction("actions/checkout@v5");
const setupNode = getAction("actions/setup-node@v4");

// Create workflow with jobs and steps
new Workflow({
  name: "CI",
  on: {
    push: {
      branches: ["main"],
    },
  },
})
  .jobs(j => j
    .add("build",
      new Job("ubuntu-latest")
        .steps(s => s
          .add(checkout({}))
          .add(setupNode({
            with: { "node-version": "20" },
          }))
          .add({ run: "npm ci" })
          .add({ run: "npm test" })
        )
    )
  )
  .build("ci");
```

### Step 4: Build and Verify

```bash
# Build TypeScript to YAML
gaji build
```

### Step 5: Clean Up

Once verified, remove the backup:

```bash
rm .github/workflows/ci.yml.backup
```

## Common Migration Patterns

### Multiple Jobs

**YAML:**
```yaml
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - run: npm test

  build:
    needs: test
    runs-on: ubuntu-latest
    steps:
      - run: npm run build
```

**TypeScript:**
```typescript
const test = new Job("ubuntu-latest")
  .steps(s => s
    .add({ run: "npm test" })
  );

const build = new Job("ubuntu-latest", {
  needs: ["test"],
})
  .steps(s => s
    .add({ run: "npm run build" })
  );

new Workflow({
  name: "CI",
  on: { push: { branches: ["main"] } },
})
  .jobs(j => j
    .add("test", test)
    .add("build", build)
  )
  .build("ci");
```

### Matrix Strategy

**YAML:**
```yaml
jobs:
  test:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest]
        node: [18, 20, 22]
```

**TypeScript:**
```typescript
const test = new Job("${{ matrix.os }}", {
  strategy: {
    matrix: {
      os: ["ubuntu-latest", "macos-latest"],
      node: ["18", "20", "22"],
    },
  },
})
  .steps(s => s
    .add(checkout({}))
  );
```

### Environment Variables

**YAML:**
```yaml
env:
  NODE_ENV: production

jobs:
  deploy:
    runs-on: ubuntu-latest
    env:
      API_KEY: ${{ secrets.API_KEY }}
```

**TypeScript:**
```typescript
new Workflow({
  name: "Deploy",
  on: { push: { branches: ["main"] } },
  env: {
    NODE_ENV: "production",
  },
})
  .jobs(j => j
    .add("deploy",
      new Job("ubuntu-latest", {
        env: {
          API_KEY: "${{ secrets.API_KEY }}",
        },
      })
    )
  )
  .build("deploy");
```

### Conditional Steps

**YAML:**
```yaml
steps:
  - name: Deploy
    if: github.ref == 'refs/heads/main'
    run: npm run deploy
```

**TypeScript:**
```typescript
.add({
  name: "Deploy",
  if: "github.ref == 'refs/heads/main'",
  run: "npm run deploy",
})
```

### Job Outputs

**YAML:**
```yaml
jobs:
  build:
    outputs:
      version: ${{ steps.version.outputs.value }}
    steps:
      - id: version
        run: echo "value=1.0.0" >> $GITHUB_OUTPUT
```

**TypeScript:**
```typescript
const build = new Job("ubuntu-latest")
  .steps(s => s
    .add({
      id: "version",
      run: 'echo "value=1.0.0" >> $GITHUB_OUTPUT',
    })
  )
  .outputs({
    version: "${{ steps.version.outputs.value }}",
  });
```

## Configuration Migration

If your project uses the older `.gaji.toml` configuration file, gaji can migrate it to `gaji.config.ts` automatically.

### Automatic Migration

When you run `gaji init` in a project that has a `.gaji.toml`, gaji detects the file and offers to migrate it:

```bash
gaji init
# Detected .gaji.toml configuration file.
# Migrate to gaji.config.ts? [y/N]
```

If you confirm, gaji will:

1. Read `.gaji.toml` and generate a `gaji.config.ts` file with `defineConfig()`
2. If `.gaji.local.toml` exists, generate a corresponding `gaji.config.local.ts`
3. Remove the old TOML files after successful migration

### Manual Migration

To migrate manually, create `gaji.config.ts` at your project root:

**Before** (`.gaji.toml`):

```toml
workflows = "src/workflows"
output = ".github"

[build]
cache_ttl_days = 14

[github]
token = "ghp_xxx"
```

**After** (`gaji.config.ts`):

```typescript
import { defineConfig } from "./generated/index.js";

export default defineConfig({
  workflows: "src/workflows",
  build: {
    cacheTtlDays: 14,
  },
});
```

Note that TOML keys use `snake_case` while TypeScript config uses `camelCase`. Secrets like `github.token` should go in `gaji.config.local.ts` (which should be added to `.gitignore`):

```typescript
// gaji.config.local.ts
import { defineConfig } from "./generated/index.js";

export default defineConfig({
  github: {
    token: "ghp_xxx",
  },
});
```

## Migration Checklist

1. Install gaji
2. Initialize project
3. Add all actions used in your workflows
4. Convert YAML to TypeScript
5. Build and verify generated YAML
6. Test workflows in a branch
7. Remove backup files
8. Update documentation

## Tips

### 1. Start Small

Migrate one workflow at a time, starting with the simplest one.

### 2. Use Automatic Migration

For complex workflows, let gaji do the initial conversion:

```bash
gaji init --migrate
```

Then refine the generated TypeScript.

### 3. Test in a Branch

Always test migrated workflows in a feature branch before merging.

### 4. Keep Both During Transition

During migration, you can keep both YAML and TypeScript versions:
- Backup old workflows with `.backup` extension (e.g., `ci.yml.backup`)
- New workflows: Generated from `workflows/*.ts` to `.github/workflows/*.yml`

## Troubleshooting

### Types Not Generated

Make sure you've added all actions:

```bash
gaji add actions/checkout@v5
gaji dev
```

### Build Fails

Check for TypeScript errors:

```bash
npx tsc --noEmit
```

### YAML Differs from Original

Minor formatting differences are normal. Verify functionality, not formatting.

## Next Steps

- Read the [CLI Reference](/reference/cli)
- See [Examples](/examples/simple-ci)
