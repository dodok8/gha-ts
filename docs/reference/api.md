# TypeScript API Reference

Reference for gaji's TypeScript API.

## Core Classes

### `Workflow`

Represents a GitHub Actions workflow.

```typescript
class Workflow {
  constructor(config: WorkflowConfig)
  addJob(id: string, job: Job<any> | CompositeJob<any> | CallJob): this
  static fromObject(def: WorkflowDefinition, id?: string): Workflow
  build(filename?: string): void
  toJSON(): WorkflowDefinition
}
```

| Method | Description |
|--------|-------------|
| `addJob(id, job)` | Add a job to the workflow. Accepts `Job`, `CompositeJob`, or `CallJob`. |
| `fromObject(def, id?)` | Create a Workflow from a raw `WorkflowDefinition` object. Useful for wrapping existing YAML-like definitions. |
| `build(filename?)` | Compile the workflow to YAML. |
| `toJSON()` | Serialize to a `WorkflowDefinition` object. |

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

Represents a job in a workflow. The type parameter `O` tracks the job's output keys for type-safe inter-job references via `jobOutputs()`.

```typescript
class Job<O extends Record<string, string> = {}> {
  constructor(runsOn: string | string[], options?: Partial<JobDefinition>)
  addStep(step: Step): this
  needs(jobs: string | string[]): this
  env(variables: Record<string, string>): this
  when(condition: string): this
  permissions(perms: Permissions): this
  outputs<T extends Record<string, string>>(outputs: T): Job<T>
  strategy(strategy: JobStrategy): this
  continueOnError(v: boolean): this
  timeoutMinutes(m: number): this
  toJSON(): JobDefinition
}
```

| Method | Description |
|--------|-------------|
| `addStep(step)` | Append a step to the job. |
| `needs(jobs)` | Set job dependencies. |
| `env(variables)` | Set environment variables. |
| `when(condition)` | Set the job's `if` condition (e.g., `"github.ref == 'refs/heads/main'"`). |
| `permissions(perms)` | Set job-level permissions (e.g., `{ contents: 'read' }`). |
| `outputs(outputs)` | Define job outputs. Returns `Job<T>` where `T` captures the output keys. |
| `strategy(strategy)` | Set matrix strategy. |
| `continueOnError(v)` | Set the `continue-on-error` flag. |
| `timeoutMinutes(m)` | Set the `timeout-minutes` value. |
| `toJSON()` | Serialize to a `JobDefinition` object. |

The optional `options` parameter in the constructor allows setting all job options at once:

```typescript
const job = new Job("ubuntu-latest", {
  needs: ["test"],
  env: { NODE_ENV: "production" },
  "timeout-minutes": 30,
});
```

#### Example

```typescript
const job = new Job("ubuntu-latest")
  .needs(["test"])
  .env({
    NODE_ENV: "production",
  })
  .when("github.event_name == 'push'")
  .permissions({ contents: "read" })
  .strategy({
    matrix: {
      node: ["18", "20", "22"],
    },
  })
  .outputs({
    version: "${{ steps.version.outputs.value }}",
  })
  .continueOnError(false)
  .timeoutMinutes(30)
  .addStep(checkout({}))
  .addStep({ run: "npm test" });
```

---

### `CompositeAction`

