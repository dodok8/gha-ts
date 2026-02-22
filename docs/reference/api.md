# TypeScript API Reference

Reference for gaji's TypeScript API.

## Core Classes

### `Workflow`

Represents a GitHub Actions workflow.

```typescript
class Workflow<Cx = {}> {
  constructor(config: WorkflowConfig)
  jobs<NewCx>(callback: (j: JobBuilder<{}>) => JobBuilder<NewCx>): Workflow<NewCx>
  static fromObject(def: WorkflowDefinition, id?: string): Workflow
  build(filename?: string): void
  toJSON(): WorkflowDefinition
}
```

| Method | Description |
|--------|-------------|
| `jobs(callback)` | Define workflow jobs via a `JobBuilder` callback. The callback receives a fresh `JobBuilder` and should return it with jobs added via `.add()`. |
| `fromObject(def, id?)` | Create a Workflow from a raw `WorkflowDefinition` object. Useful for wrapping existing YAML-like definitions. |
| `build(filename?)` | Compile the workflow to YAML. |
| `toJSON()` | Serialize to a `WorkflowDefinition` object. |

#### `WorkflowConfig`

```typescript
interface WorkflowConfig {
  name?: string
  on: WorkflowOn
  env?: Record<string, string>
  permissions?: Permissions
  concurrency?: { group: string; 'cancel-in-progress'?: boolean } | string
  defaults?: { run?: { shell?: string; 'working-directory'?: string } }
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
  .jobs(j => j
    .add("test", testJob)
    .add("build", buildJob)
  );

workflow.build("ci");
```

#### `Workflow.fromObject()` Example

```typescript
const workflow = Workflow.fromObject({
  name: "Raw Workflow",
  on: { push: {} },
  jobs: {
    test: {
      "runs-on": "ubuntu-latest",
      steps: [{ run: "echo hello" }],
    },
  },
});

workflow.build("raw");
```

---

### `Job`

Represents a job in a workflow. Has two type parameters: `Cx` tracks accumulated step output context (from `.steps()`), and `O` tracks the job's declared output keys (from `.outputs()`).

```typescript
class Job<Cx = {}, O extends Record<string, string> = {}> {
  constructor(runsOn: string | string[], config?: JobConfig)
  steps<NewCx>(callback: (s: StepBuilder<{}>) => StepBuilder<NewCx>): Job<NewCx, O>
  outputs<T extends Record<string, string>>(outputs: T | ((output: Cx) => T)): Job<Cx, T>
  toJSON(): JobDefinition
}
```

| Method | Description |
|--------|-------------|
| `steps(callback)` | Define job steps via a `StepBuilder` callback. The callback receives a fresh `StepBuilder` and should return it with steps added via `.add()`. |
| `outputs(outputs)` | Define job outputs. Accepts a plain object or a callback that receives the step output context (`Cx`). |
| `toJSON()` | Serialize to a `JobDefinition` object. |

#### `JobConfig`

All job-level settings are passed via the constructor's `config` parameter:

```typescript
interface JobConfig {
  permissions?: Permissions
  needs?: string[]
  strategy?: { matrix?: Record<string, unknown>; 'fail-fast'?: boolean; 'max-parallel'?: number }
  if?: string
  environment?: string | { name: string; url?: string }
  concurrency?: { group: string; 'cancel-in-progress'?: boolean } | string
  'timeout-minutes'?: number
  env?: Record<string, string>
  defaults?: { run?: { shell?: string; 'working-directory'?: string } }
  services?: Record<string, Service>
  container?: Container
  'continue-on-error'?: boolean
}
```

#### Example

```typescript
const checkout = getAction("actions/checkout@v5");

const job = new Job("ubuntu-latest", {
  needs: ["test"],
  env: { NODE_ENV: "production" },
  if: "github.event_name == 'push'",
  permissions: { contents: "read" },
  strategy: {
    matrix: {
      node: ["18", "20", "22"],
    },
  },
  "continue-on-error": false,
  "timeout-minutes": 30,
})
  .steps(s => s
    .add(checkout({}))
    .add({ run: "npm test" })
  )
  .outputs({
    version: "${{ steps.version.outputs.value }}",
  });
```

---

### `StepBuilder`

Accumulates steps inside a `.steps()` callback. Each `.add()` call appends a step and updates the output context `Cx` when a step has an `id` and typed outputs.

