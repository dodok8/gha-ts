# Example: Composite Action

Create reusable composite actions and jobs.

## Composite Action

Create a reusable action that can be shared across workflows.

### Creating the Action

Create `actions/setup-env/action.ts`:

```ts twoslash
// @filename: examples/workflows/example.ts
// ---cut---
import { CompositeAction, getAction } from "../../generated/index.js";

const checkout = getAction("actions/checkout@v5");
const setupNode = getAction("actions/setup-node@v4");
const cache = getAction("actions/cache@v4");

const setupEnv = new CompositeAction({
  name: "Setup Environment",
  description: "Setup Node.js and install dependencies",
  inputs: {
    "node-version": {
      description: "Node.js version to use",
      required: false,
      default: "20",
    },
    "cache-dependency-path": {
      description: "Path to dependency file",
      required: false,
      default: "package-lock.json",
    },
  },
  outputs: {
    "node-version": {
      description: "Installed Node.js version",
      value: "${{ steps.setup-node.outputs.node-version }}",
    },
  },
})
  .addStep(checkout({
    name: "Checkout code",
  }))
  .addStep(setupNode({
    id: "setup-node",
    name: "Setup Node.js",
    with: {
      "node-version": "${{ inputs.node-version }}",
      cache: "npm",
      "cache-dependency-path": "${{ inputs.cache-dependency-path }}",
    },
  }))
  .addStep({
    name: "Install dependencies",
    run: "npm ci",
  });

setupEnv.build("setup-env");
```

Build it:

```bash
gaji build
```

This generates `actions/setup-env/action.yml`.

### Using the Composite Action

In your workflow:

```ts twoslash
// @filename: examples/workflows/example.ts
// ---cut---
import { getAction, Job, Workflow } from "../../generated/index.js";

// Reference local composite action
const setupEnv = getAction("./actions/setup-env");

const test = new Job("ubuntu-latest")
  .addStep(setupEnv({
    name: "Setup environment",
    with: {
      "node-version": "20",
    },
  }))
  .addStep({
    name: "Run tests",
    run: "npm test",
  });

const workflow = new Workflow({
  name: "CI",
  on: { push: { branches: ["main"] } },
}).addJob("test", test);

workflow.build("ci");
```

## Composite Job

Create reusable job templates with `CompositeJob`.

### Basic Example

```ts twoslash
// @filename: examples/workflows/example.ts
// ---cut---
import { CompositeJob, getAction, Workflow } from "../../generated/index.js";

const checkout = getAction("actions/checkout@v5");
const setupNode = getAction("actions/setup-node@v4");

// Define a reusable job class
class NodeTestJob extends CompositeJob {
  constructor(nodeVersion: string) {
    super("ubuntu-latest");

    this
      .addStep(checkout({
        name: "Checkout code",
      }))
      .addStep(setupNode({
        name: `Setup Node.js ${nodeVersion}`,
        with: {
          "node-version": nodeVersion,
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
  }
}

// Use in workflow
const workflow = new Workflow({
  name: "Test Matrix",
  on: { push: { branches: ["main"] } },
})
  .addJob("test-node-18", new NodeTestJob("18"))
  .addJob("test-node-20", new NodeTestJob("20"))
  .addJob("test-node-22", new NodeTestJob("22"));

workflow.build("test-matrix");
```

### Advanced Example: Parameterized Deploy Job

```ts twoslash
// @filename: examples/workflows/example.ts
// ---cut---
import { CompositeJob, getAction, Workflow } from "../../generated/index.js";

const checkout = getAction("actions/checkout@v5");
const setupNode = getAction("actions/setup-node@v4");

class DeployJob extends CompositeJob {
  constructor(
    environment: "staging" | "production",
    region: string = "us-east-1"
  ) {
    super("ubuntu-latest");

    this
      .env({
        ENVIRONMENT: environment,
        REGION: region,
        API_URL: environment === "production"
          ? "https://api.example.com"
          : "https://staging.api.example.com",
      })
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
        name: "Build",
        run: `npm run build:${environment}`,
      })
      .addStep({
        name: `Deploy to ${environment}`,
        run: `npm run deploy`,
        env: {
          DEPLOY_TOKEN: "${{ secrets.DEPLOY_TOKEN }}",
          AWS_REGION: region,
        },
      });
  }
}

// Use in workflow
const workflow = new Workflow({
  name: "Deploy",
  on: {
    push: {
      tags: ["v*"],
    },
  },
})
  .addJob("deploy-staging-us", new DeployJob("staging", "us-east-1"))
  .addJob("deploy-staging-eu", new DeployJob("staging", "eu-west-1"))
  .addJob("deploy-production",
    new DeployJob("production", "us-east-1")
      .needs(["deploy-staging-us", "deploy-staging-eu"])
  );

workflow.build("deploy");
```

### Complex Example: Reusable Test Suite

```ts twoslash
// @filename: examples/workflows/example.ts
// ---cut---
import { CompositeJob, getAction, Workflow } from "../../generated/index.js";

const checkout = getAction("actions/checkout@v5");
const setupNode = getAction("actions/setup-node@v4");
const uploadArtifact = getAction("actions/upload-artifact@v4");
const uploadCodecov = getAction("codecov/codecov-action@v4");

interface TestOptions {
  nodeVersion: string;
  coverage?: boolean;
  uploadArtifacts?: boolean;
  additionalTests?: string[];
}

class TestSuiteJob extends CompositeJob {
  constructor(options: TestOptions) {
    super("ubuntu-latest");

    // Setup
    this
      .addStep(checkout({}))
      .addStep(setupNode({
        with: {
          "node-version": options.nodeVersion,
          cache: "npm",
        },
      }))
      .addStep({ run: "npm ci" });

    // Lint
    this.addStep({
      name: "Run linter",
      run: "npm run lint",
    });

    // Tests
    if (options.coverage) {
      this.addStep({
        name: "Run tests with coverage",
        run: "npm run test:coverage",
      });

      this.addStep(uploadCodecov({
        name: "Upload coverage",
        with: {
          files: "./coverage/lcov.info",
        },
      }));
    } else {
      this.addStep({
        name: "Run tests",
        run: "npm test",
      });
    }

    // Additional tests
    if (options.additionalTests) {
      for (const test of options.additionalTests) {
        this.addStep({
          name: `Run ${test}`,
          run: `npm run test:${test}`,
        });
      }
    }

    // Upload artifacts
    if (options.uploadArtifacts) {
      this.addStep(uploadArtifact({
        name: "Upload test results",
        with: {
          name: "test-results",
          path: "test-results/",
        },
      }));
    }
  }
}

// Use in workflow
const workflow = new Workflow({
  name: "Full Test Suite",
  on: {
    push: { branches: ["main"] },
    pull_request: { branches: ["main"] },
  },
})
  .addJob("test-basic", new TestSuiteJob({
    nodeVersion: "20",
  }))
  .addJob("test-full", new TestSuiteJob({
    nodeVersion: "20",
    coverage: true,
    uploadArtifacts: true,
    additionalTests: ["integration", "e2e"],
  }));

workflow.build("full-test");
```

## Benefits

Composite actions and CompositeJob let you define patterns once and reuse them across workflows. Action inputs and job parameters are type-checked, so refactoring is safer. Updates to the shared definition propagate to all callers.

## Further Reading

- [Composite Action](https://docs.github.com/en/actions/tutorials/create-actions/create-a-composite-action)

## Next Steps

- See [API Reference](/reference/api)
- Learn about [Matrix Builds](./matrix-build.md)
