# Example: Composite Action

Create reusable composite actions and jobs.

## Composite Action

Create a reusable action that can be shared across workflows.

### Creating the Action

Create `actions/setup-env/action.ts`:

```ts twoslash
// @filename: examples/workflows/example.ts
// ---cut---
import { Action, getAction } from "../../generated/index.js";

const checkout = getAction("actions/checkout@v5");
const setupNode = getAction("actions/setup-node@v4");
const cache = getAction("actions/cache@v4");

const setupEnv = new Action({
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
  .steps(s => s
    .add(checkout({
      name: "Checkout code",
    }))
    .add(setupNode({
      id: "setup-node",
      name: "Setup Node.js",
      with: {
        "node-version": "${{ inputs.node-version }}",
        cache: "npm",
        "cache-dependency-path": "${{ inputs.cache-dependency-path }}",
      },
    }))
    .add({
      name: "Install dependencies",
      run: "npm ci",
    })
  );

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
  .steps(s => s
    .add(setupEnv({
      name: "Setup environment",
      with: {
        "node-version": "20",
      },
    }))
    .add({
      name: "Run tests",
      run: "npm test",
    })
  );

const workflow = new Workflow({
  name: "CI",
  on: { push: { branches: ["main"] } },
}).jobs(j => j
    .add("test", test)
  );

workflow.build("ci");
```

## Job Inheritance

Create reusable job templates by extending `Job`.

### Basic Example

```ts twoslash
// @filename: examples/workflows/example.ts
// ---cut---
import { Job, getAction, Workflow } from "../../generated/index.js";

const checkout = getAction("actions/checkout@v5");
const setupNode = getAction("actions/setup-node@v4");

// Define a reusable job class
class NodeTestJob extends Job {
  constructor(nodeVersion: string) {
    super("ubuntu-latest");

    this.steps(s => s
      .add(checkout({
        name: "Checkout code",
      }))
      .add(setupNode({
        name: `Setup Node.js ${nodeVersion}`,
        with: {
          "node-version": nodeVersion,
          cache: "npm",
        },
      }))
      .add({
        name: "Install dependencies",
        run: "npm ci",
      })
      .add({
        name: "Run tests",
        run: "npm test",
      })
    );
  }
}

// Use in workflow
const workflow = new Workflow({
  name: "Test Matrix",
  on: { push: { branches: ["main"] } },
}).jobs(j => j
    .add("test-node-18", new NodeTestJob("18"))
    .add("test-node-20", new NodeTestJob("20"))
    .add("test-node-22", new NodeTestJob("22"))
  );

workflow.build("test-matrix");
```

### Advanced Example: Parameterized Deploy Job

```ts twoslash
// @filename: examples/workflows/example.ts
// ---cut---
import { Job, getAction, Workflow } from "../../generated/index.js";

const checkout = getAction("actions/checkout@v5");
const setupNode = getAction("actions/setup-node@v4");

class DeployJob extends Job {
  constructor(
    environment: "staging" | "production",
    region: string = "us-east-1",
    config: Record<string, unknown> = {}
  ) {
    super("ubuntu-latest", {
      env: {
        ENVIRONMENT: environment,
        REGION: region,
        API_URL: environment === "production"
          ? "https://api.example.com"
          : "https://staging.api.example.com",
      },
      ...config,
    });

    this.steps(s => s
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
        name: "Build",
        run: `npm run build:${environment}`,
      })
      .add({
        name: `Deploy to ${environment}`,
        run: `npm run deploy`,
        env: {
          DEPLOY_TOKEN: "${{ secrets.DEPLOY_TOKEN }}",
          AWS_REGION: region,
        },
      })
    );
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
}).jobs(j => j
    .add("deploy-staging-us", new DeployJob("staging", "us-east-1"))
    .add("deploy-staging-eu", new DeployJob("staging", "eu-west-1"))
    .add("deploy-production",
      new DeployJob("production", "us-east-1", {
        needs: ["deploy-staging-us", "deploy-staging-eu"],
      })
    )
  );

workflow.build("deploy");
```

### Complex Example: Reusable Test Suite

```ts twoslash
// @filename: examples/workflows/example.ts
// ---cut---
import { Job, getAction, Workflow } from "../../generated/index.js";

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

class TestSuiteJob extends Job {
  constructor(options: TestOptions) {
    super("ubuntu-latest");

    this.steps(s => {
      // Setup
      s = s
        .add(checkout({}))
        .add(setupNode({
          with: {
            "node-version": options.nodeVersion,
            cache: "npm",
          },
        }))
        .add({ run: "npm ci" });

      // Lint
      s = s.add({
        name: "Run linter",
        run: "npm run lint",
      });

      // Tests
      if (options.coverage) {
        s = s.add({
          name: "Run tests with coverage",
          run: "npm run test:coverage",
        });

        s = s.add(uploadCodecov({
          name: "Upload coverage",
          with: {
            files: "./coverage/lcov.info",
          },
        }));
      } else {
        s = s.add({
          name: "Run tests",
          run: "npm test",
        });
      }

      // Additional tests
      if (options.additionalTests) {
        for (const test of options.additionalTests) {
          s = s.add({
            name: `Run ${test}`,
            run: `npm run test:${test}`,
          });
        }
      }

      // Upload artifacts
      if (options.uploadArtifacts) {
        s = s.add(uploadArtifact({
          name: "Upload test results",
          with: {
            name: "test-results",
            path: "test-results/",
          },
        }));
      }

      return s;
    });
  }
}

// Use in workflow
const workflow = new Workflow({
  name: "Full Test Suite",
  on: {
    push: { branches: ["main"] },
    pull_request: { branches: ["main"] },
  },
}).jobs(j => j
    .add("test-basic", new TestSuiteJob({
      nodeVersion: "20",
    }))
    .add("test-full", new TestSuiteJob({
      nodeVersion: "20",
      coverage: true,
      uploadArtifacts: true,
      additionalTests: ["integration", "e2e"],
    }))
  );

workflow.build("full-test");
```

## Benefits

Composite actions and Job inheritance let you define patterns once and reuse them across workflows. Action inputs and job parameters are type-checked, so refactoring is safer. Updates to the shared definition propagate to all callers.

## Further Reading

- [Composite Action](https://docs.github.com/en/actions/tutorials/create-actions/create-a-composite-action)

## Next Steps

- See [API Reference](/reference/api)
- Learn about [Matrix Builds](./matrix-build.md)
