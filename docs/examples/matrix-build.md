# Example: Matrix Build

Test across multiple operating systems and Node.js versions.

## Workflow

```typescript
import { getAction, Job, Workflow } from "../../generated/index.js";

const checkout = getAction("actions/checkout@v4");
const setupNode = getAction("actions/setup-node@v4");

// Define matrix test job
const test = new Job("${{ matrix.os }}")
  .strategy({
    matrix: {
      os: ["ubuntu-latest", "macos-latest", "windows-latest"],
      node: ["18", "20", "22"],
    },
  })
  .addStep(checkout({
    name: "Checkout code",
  }))
  .addStep(setupNode({
    name: "Setup Node.js ${{ matrix.node }}",
    with: {
      "node-version": "${{ matrix.node }}",
      cache: "npm",
    },
  }))
  .addStep({
    name: "Install dependencies",
    run: "npm ci",
  })
  .addStep({
    name: "Run tests",
    run: "npm test",
  });

// Create workflow
const workflow = new Workflow({
  name: "Matrix Test",
  on: {
    push: {
      branches: ["main"],
    },
    pull_request: {
      branches: ["main"],
    },
  },
}).addJob("test", test);

workflow.build("matrix-test");
```

## Generated YAML

```yaml
name: Matrix Test
on:
  push:
    branches:
      - main
  pull_request:
    branches:
      - main
jobs:
  test:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os:
          - ubuntu-latest
          - macos-latest
          - windows-latest
        node:
          - '18'
          - '20'
          - '22'
    steps:
      - name: Checkout code
        uses: actions/checkout@v4
      - name: Setup Node.js ${{ matrix.node }}
        uses: actions/setup-node@v4
        with:
          node-version: ${{ matrix.node }}
          cache: npm
      - name: Install dependencies
        run: npm ci
      - name: Run tests
        run: npm test
```

This creates **9 jobs** (3 OS × 3 Node versions).

## Matrix Variations

### Include/Exclude

```typescript
.strategy({
  matrix: {
    os: ["ubuntu-latest", "macos-latest", "windows-latest"],
    node: ["18", "20", "22"],
    include: [
      {
        os: "ubuntu-latest",
        node: "16",
        experimental: true,
      },
    ],
    exclude: [
      {
        os: "macos-latest",
        node: "18",  // Skip Node 18 on macOS
      },
    ],
  },
})
```

### Fail-Fast

```typescript
.strategy({
  "fail-fast": false,  // Continue even if one job fails
  matrix: {
    node: ["18", "20", "22"],
  },
})
```

### Max Parallel

```typescript
.strategy({
  "max-parallel": 2,  // Run max 2 jobs in parallel
  matrix: {
    node: ["18", "20", "22"],
  },
})
```

## Advanced Example: Test + Build

```typescript
import { getAction, Job, Workflow } from "../../generated/index.js";

const checkout = getAction("actions/checkout@v4");
const setupNode = getAction("actions/setup-node@v4");
const uploadArtifact = getAction("actions/upload-artifact@v4");

// Matrix test job
const test = new Job("${{ matrix.os }}")
  .strategy({
    matrix: {
      os: ["ubuntu-latest", "macos-latest", "windows-latest"],
      node: ["20"],
    },
  })
  .addStep(checkout({}))
  .addStep(setupNode({
    with: { "node-version": "${{ matrix.node }}" },
  }))
  .addStep({ run: "npm ci" })
  .addStep({ run: "npm test" });

// Build job (runs after all tests pass)
const build = new Job("ubuntu-latest")
  .needs(["test"])
  .addStep(checkout({}))
  .addStep(setupNode({ with: { "node-version": "20" } }))
  .addStep({ run: "npm ci" })
  .addStep({ run: "npm run build" })
  .addStep(uploadArtifact({
    with: {
      name: "build-output",
      path: "dist/",
    },
  }));

// Create workflow
const workflow = new Workflow({
  name: "Test and Build",
  on: {
    push: { branches: ["main"] },
  },
})
  .addJob("test", test)
  .addJob("build", build);

workflow.build("test-build");
```

## Real-World Example: Cross-Platform Binary

```typescript
const build = new Job("${{ matrix.os }}")
  .strategy({
    matrix: {
      include: [
        { os: "ubuntu-latest", target: "x86_64-unknown-linux-gnu", name: "linux-x64" },
        { os: "macos-latest", target: "x86_64-apple-darwin", name: "darwin-x64" },
        { os: "macos-latest", target: "aarch64-apple-darwin", name: "darwin-arm64" },
        { os: "windows-latest", target: "x86_64-pc-windows-msvc", name: "win32-x64" },
      ],
    },
  })
  .addStep(checkout({}))
  .addStep({
    name: "Build binary",
    run: "cargo build --release --target ${{ matrix.target }}",
  })
  .addStep(uploadArtifact({
    with: {
      name: "binary-${{ matrix.name }}",
      path: "target/${{ matrix.target }}/release/",
    },
  }));
```

## Next Steps

- See [Composite Action](./composite-action.md)
- Learn about [Job Dependencies](/reference/api#job)
