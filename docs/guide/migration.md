# Migration

This guide helps you migrate existing YAML workflows to TypeScript with gaji.

## Automatic Migration

gaji can automatically convert existing YAML workflows to TypeScript:

```bash
gaji init --migrate
```

This will:
1. Detect existing YAML workflows in `.github/workflows/`
2. Convert them to TypeScript in `workflows/`
3. Backup original YAML files (`.yml.backup`)
4. Generate types for all actions used

## Manual Migration

If you prefer to migrate manually, follow these steps:

### Step 1: Analyze Your YAML

Start with a simple YAML workflow:

```yaml
name: CI
on:
  push:
    branches: [main]
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: '20'
      - run: npm ci
      - run: npm test
```

### Step 2: Add Required Actions

```bash
gaji add actions/checkout@v4
gaji add actions/setup-node@v4
```

### Step 3: Convert to TypeScript

Create `workflows/ci.ts`:

```typescript
import { getAction, Job, Workflow } from "../generated/index.js";

// Import actions
const checkout = getAction("actions/checkout@v4");
const setupNode = getAction("actions/setup-node@v4");

// Create job
const build = new Job("ubuntu-latest")
  .addStep(checkout({}))
  .addStep(setupNode({
    with: {
      "node-version": "20",
    },
  }))
  .addStep({ run: "npm ci" })
  .addStep({ run: "npm test" });

// Create workflow
const workflow = new Workflow({
  name: "CI",
  on: {
    push: {
      branches: ["main"],
    },
  },
}).addJob("build", build);

// Build YAML
workflow.build("ci");
```

### Step 4: Build and Verify

```bash
# Build TypeScript to YAML
gaji build

# Compare with original
diff .github/workflows/ci.yml .github/workflows/ci.yml.backup
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
  .addStep({ run: "npm test" });

const build = new Job("ubuntu-latest")
  .needs(["test"])
  .addStep({ run: "npm run build" });

const workflow = new Workflow({
  name: "CI",
  on: { push: { branches: ["main"] } },
})
  .addJob("test", test)
  .addJob("build", build);
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
const test = new Job("${{ matrix.os }}")
  .strategy({
    matrix: {
      os: ["ubuntu-latest", "macos-latest"],
      node: ["18", "20", "22"],
    },
  })
  .addStep(checkout({}));
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
const deploy = new Job("ubuntu-latest")
  .env({
    API_KEY: "${{ secrets.API_KEY }}",
  });

const workflow = new Workflow({
  name: "Deploy",
  on: { push: { branches: ["main"] } },
  env: {
    NODE_ENV: "production",
  },
}).addJob("deploy", deploy);
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
.addStep({
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
  .outputs({
    version: "${{ steps.version.outputs.value }}",
  })
  .addStep({
    id: "version",
    run: 'echo "value=1.0.0" >> $GITHUB_OUTPUT',
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
gaji add actions/checkout@v4
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
