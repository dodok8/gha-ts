# TypeScript API Reference

Complete reference for gaji's TypeScript API.

## Core Classes

### `Workflow`

Represents a GitHub Actions workflow.

```typescript
class Workflow {
  constructor(config: WorkflowConfig)
  addJob(id: string, job: Job): this
  build(filename: string): void
}
```

#### `WorkflowConfig`

```typescript
interface WorkflowConfig {
  name: string
  on: WorkflowTriggers
  env?: Record<string, string>
  permissions?: WorkflowPermissions
  concurrency?: WorkflowConcurrency
}
```

#### Example

```typescript
const workflow = new Workflow({
  name: "CI",
  on: {
    push: { branches: ["main"] },
    pull_request: { branches: ["main"] },
  },
  env: {
    NODE_ENV: "production",
  },
})
  .addJob("test", testJob)
  .addJob("build", buildJob);

workflow.build("ci");
```

---

### `Job`

Represents a job in a workflow.

```typescript
class Job {
  constructor(runsOn: string | string[])
  addStep(step: Step): this
  needs(jobs: string[]): this
  strategy(strategy: JobStrategy): this
  env(variables: Record<string, string>): this
  outputs(outputs: Record<string, string>): this
}
```

#### Example

```typescript
const job = new Job("ubuntu-latest")
  .needs(["test"])
  .env({
    NODE_ENV: "production",
  })
  .strategy({
    matrix: {
      node: ["18", "20", "22"],
    },
  })
  .outputs({
    version: "${{ steps.version.outputs.value }}",
  })
  .addStep(checkout({}))
  .addStep({ run: "npm test" });
```

---

### `CompositeAction`

Create reusable composite actions.

```typescript
class CompositeAction {
  constructor(config: CompositeActionConfig)
  addStep(step: Step): this
  build(filename: string): void
}
```

#### `CompositeActionConfig`

```typescript
interface CompositeActionConfig {
  name: string
  description: string
  inputs?: Record<string, ActionInput>
  outputs?: Record<string, ActionOutput>
}
```

#### Example

```typescript
import { CompositeAction } from "../generated/index.js";

const setupEnv = new CompositeAction({
  name: "Setup Environment",
  description: "Setup Node.js and install dependencies",
  inputs: {
    "node-version": {
      description: "Node.js version",
      required: true,
      default: "20",
    },
  },
})
  .addStep(checkout({}))
  .addStep(setupNode({
    with: {
      "node-version": "${{ inputs.node-version }}",
    },
  }))
  .addStep({
    run: "npm ci",
  });

setupEnv.build("setup-env");
```

This generates `action.yml` that can be used like:

```typescript
// In another workflow
const setupEnv = getAction("./setup-env");

const job = new Job("ubuntu-latest")
  .addStep(setupEnv({
    with: {
      "node-version": "20",
    },
  }));
```

---

### `CompositeJob`

Create reusable job templates.

```typescript
class CompositeJob {
  constructor(runsOn: string | string[])
  addStep(step: Step): this
  needs(jobs: string[]): this
  strategy(strategy: JobStrategy): this
  env(variables: Record<string, string>): this
}
```

#### Example

```typescript
import { CompositeJob } from "../generated/index.js";

// Define a reusable job template
class NodeTestJob extends CompositeJob {
  constructor(nodeVersion: string) {
    super("ubuntu-latest");

    this
      .addStep(checkout({}))
      .addStep(setupNode({
        with: {
          "node-version": nodeVersion,
        },
      }))
      .addStep({ run: "npm ci" })
      .addStep({ run: "npm test" });
  }
}

// Use in workflows
const workflow = new Workflow({
  name: "Test Matrix",
  on: { push: { branches: ["main"] } },
})
  .addJob("test-node-18", new NodeTestJob("18"))
  .addJob("test-node-20", new NodeTestJob("20"))
  .addJob("test-node-22", new NodeTestJob("22"));
```

