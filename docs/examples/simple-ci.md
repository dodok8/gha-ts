# Example: Simple CI

A basic CI workflow that runs tests on push and pull requests.

## Workflow

```typescript
import { getAction, Job, Workflow } from "../../generated/index.js";

// Add actions
const checkout = getAction("actions/checkout@v4");
const setupNode = getAction("actions/setup-node@v4");

// Define the test job
const test = new Job("ubuntu-latest")
  .addStep(checkout({
    name: "Checkout code",
  }))
  .addStep(setupNode({
    name: "Setup Node.js",
    with: {
      "node-version": "20",
      cache: "npm",
    },
  }))
  .addStep({
    name: "Install dependencies",
    run: "npm ci",
  })
  .addStep({
    name: "Run linter",
    run: "npm run lint",
  })
  .addStep({
    name: "Run tests",
    run: "npm test",
  });

// Create the workflow
const workflow = new Workflow({
  name: "CI",
  on: {
    push: {
      branches: ["main"],
    },
    pull_request: {
      branches: ["main"],
    },
  },
}).addJob("test", test);

// Build to YAML
workflow.build("ci");
```

## Generated YAML

```yaml
name: CI
on:
  push:
    branches:
      - main
  pull_request:
    branches:
      - main
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout code
        uses: actions/checkout@v4
      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: npm
      - name: Install dependencies
        run: npm ci
      - name: Run linter
        run: npm run lint
      - name: Run tests
        run: npm test
```

## Setup

1. **Add required actions**:
   ```bash
   gaji add actions/checkout@v4
   gaji add actions/setup-node@v4
   ```

2. **Generate types**:
   ```bash
   gaji dev
   ```

3. **Create workflow**:
   Create `workflows/ci.ts` with the code above.

4. **Build**:
   ```bash
   gaji build
   ```

## Customization

### Different Node Version

```typescript
setupNode({
  with: {
    "node-version": "18",  // Use Node.js 18
  },
})
```

### Multiple Package Managers

```typescript
setupNode({
  with: {
    "node-version": "20",
    cache: "pnpm",  // or "yarn"
  },
})
```

### Add Build Step

```typescript
const test = new Job("ubuntu-latest")
  .addStep(checkout({}))
  .addStep(setupNode({ with: { "node-version": "20" } }))
  .addStep({ run: "npm ci" })
  .addStep({ run: "npm test" })
  .addStep({ run: "npm run build" });  // Add build
```

## Next Steps

- See [Matrix Build](./matrix-build.md) for testing multiple versions
- Learn about [Composite Actions](./composite-action.md)