```typescript
class StepBuilder<Cx = {}> {
  add<Id extends string, StepO>(step: ActionStep<StepO, Id>): StepBuilder<Cx & Record<Id, StepO>>
  add(step: JobStep): StepBuilder<Cx>
  add<Id extends string, StepO>(stepFn: (output: Cx) => ActionStep<StepO, Id>): StepBuilder<Cx & Record<Id, StepO>>
  add(stepFn: (output: Cx) => JobStep): StepBuilder<Cx>
}
```

The four overloads cover:

| Overload | Description |
|----------|-------------|
| `add(actionStep)` | Add an `ActionStep` with typed outputs (returned by `getAction()` with `id`). Merges outputs into `Cx`. |
| `add(jobStep)` | Add a plain `JobStep` (run command or action without `id`). `Cx` unchanged. |
| `add(output => actionStep)` | Callback form — receives previous step outputs (`Cx`), returns an `ActionStep`. |
| `add(output => jobStep)` | Callback form — receives previous step outputs (`Cx`), returns a `JobStep`. |

#### Example

```typescript
const checkout = getAction("actions/checkout@v5");

new Job("ubuntu-latest")
  .steps(s => s
    .add(checkout({ id: "co" }))
    .add(output => ({
      name: "Use ref",
      run: "echo " + output.co.ref,  // "${{ steps.co.outputs.ref }}"
    }))
  );
```

---

### `JobBuilder`

Accumulates jobs inside a `.jobs()` callback. Each `.add()` call registers a job and updates the output context `Cx` when a job has declared outputs.

```typescript
class JobBuilder<Cx = {}> {
  add<Id extends string, O extends Record<string, string>>(
    id: Id, job: Job<any, O>
  ): JobBuilder<Cx & Record<Id, O>>
  add(id: string, job: Job | WorkflowCall): JobBuilder<Cx>
  add<Id extends string, O extends Record<string, string>>(
    id: Id, jobFn: (output: Cx) => Job<any, O>
  ): JobBuilder<Cx & Record<Id, O>>
  add(id: string, jobFn: (output: Cx) => Job | WorkflowCall): JobBuilder<Cx>
}
```

The four overloads cover:

| Overload | Description |
|----------|-------------|
| `add(id, job)` | Add a `Job` with known outputs. Merges outputs into `Cx`. |
| `add(id, job)` | Add a `Job` or `WorkflowCall` without output tracking. |
| `add(id, output => job)` | Callback form — receives previous job outputs (`Cx`), returns a `Job`. |
| `add(id, output => job)` | Callback form — receives previous job outputs (`Cx`), returns a `Job` or `WorkflowCall`. |

#### Example

```typescript
new Workflow({ name: "CI", on: { push: {} } })
  .jobs(j => j
    .add("build",
      new Job("ubuntu-latest")
        .steps(s => s.add(checkout({ id: "co" })))
        .outputs(output => ({ ref: output.co.ref }))
    )
    .add("deploy", output =>
      new Job("ubuntu-latest", { needs: ["build"] })
        .steps(s => s
          .add({ run: "echo " + output.build.ref })
        )
    )
  )
  .build("ci");
```

---

### `Action`