You can also create more complex reusable jobs:

```typescript
class DeployJob extends CompositeJob {
  constructor(environment: "staging" | "production") {
    super("ubuntu-latest");

    this
      .env({
        ENVIRONMENT: environment,
        API_URL: environment === "production"
          ? "https://api.example.com"
          : "https://staging.api.example.com",
      })
      .addStep(checkout({}))
      .addStep(setupNode({ with: { "node-version": "20" } }))
      .addStep({
        name: "Deploy",
        run: `npm run deploy:${environment}`,
        env: {
          DEPLOY_TOKEN: "${{ secrets.DEPLOY_TOKEN }}",
        },
      });
  }
}

// Use in workflow
const workflow = new Workflow({
  name: "Deploy",
  on: { push: { tags: ["v*"] } },
})
  .addJob("deploy-staging", new DeployJob("staging"))
  .addJob("deploy-production", new DeployJob("production").needs(["deploy-staging"]));
```

---

## Functions

### `getAction()`

Get a typed action function.

```typescript
function getAction<T extends string>(
  ref: T
): (config?: ActionConfig) => Step
```

#### Example

```typescript
const checkout = getAction("actions/checkout@v4");
const setupNode = getAction("actions/setup-node@v4");

// Use with full type safety
const step = checkout({
  name: "Checkout code",
  with: {
    // ✅ Autocomplete available!
    repository: "owner/repo",
    ref: "main",
    "fetch-depth": 0,
  },
});
```

---

## Type Definitions

### `Step`

A workflow step.

```typescript
interface Step {
  name?: string
  id?: string
  if?: string
  uses?: string
  with?: Record<string, string | number | boolean>
  run?: string
  env?: Record<string, string>
  "continue-on-error"?: boolean
  "timeout-minutes"?: number
}
```

### `WorkflowTriggers`

Workflow trigger events.

```typescript
interface WorkflowTriggers {
  push?: PushTrigger
  pull_request?: PullRequestTrigger
  schedule?: ScheduleTrigger[]
  workflow_dispatch?: WorkflowDispatchTrigger
  [key: string]: any
}

interface PushTrigger {
  branches?: string[]
  tags?: string[]
  paths?: string[]
}

interface PullRequestTrigger {
  branches?: string[]
  types?: string[]
  paths?: string[]
}

interface ScheduleTrigger {
  cron: string
}
```

### `JobStrategy`

Job matrix strategy.

```typescript
interface JobStrategy {
  matrix?: {
    [key: string]: string[] | number[]
  }
  "fail-fast"?: boolean
  "max-parallel"?: number
}
```

### `ActionInput`

Action input definition (for CompositeAction).

```typescript
interface ActionInput {
  description: string
  required?: boolean
  default?: string
}
```

### `ActionOutput`

Action output definition (for CompositeAction).

```typescript
interface ActionOutput {
  description: string
  value: string
}
```

## Examples

### Complete Workflow

```typescript
import { getAction, Job, Workflow } from "../generated/index.js";

const checkout = getAction("actions/checkout@v4");
const setupNode = getAction("actions/setup-node@v4");

const test = new Job("ubuntu-latest")
  .addStep(checkout({}))
  .addStep(setupNode({ with: { "node-version": "20" } }))
  .addStep({ run: "npm ci" })
  .addStep({ run: "npm test" });

const build = new Job("ubuntu-latest")
  .needs(["test"])
  .addStep(checkout({}))
  .addStep(setupNode({ with: { "node-version": "20" } }))
  .addStep({ run: "npm ci" })
  .addStep({ run: "npm run build" });

const workflow = new Workflow({
  name: "CI",
  on: {
    push: { branches: ["main"] },
    pull_request: { branches: ["main"] },
  },
})
  .addJob("test", test)
  .addJob("build", build);

workflow.build("ci");
```

## Next Steps

- See [Examples](/examples/simple-ci)
- Learn about [Actions](./actions.md)
