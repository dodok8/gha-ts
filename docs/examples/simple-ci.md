# Example: Simple CI

A basic CI workflow that runs tests on push and pull requests.

## Workflow

```ts twoslash
// @filename: examples/workflows/example.ts
// ---cut---
import { getAction, Job, Workflow } from "../../generated/index.js";

// Add actions
const checkout = getAction("actions/checkout@v5");
const setupNode = getAction("actions/setup-node@v4");

// Define the test job
const test = new Job("ubuntu-latest")
  .steps(s => s
    .add(checkout({
      name: "Checkout code",
    }))
    .add(setupNode({
      name: "Setup Node.js",
      with: {
        "node-version": "20",
        cache: "npm",
      },
    }))
    .add({
      name: "Install dependencies",
      run: "npm ci",
    })
    .add({
      name: "Run linter",
      run: "npm run lint",
    })
    .add({
      name: "Run tests",
      run: "npm test",
    })
  );

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
}).jobs(j => j
    .add("test", test)
  );

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
        uses: actions/checkout@v5
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

1. **Generate types**:
   ```bash
   gaji dev --watch
   ```

2. **Create workflow**:
   Create `workflows/ci.ts` with the code above.

3. **Build**:
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
  .steps(s => s
    .add(checkout({}))
    .add(setupNode({ with: { "node-version": "20" } }))
    .add({ run: "npm ci" })
    .add({ run: "npm test" })
    .add({ run: "npm run build" })  // Add build
  );
```

## Next Steps

- See [Matrix Build](./matrix-build.md) for testing multiple versions
- Learn about [Composite Actions](./composite-action.md)