Create reusable [composite actions](https://docs.github.com/en/actions/sharing-automations/creating-actions/creating-a-composite-action).

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

```ts twoslash
// @filename: workflows/example.ts
// ---cut---
import { CompositeAction, getAction } from "../generated/index.js";

const checkout = getAction("actions/checkout@v5");
const setupNode = getAction("actions/setup-node@v4");

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

### `JavaScriptAction`

Create [Node.js-based GitHub Actions](https://docs.github.com/en/actions/sharing-automations/creating-actions/creating-a-javascript-action).

```typescript
class JavaScriptAction {
  constructor(config: JavaScriptActionConfig, runs: JavaScriptActionRuns)
  build(filename: string): void
}
```

#### `JavaScriptActionConfig`

```typescript
interface JavaScriptActionConfig {
  name: string
  description: string
  inputs?: Record<string, ActionInputDefinition>
  outputs?: Record<string, ActionOutputDefinition>
}
```

#### `JavaScriptActionRuns`

```typescript
interface JavaScriptActionRuns {
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
import { JavaScriptAction } from "../generated/index.js";

const action = new JavaScriptAction(
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

This generates `.github/actions/hello-world/action.yml`.

Use `CallAction.from()` to reference it in a workflow:

```typescript
const step = {
  id: "hello",
  ...CallAction.from(action).toJSON(),
  with: { "who-to-greet": "Mona the Octocat" },
};
```

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

### `CompositeJob`

Create reusable job templates via TypeScript class inheritance. `CompositeJob` extends `Job`, so all `Job` methods are available.

```typescript
class CompositeJob<O extends Record<string, string> = {}> extends Job<O> {
  constructor(runsOn: string | string[], options?: Partial<JobDefinition>)
}
```

Unlike `Job`, `CompositeJob` is designed to be subclassed with `extends` to create domain-specific, parameterized job templates. It produces the same YAML output as a regular `Job`.

::: tip CompositeJob vs Job
Use `Job` directly for one-off jobs. Use `CompositeJob` when you want to create a **reusable class** that encapsulates a common job pattern with parameters.
:::

#### Example

```ts twoslash
// @filename: workflows/example.ts
// ---cut---
import { CompositeJob, getAction, Workflow } from "../generated/index.js";

const checkout = getAction("actions/checkout@v5");
const setupNode = getAction("actions/setup-node@v4");

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

### `CallJob`

Call a [reusable workflow](https://docs.github.com/en/actions/using-workflows/reusing-workflows) defined in another repository or file. Unlike `Job`, a `CallJob` has no `steps` — it delegates entirely to the referenced workflow via `uses`.

```typescript
class CallJob {
  constructor(uses: string)
  with(inputs: Record<string, unknown>): this
  secrets(s: Record<string, unknown> | 'inherit'): this
  needs(deps: string | string[]): this
  when(condition: string): this
  permissions(perms: Permissions): this
  toJSON(): object
}
```

| Method | Description |
|--------|-------------|
| `with(inputs)` | Pass inputs to the reusable workflow. |
| `secrets(s)` | Pass secrets explicitly, or use `'inherit'` to forward all secrets. |
| `needs(deps)` | Set job dependencies. |
| `when(condition)` | Set the job's `if` condition. |
| `permissions(perms)` | Set job-level permissions. |

#### Example

```ts twoslash
// @filename: workflows/example.ts
// ---cut---
import { CallJob, Workflow } from "../generated/index.js";

const deploy = new CallJob("octo-org/deploy/.github/workflows/deploy.yml@main")
  .with({ environment: "production" })
  .secrets("inherit")
  .needs(["build"]);

const workflow = new Workflow({
  name: "Release",
  on: { push: { tags: ["v*"] } },
})
  .addJob("deploy", deploy);

workflow.build("release");
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

### `CallAction`

Reference a local composite or JavaScript action built by gaji, for use as a step in a job.

```typescript
class CallAction {
  constructor(uses: string)
  static from(action: CompositeAction | JavaScriptAction | DockerAction): CallAction
  toJSON(): Step
}
```

| Method | Description |
|--------|-------------|
| `from(action)` | Create a `CallAction` from a `CompositeAction`, `JavaScriptAction`, or `DockerAction` instance. Automatically resolves the `.github/actions/<id>` path. |

#### Example

```ts twoslash
// @filename: workflows/example.ts
// ---cut---
import { CompositeAction, CallAction, Job } from "../generated/index.js";

const setupEnv = new CompositeAction({
  name: "Setup",
  description: "Setup environment",
});
setupEnv.build("setup-env");

const job = new Job("ubuntu-latest")
  .addStep({
    ...CallAction.from(setupEnv).toJSON(),
    with: { "node-version": "20" },
  });
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
const setupNode = getAction("actions/setup-node@v4");

// Use with full type safety
const step = checkout({
  name: "Checkout code",
  with: {
    // Autocomplete available for inputs
    repository: "owner/repo",
    ref: "main",
    "fetch-depth": 0,
  },
});

// Typed step outputs (requires id)
const checkoutStep = checkout({ id: "my-checkout" });
// checkoutStep.outputs.ref → "${{ steps.my-checkout.outputs.ref }}"
// checkoutStep.outputs.commit → "${{ steps.my-checkout.outputs.commit }}"

const job = new Job("ubuntu-latest")
  .addStep(checkoutStep)
  .addStep({ run: `echo ${checkoutStep.outputs.ref}` });
```

---

### `jobOutputs()`

Create typed references to a job's outputs for use in downstream jobs. Reads the output keys from the `Job` object's `.outputs()` call and generates `${{ needs.<jobId>.outputs.<key> }}` expressions.

```typescript
function jobOutputs<O extends Record<string, string>>(
  jobId: string,
  job: Job<O>,
): JobOutputs<O>
```

#### Example

```typescript
const checkout = getAction("actions/checkout@v5");
const step = checkout({ id: "my-checkout" });

const build = new Job("ubuntu-latest")
  .addStep(step)
  .outputs({ ref: step.outputs.ref, sha: step.outputs.commit });

// Create typed references for downstream jobs
const buildOutputs = jobOutputs("build", build);
// buildOutputs.ref → "${{ needs.build.outputs.ref }}"
// buildOutputs.sha → "${{ needs.build.outputs.sha }}"

const deploy = new Job("ubuntu-latest")
  .needs("build")
  .addStep({ run: `echo ${buildOutputs.ref}` });

const workflow = new Workflow({
  name: "CI",
  on: { push: { branches: ["main"] } },
})
  .addJob("build", build)
  .addJob("deploy", deploy);
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
interface ActionStep<O = {}> extends JobStep {
  readonly outputs: O
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

Mapped type for typed job output references. Each key resolves to a `${{ needs.<jobId>.outputs.<key> }}` expression.

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

```ts twoslash
// @filename: workflows/example.ts
// ---cut---
import { getAction, Job, Workflow } from "../generated/index.js";

const checkout = getAction("actions/checkout@v5");
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