Create reusable [composite actions](https://docs.github.com/en/actions/sharing-automations/creating-actions/creating-a-composite-action).

```typescript
class Action<Cx = {}> {
  constructor(config: { name: string; description: string; inputs?: Record<string, unknown>; outputs?: Record<string, unknown> })
  steps<NewCx>(callback: (s: StepBuilder<{}>) => StepBuilder<NewCx>): Action<NewCx>
  outputMapping<T extends Record<string, string>>(mapping: (output: Cx) => T): Action<Cx>
  build(filename: string): void
  toJSON(): object
}
```

| Method | Description |
|--------|-------------|
| `steps(callback)` | Define action steps via a `StepBuilder` callback. |
| `outputMapping(fn)` | Map step outputs to action outputs. |
| `build(filename)` | Compile the action to `action.yml`. |

#### Example

```ts twoslash
// @filename: workflows/example.ts
// ---cut---
import { Action, getAction } from "../generated/index.js";

const checkout = getAction("actions/checkout@v5");
const setupNode = getAction("actions/setup-node@v4");

new Action({
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
  .steps(s => s
    .add(checkout({}))
    .add(setupNode({
      with: {
        "node-version": "${{ inputs.node-version }}",
      },
    }))
    .add({ run: "npm ci" })
  )
  .build("setup-env");
```

This generates `action.yml` that can be used like:

```typescript
// In another workflow
const setupEnv = getAction("./setup-env");

new Job("ubuntu-latest")
  .steps(s => s
    .add(setupEnv({
      with: { "node-version": "20" },
    }))
  );
```

---

### `NodeAction`

Create [Node.js-based GitHub Actions](https://docs.github.com/en/actions/sharing-automations/creating-actions/creating-a-javascript-action).

```typescript
class NodeAction {
  constructor(config: NodeActionConfig, runs: NodeActionRuns)
  build(filename: string): void
}
```

#### `NodeActionConfig`

```typescript
interface NodeActionConfig {
  name: string
  description: string
  inputs?: Record<string, ActionInputDefinition>
  outputs?: Record<string, ActionOutputDefinition>
}
```

#### `NodeActionRuns`

```typescript
interface NodeActionRuns {
  using: 'node12' | 'node16' | 'node20'
  main: string
  pre?: string
  post?: string
  'pre-if'?: string
  'post-if'?: string
}
```

#### Example

```ts twoslash
// @filename: workflows/example.ts
// ---cut---
import { NodeAction } from "../generated/index.js";

const action = new NodeAction(
  {
    name: "Hello World",
    description: "Greet someone and record the time",
    inputs: {
      "who-to-greet": {
        description: "Who to greet",
        required: true,
        default: "World",
      },
    },
    outputs: {
      time: {
        description: "The time we greeted you",
      },
    },
  },
  {
    using: "node20",
    main: "dist/index.js",
  },
);

action.build("hello-world");
```

This generates `.github/actions/hello-world/action.yml`. Use [`ActionRef.from()`](#actionref) to reference it in a workflow.

For a complete example with workflow integration, see [NodeAction Example](/examples/javascript-action).

---

### `DockerAction`

Create [Docker container actions](https://docs.github.com/en/actions/sharing-automations/creating-actions/creating-a-docker-container-action).

```typescript
class DockerAction {
  constructor(config: DockerActionConfig, runs: DockerActionRuns)
  build(filename: string): void
}
```

#### `DockerActionConfig`

```typescript
interface DockerActionConfig {
  name: string
  description: string
  inputs?: Record<string, ActionInputDefinition>
  outputs?: Record<string, ActionOutputDefinition>
}
```

#### `DockerActionRuns`

```typescript
interface DockerActionRuns {
  using: 'docker'
  image: string
  entrypoint?: string
  args?: string[]
  env?: Record<string, string>
  'pre-entrypoint'?: string
  'post-entrypoint'?: string
  'pre-if'?: string
  'post-if'?: string
}
```

#### Example

```ts twoslash
// @noErrors
// @filename: workflows/example.ts
// ---cut---
import { DockerAction } from "../generated/index.js";

const action = new DockerAction(
  {
    name: "Greeting",
    description: "Docker-based greeter",
    inputs: {
      name: {
        description: "Who to greet",
        required: true,
        default: "World",
      },
    },
  },
  {
    using: "docker",
    image: "Dockerfile",
    args: ["${{ inputs.name }}"],
  },
);

action.build("greeting");
```

This generates `.github/actions/greeting/action.yml`.

To use a Docker Hub image directly, prefix `image` with `docker://`:

```typescript
{
  using: "docker",
  image: "docker://alpine:3.19",
}
```

---

### Job Inheritance

Create reusable job templates via TypeScript class inheritance. Extend `Job` directly.

::: tip
Use `Job` directly for one-off jobs. Extend `Job` when you want to create a **reusable class** that encapsulates a common job pattern with parameters.
:::

#### Example

```ts twoslash
// @filename: workflows/example.ts
// ---cut---
import { Job, getAction, Workflow } from "../generated/index.js";

const checkout = getAction("actions/checkout@v5");
const setupNode = getAction("actions/setup-node@v4");

// Define a reusable job template
class NodeTestJob extends Job {
  constructor(nodeVersion: string) {
    super("ubuntu-latest");

    this.steps(s => s
      .add(checkout({}))
      .add(setupNode({
        with: { "node-version": nodeVersion },
      }))
      .add({ run: "npm ci" })
      .add({ run: "npm test" })
    );
  }
}

// Use in workflows
new Workflow({
  name: "Test Matrix",
  on: { push: { branches: ["main"] } },
})
  .jobs(j => j
    .add("test-node-18", new NodeTestJob("18"))
    .add("test-node-20", new NodeTestJob("20"))
    .add("test-node-22", new NodeTestJob("22"))
  )
  .build("test-matrix");
```

You can also create more complex reusable jobs:

```typescript
class DeployJob extends Job {
  constructor(environment: "staging" | "production") {
    super("ubuntu-latest", {
      env: {
        ENVIRONMENT: environment,
        API_URL: environment === "production"
          ? "https://api.example.com"
          : "https://staging.api.example.com",
      },
    });

    this.steps(s => s
      .add(checkout({}))
      .add(setupNode({ with: { "node-version": "20" } }))
      .add({
        name: "Deploy",
        run: `npm run deploy:${environment}`,
        env: {
          DEPLOY_TOKEN: "${{ secrets.DEPLOY_TOKEN }}",
        },
      })
    );
  }
}

// Use in workflow
new Workflow({
  name: "Deploy",
  on: { push: { tags: ["v*"] } },
})
  .jobs(j => j
    .add("deploy-staging", new DeployJob("staging"))
    .add("deploy-production", new DeployJob("production"))
  )
  .build("deploy");
```

---

### `WorkflowCall`

Call a [reusable workflow](https://docs.github.com/en/actions/using-workflows/reusing-workflows) defined in another repository or file. Unlike `Job`, a `WorkflowCall` has no `steps` — it delegates entirely to the referenced workflow via `uses`.

```typescript
class WorkflowCall {
  constructor(uses: string, config?: {
    with?: Record<string, unknown>
    secrets?: Record<string, unknown> | 'inherit'
    needs?: string[]
    if?: string
    permissions?: Permissions
  })
  toJSON(): object
}
```

All options are passed via the constructor's `config` parameter.

#### Example

```ts twoslash
// @filename: workflows/example.ts
// ---cut---
import { WorkflowCall, Workflow } from "../generated/index.js";

const deploy = new WorkflowCall(
  "octo-org/deploy/.github/workflows/deploy.yml@main",
  {
    with: { environment: "production" },
    secrets: "inherit",
    needs: ["build"],
  },
);

new Workflow({
  name: "Release",
  on: { push: { tags: ["v*"] } },
})
  .jobs(j => j
    .add("deploy", deploy)
  )
  .build("release");
```

Generated YAML:

```yaml
jobs:
  deploy:
    uses: octo-org/deploy/.github/workflows/deploy.yml@main
    with:
      environment: production
    secrets: inherit
    needs:
      - build
```

---

### `ActionRef`

Reference a local composite or JavaScript action built by gaji, for use as a step in a job.

```typescript
class ActionRef {
  constructor(uses: string)
  static from(action: Action | NodeAction | DockerAction): ActionRef
  toJSON(): Step
}
```

| Method | Description |
|--------|-------------|
| `from(action)` | Create an `ActionRef` from an `Action`, `NodeAction`, or `DockerAction` instance. Automatically resolves the `.github/actions/<id>` path. |

#### Example

```ts twoslash
// @filename: workflows/example.ts
// ---cut---
import { Action, ActionRef, Job } from "../generated/index.js";

const setupEnv = new Action({
  name: "Setup",
  description: "Setup environment",
});
setupEnv.build("setup-env");

new Job("ubuntu-latest")
  .steps(s => s
    .add({
      ...ActionRef.from(setupEnv).toJSON(),
      with: { "node-version": "20" },
    })
  );
```

---

## Functions

### `getAction()`

Get a typed action function. For actions that define outputs, `getAction()` returns a callable with two overloads:

- With `id` (required): returns `ActionStep<Outputs>` with typed output properties
- Without `id` (optional): returns `JobStep`

```typescript
// For actions WITH outputs (e.g., actions/checkout@v5)
function getAction(ref: 'actions/checkout@v5'): {
  (config: { id: string; with?: Inputs; ... }): ActionStep<Outputs>
  (config?: { id?: string; with?: Inputs; ... }): JobStep
}

// For actions WITHOUT outputs
function getAction(ref: 'actions/setup-node@v4'):
  (config?: { with?: Inputs; ... }) => JobStep

// Fallback for unknown actions
function getAction<T extends string>(ref: T): {
  (config: { id: string; ... }): ActionStep<Record<string, string>>
  (config?: { ... }): JobStep
}
```

#### Example

```typescript
const checkout = getAction("actions/checkout@v5");

// Use with full type safety
const step = checkout({
  with: {
    repository: "owner/repo",
    ref: "main",
    "fetch-depth": 0,
  },
});

// Typed step outputs (requires id)
const checkoutStep = checkout({ id: "my-checkout" });
// checkoutStep.outputs.ref → "${{ steps.my-checkout.outputs.ref }}"
```

For a complete typed outputs example, see [Outputs in Writing Workflows](../guide/writing-workflows.md#outputs).

---

### `jobOutputs()`

Create typed references to a job's outputs for use in downstream jobs. Reads the output keys from the `Job` object's `.outputs()` call and generates <code v-pre>${{ needs.&lt;jobId&gt;.outputs.&lt;key&gt; }}</code> expressions.

This is a compatibility helper. The primary pattern is to use the `.jobs()` callback, where job output context is passed automatically.

```typescript
function jobOutputs<O extends Record<string, string>>(
  jobId: string,
  job: Job<any, O>,
): JobOutputs<O>
```

#### Example

```typescript
const buildOutputs = jobOutputs("build", build);
// buildOutputs.ref → "${{ needs.build.outputs.ref }}"
// buildOutputs.sha → "${{ needs.build.outputs.sha }}"
```

For a complete example, see [Outputs in Writing Workflows](../guide/writing-workflows.md#outputs).

---

### `defineConfig()`

Type-safe configuration helper for `gaji.config.ts`.

```typescript
function defineConfig(config: GajiConfig): GajiConfig
```

#### `GajiConfig`

```typescript
interface GajiConfig {
  workflows?: string        // Default: "workflows"
  output?: string           // Default: ".github"
  generated?: string        // Default: "generated"
  watch?: {
    debounce?: number       // Default: 300 (ms)
    ignore?: string[]       // Default: ["node_modules", ".git", "generated"]
  }
  build?: {
    validate?: boolean      // Default: true
    format?: boolean        // Default: true
    cacheTtlDays?: number   // Default: 30
  }
  github?: {
    token?: string
    apiUrl?: string         // For GitHub Enterprise
  }
}
```

#### Example

```typescript
// gaji.config.ts
import { defineConfig } from "./generated/index.js";

export default defineConfig({
  workflows: "workflows",
  output: ".github",
  build: {
    cacheTtlDays: 14,
  },
});
```

---

## Type Definitions

### `JobStep`

A workflow step.

```typescript
interface JobStep {
  name?: string
  id?: string
  if?: string
  uses?: string
  with?: Record<string, unknown>
  run?: string
  env?: Record<string, string>
  shell?: string
  'working-directory'?: string
  "continue-on-error"?: boolean
  "timeout-minutes"?: number
}
```

### `ActionStep<O>`

A step returned by `getAction()` when `id` is provided. Extends `JobStep` with typed output access.

```typescript
interface ActionStep<O = {}, Id extends string = string> extends JobStep {
  readonly outputs: O
  readonly id: Id
}
```

Output properties resolve to expression strings:

```typescript
const step = checkout({ id: "co" });
step.outputs.ref  // "${{ steps.co.outputs.ref }}"
```

### `Step`

Union of step types.

```typescript
type Step = JobStep | ActionStep<any>
```

### `JobOutputs<T>`

Mapped type for typed job output references. Each key resolves to a <code v-pre>${{ needs.&lt;jobId&gt;.outputs.&lt;key&gt; }}</code> expression.

```typescript
type JobOutputs<T extends Record<string, string>> = {
  readonly [K in keyof T]: string
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

Action input definition (for Action).

```typescript
interface ActionInput {
  description: string
  required?: boolean
  default?: string
}
```

### `ActionOutput`

Action output definition (for Action).

```typescript
interface ActionOutput {
  description: string
  value: string
}
```

## Examples

### Complete Workflow

```ts twoslash
// @filename: workflows/example.ts
// ---cut---
import { getAction, Job, Workflow } from "../generated/index.js";

const checkout = getAction("actions/checkout@v5");
const setupNode = getAction("actions/setup-node@v4");

new Workflow({
  name: "CI",
  on: {
    push: { branches: ["main"] },
    pull_request: { branches: ["main"] },
  },
})
  .jobs(j => j
    .add("test",
      new Job("ubuntu-latest")
        .steps(s => s
          .add(checkout({}))
          .add(setupNode({ with: { "node-version": "20" } }))
          .add({ run: "npm ci" })
          .add({ run: "npm test" })
        )
    )
    .add("build",
      new Job("ubuntu-latest", { needs: ["test"] })
        .steps(s => s
          .add(checkout({}))
          .add(setupNode({ with: { "node-version": "20" } }))
          .add({ run: "npm ci" })
          .add({ run: "npm run build" })
        )
    )
  )
  .build("ci");
```

## Next Steps

- See [Examples](/examples/simple-ci)
- Learn about [Actions](./actions.md)
