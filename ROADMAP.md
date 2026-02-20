# gaji 1.0 Plan

## Context

gaji 1.0 refines the TypeScript API with a callback builder pattern and constructor-only configuration:

1. **Step level**: `Job` has a `steps(s => s.add(...))` method. `StepBuilder` accumulates step output types in `Cx`. Callbacks receive previous step outputs via `output`.
2. **Job level**: `Workflow` has a `jobs(j => j.add(...))` method. `JobBuilder` accumulates job output types in `Cx`. Callbacks receive previous job outputs via `output`.
3. **Constructor-only config**: Job-level settings (permissions, needs, strategy, etc.) are passed through the constructor, not builder methods. This keeps configuration separate from step definitions.

The current API requires storing step/job references in variables to access outputs; the new API accumulates output types in `Cx` generic parameters so callbacks can access previous outputs through the `output` parameter. Additionally, several class names are simplified, and `CompositeJob` (which is identical to `Job`) is removed.

3. **TypeScript configuration**: `.gaji.toml` / `.gaji.local.toml` are replaced by `gaji.config.ts` / `gaji.config.local.ts`. Configuration becomes type-safe with autocomplete.

## Class Naming Changes

| Current | New | Notes |
|---------|-----|-------|
| `Job` | `Job` | Keep |
| `CompositeJob` | **removed** | Identical to `Job`; extend `Job` directly |
| `Workflow` | `Workflow` | Keep |
| `CompositeAction` | `Action` | Most common action type |
| `JavaScriptAction` | `NodeAction` | Shorter, matches GitHub's `using: nodeXX` |
| `DockerAction` | `DockerAction` | Keep |
| `CallJob` | `WorkflowCall` | Calls a reusable workflow |
| `CallAction` | `ActionRef` | Reference to a local action |
| `jobOutputs` | `jobOutputs` | Keep as compatibility helper; primary pattern moves to `Workflow.jobs()` context |
| `JavaScriptActionConfig` | `NodeActionConfig` | Matches `NodeAction` rename |
| `JavaScriptActionRuns` | `NodeActionRuns` | Matches `NodeAction` rename |
| `DockerActionConfig` | `DockerActionConfig` | Keep |
| `DockerActionRuns` | `DockerActionRuns` | Keep |

## Usage Examples (Before → After)

### Step output access — Before (current)
```typescript
const checkout = getAction("actions/checkout@v5");
const step = checkout({ id: "co" });

const build = new Job("ubuntu-latest")
  .addStep(step)
  .addStep({ name: "Use ref", run: "echo " + step.outputs.ref })
  .outputs({ ref: step.outputs.ref, sha: step.outputs.commit });

const buildOutputs = jobOutputs("build", build);

const deploy = new Job("ubuntu-latest")
  .needs("build")
  .addStep({ name: "Deploy", run: "echo " + buildOutputs.ref });

new Workflow({ name: "CI", on: { push: {} } })
  .addJob("build", build)
  .addJob("deploy", deploy)
  .build("ci");
```

### Step output access — After (gaji 1.0)
```typescript
const checkout = getAction("actions/checkout@v5");

new Workflow({ name: "CI", on: { push: {} } })
  .jobs(j => j
    .add("build",
      new Job("ubuntu-latest")
        .steps(s => s
          .add(checkout({ id: "co" }))
          .add(output => ({ name: "Use ref", run: "echo " + output.co.ref }))
        )
        .outputs(output => ({ ref: output.co.ref, sha: output.co.commit }))
    )
    .add("deploy", output =>
      new Job("ubuntu-latest", { needs: ["build"] })
        .steps(s => s
          .add({ name: "Deploy", run: "echo " + output.build.ref })
        )
    )
  )
  .build("ci");
```

Key differences:
- No need to store step/job in variables just to reference outputs
- `steps(s => s.add(output => ...))` callback receives accumulated step outputs via `output`
- `outputs(output => ...)` callback can also access step outputs
- `jobs(j => j.add("id", output => ...))` callback receives accumulated job outputs via `output`
- Job config (needs, permissions, strategy) is passed through the constructor
- Type error if accessing `output.co` when step has no `id`, or `output.build.nonExistent` when output not defined

### Class renames — Before
```typescript
import { CompositeAction, JavaScriptAction, CallJob, CallAction, CompositeJob } from "../generated/index.js";

const action = new CompositeAction({ name: "Setup", description: "..." });
const nodeAction = new JavaScriptAction({ ... }, { using: "node20", main: "index.js" });
const ref = CallAction.from(action);
const call = new CallJob("org/repo/.github/workflows/test.yml@main");
class Deploy extends CompositeJob { ... }
```

### Class renames — After
```typescript
import { Action, NodeAction, WorkflowCall, ActionRef } from "../generated/index.js";

const action = new Action({ name: "Setup", description: "..." });
const nodeAction = new NodeAction({ ... }, { using: "node20", main: "index.js" });
const ref = ActionRef.from(action);
const call = new WorkflowCall("org/repo/.github/workflows/test.yml@main");
class Deploy extends Job { ... }  // CompositeJob removed, extend Job directly
```

### Existing workflows — Migration needed
```typescript
// workflows/ci.ts — before
import { getAction, Job, Workflow } from "../generated/index.js";

const checkout = getAction("actions/checkout@v5");
const fmt = new Job("ubuntu-latest")
  .addStep(checkout({}))
  .addStep({ run: "cargo fmt --all --check" });

new Workflow({ name: "PR", on: { pull_request: { branches: ["main"] } } })
  .addJob("fmt", fmt)
  .build("pr");
```

```typescript
// workflows/ci.ts — after
import { getAction, Job, Workflow } from "../generated/index.js";

const checkout = getAction("actions/checkout@v5");

new Workflow({ name: "PR", on: { pull_request: { branches: ["main"] } } })
  .jobs(j => j
    .add("fmt",
      new Job("ubuntu-latest")
        .steps(s => s
          .add(checkout({}))
          .add({ run: "cargo fmt --all --check" })
        )
    )
  )
  .build("pr");
```

### Type error examples
```typescript
const checkout = getAction("actions/checkout@v5");

new Job("ubuntu-latest")
  .steps(s => s
    .add(checkout({}))  // no id → no context expansion
    .add(output => ({
      run: output.co.ref  // ❌ Type error: Property 'co' does not exist on type '{}'
    }))
  );

new Job("ubuntu-latest")
  .steps(s => s
    .add(checkout({ id: "co" }))  // id "co" → context expands with checkout outputs
    .add(output => ({
      run: output.co.ref      // ✅ OK: output.co has type CheckoutOutputs
    }))
  );
```

## API Design: `steps()` / `jobs()` Callback Builder with Constructor-Only Config

### Pattern

`Job` has a `steps()` method and `Workflow` has a `jobs()` method. Both take a callback receiving a builder (`s` for steps, `j` for jobs) with a short `add()` method. Context callbacks use `output` to access previous step/job outputs. All job-level configuration (permissions, needs, strategy, etc.) is passed through the constructor.

```typescript
const checkout = getAction("actions/checkout@v5");
const setupNode = getAction("actions/setup-node@v4");

const build = new Job("ubuntu-latest", {
    permissions: { contents: "read" },
  })
  .steps(s => s
    .add(checkout({ id: "co" }))
    .add(setupNode({ with: { "node-version": "20" } }))
    .add(output => ({
      name: "Show ref",
      run: "echo " + output.co.ref,
    }))
  )
  .outputs(output => ({ ref: output.co.ref }));
```

Type flow:
```
Job<{}>
  → .steps(s => s                                → StepBuilder<{}>
      .add(checkout({ id: "co" }))               → StepBuilder<{ co: CheckoutOutputs }>
      .add(setupNode({ ... }))                   → StepBuilder<{ co: CheckoutOutputs }>  (no id → no expansion)
      .add(output => ...)                        → StepBuilder<{ co: CheckoutOutputs }>  (output.co.ref ✅)
    )                                            → Job<{ co: CheckoutOutputs }>
  → .outputs(output => ({ ref: output.co.ref })) → Job<{ co: CheckoutOutputs }, { ref: string }>
```

### Constructor-Only Config

Job-level settings are passed as the second argument to the `Job` constructor:

```typescript
new Job("ubuntu-latest", {
    permissions: { contents: "read", packages: "write" },
    needs: ["build", "test"],
    strategy: { matrix: { os: ["ubuntu-latest", "macos-latest"] } },
    if: "github.event_name == 'push'",
    environment: "production",
    concurrency: { group: "deploy", "cancel-in-progress": true },
  })
  .steps(s => s
    .add(checkout({}))
    .add({ run: "npm run deploy" })
  );
```

This keeps configuration separate from step definitions — the constructor declares *what* the job is, `steps()` declares *what it does*.

### Workflow Example

```typescript
new Workflow({ name: "CI", on: { push: {} } })
  .jobs(j => j
    .add("build",
      new Job("ubuntu-latest")
        .steps(s => s
          .add(checkout({ id: "co" }))
          .add(setupNode({ with: { "node-version": "20" } }))
          .add(output => ({ name: "Log", run: "echo " + output.co.ref }))
        )
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

### Action Output Access

Composite actions use the same `steps()` pattern:

```typescript
new Action({
    name: "Setup and Build",
    description: "Checkout, setup Node, and build",
    inputs: {
      "node-version": { description: "Node.js version", default: "20" },
    },
    outputs: {
      "build-hash": { description: "Hash of the build output" },
    },
  })
  .steps(s => s
    .add(checkout({ id: "co" }))
    .add(setupNode({
      id: "node",
      with: { "node-version": "${{ inputs.node-version }}" },
    }))
    .add(output => ({
      id: "build",
      run: "echo ref=" + output.co.ref + "\nnpm run build",
      shell: "bash",
    }))
  )
  .outputMapping(output => ({
    "build-hash": output.build.hash,
  }))
  .build("setup-and-build");
```

Using the action in a workflow via `ActionRef`:

```typescript
const setupBuild = ActionRef.from(action);

new Workflow({ name: "CI", on: { push: {} } })
  .jobs(j => j
    .add("build",
      new Job("ubuntu-latest")
        .steps(s => s
          .add(setupBuild({ id: "setup", with: { "node-version": "22" } }))
          .add(output => ({
            name: "Show hash",
            run: "echo " + output.setup["build-hash"],
          }))
        )
        .outputs(output => ({
          hash: output.setup["build-hash"],
        }))
    )
    .add("deploy", output =>
      new Job("ubuntu-latest", { needs: ["build"] })
        .steps(s => s
          .add({ run: "echo hash=" + output.build.hash })
        )
    )
  )
  .build("ci");
```

### How `output` Flows

| Scope | `output` contains | Example access |
|-------|-------------------|----------------|
| Step callback `s.add(output => ...)` | Previous step outputs (keyed by step `id`) | `output.co.ref` |
| `outputs(output => ...)` on Job/Action | All step outputs in that job/action | `output.co.ref` |
| Job callback `j.add("id", output => ...)` | Previous job outputs (keyed by job id) | `output.build.ref` |

All runtime values are GitHub Actions expression strings like `"${{ steps.co.outputs.ref }}"` or `"${{ needs.build.outputs.ref }}"`.

### Callback Variable Naming Convention

| Variable | Role | Where it appears |
|----------|------|------------------|
| `output` | Previous outputs — step outputs in `add` callbacks, job outputs in `add` callbacks | `add(output => ...)`, `outputs(output => ...)` |
| `s` | Step builder | `steps(s => s.add(...))` |
| `j` | Job builder | `jobs(j => j.add(...))` |

Internal implementation uses `_ctx` for the runtime context storage field. The `Cx` generic type parameter tracks the accumulated output types at the type level.

## Type System Changes

### 1. `ActionStep` carries `Id` type

```typescript
export interface ActionStep<O = {}, Id extends string = string> extends JobStep {
    readonly outputs: O;
    readonly id: Id;
}
```

### 2. `getAction` infers `Id` literal

```typescript
// For actions WITH outputs:
export declare function getAction(ref: 'actions/checkout@v5'): {
    <Id extends string>(config: { id: Id; with?: Inputs; ... }): ActionStep<Outputs, Id>;
    (config?: { with?: Inputs; id?: string; ... }): JobStep;
};
```

### 3. `StepBuilder<Cx>` accumulates step context

```typescript
export declare class StepBuilder<Cx = {}> {
    add<Id extends string, StepO>(step: ActionStep<StepO, Id>): StepBuilder<Cx & Record<Id, StepO>>;
    add(step: JobStep): StepBuilder<Cx>;
    add<Id extends string, StepO>(stepFn: (output: Cx) => ActionStep<StepO, Id>): StepBuilder<Cx & Record<Id, StepO>>;
    add(stepFn: (output: Cx) => JobStep): StepBuilder<Cx>;
}
```

### 4. `Job<Cx, O>` with constructor-only config

```typescript
export interface JobConfig {
    permissions?: Permissions;
    needs?: string[];
    strategy?: Strategy;
    if?: string;
    environment?: string | Environment;
    concurrency?: Concurrency;
    "timeout-minutes"?: number;
    env?: Record<string, string>;
}

export declare class Job<Cx = {}, O extends Record<string, string> = {}> {
    constructor(runner: string, config?: JobConfig);
    steps<NewCx>(callback: (s: StepBuilder<{}>) => StepBuilder<NewCx>): Job<NewCx, O>;
    outputs<T extends Record<string, string>>(outputs: T | ((output: Cx) => T)): Job<Cx, T>;
}
```

### 5. `JobBuilder<Cx>` accumulates job context

```typescript
export declare class JobBuilder<Cx = {}> {
    add<Id extends string, O extends Record<string, string>>(
        id: Id, job: Job<any, O>
    ): JobBuilder<Cx & Record<Id, O>>;
    add(id: string, job: Job | WorkflowCall): JobBuilder<Cx>;
    add<Id extends string, O extends Record<string, string>>(
        id: Id, jobFn: (output: Cx) => Job<any, O>
    ): JobBuilder<Cx & Record<Id, O>>;
    add(id: string, jobFn: (output: Cx) => Job | WorkflowCall): JobBuilder<Cx>;
}
```

### 6. `Workflow<Cx>` with `jobs()` method

```typescript
export declare class Workflow<Cx = {}> {
    constructor(config: WorkflowConfig);
    jobs<NewCx>(callback: (j: JobBuilder<{}>) => JobBuilder<NewCx>): Workflow<NewCx>;
    static fromObject(def: WorkflowDefinition, id?: string): Workflow;
    toJSON(): WorkflowDefinition;
    build(id?: string): void;
}
```

Usage:
```typescript
new Workflow({ name: "CI", on: { push: {} } })
  .jobs(j => j
    .add("build", buildJob)  // buildJob has .outputs({ ref: ... })
    .add("deploy", output =>   // output.build.ref = "${{ needs.build.outputs.ref }}"
      new Job("ubuntu-latest", { needs: ["build"] })
        .steps(s => s
          .add({ run: "echo " + output.build.ref })
        )
    )
  )
```

### 7. Runtime `_ctx` mechanism

**StepBuilder**: maintains `_ctx` for step outputs. On `add()`, if step has `id` and `outputs`, collects into `_ctx[step.id]`. Callbacks receive `_ctx` as the `output` parameter. When `steps()` returns, `_ctx` is transferred to the `Job`.

**Job/Action**: `steps()` creates a `StepBuilder`, passes it to the callback, and receives the accumulated `_ctx` back. `outputs()` callback receives this `_ctx`.

**JobBuilder**: maintains `_ctx` for job outputs. On `add(id, job)`, if `job._outputs` exists, creates `_ctx[id]` with `${{ needs.id.outputs.key }}` expressions. If argument is a function, calls with `_ctx` first.

**Workflow**: `jobs()` creates a `JobBuilder`, passes it to the callback, and receives the accumulated `_ctx` back.

`jobOutputs` function is kept as a compatibility helper.

## TypeScript Configuration

### Motivation

Currently gaji uses `.gaji.toml` (committed) and `.gaji.local.toml` (gitignored) for configuration. Since gaji already runs TypeScript through QuickJS, configuration should also be TypeScript for consistency and type safety.

### Configuration Files

| Current | New | Committed |
|---------|-----|-----------|
| `.gaji.toml` | `gaji.config.ts` | Yes |
| `.gaji.local.toml` | `gaji.config.local.ts` | No (.gitignore) |

### Configuration Examples (Before → After)

#### Before (TOML)

`.gaji.toml`:
```toml
[project]
workflows_dir = "workflows"
output_dir = ".github"
generated_dir = "generated"

[watch]
debounce_ms = 500
ignored_patterns = ["dist", "tmp"]

[build]
validate = true
format = true
cache_ttl_days = 14
```

`.gaji.local.toml`:
```toml
[github]
token = "ghp_xxxxxxxxxxxxxxxxxxxx"
api_url = "https://github.example.com"
```

#### After (TypeScript)

`gaji.config.ts`:
```typescript
import { defineConfig } from "./generated/index.js";

export default defineConfig({
    workflows: "workflows",
    output: ".github",
    generated: "generated",
    watch: {
        debounce: 500,
        ignore: ["dist", "tmp"],
    },
    build: {
        validate: true,
        format: true,
        cacheTtlDays: 14,
    },
});
```

`gaji.config.local.ts`:
```typescript
import { defineConfig } from "./generated/index.js";

export default defineConfig({
    github: {
        token: "ghp_xxxxxxxxxxxxxxxxxxxx",
        apiUrl: "https://github.example.com",
    },
});
```

Key differences:
- Type-safe with autocomplete — `defineConfig` validates all fields
- camelCase field names (`cacheTtlDays` instead of `cache_ttl_days`, `debounce` instead of `debounce_ms`)
- Same import pattern as workflow files (`from "./generated/index.js"`)
- `gaji.config.local.ts` is gitignored, same as `.gaji.local.toml` was

#### Minimal config — defaults are enough

```typescript
// gaji.config.ts — all defaults, same as having no .gaji.toml at all
import { defineConfig } from "./generated/index.js";
export default defineConfig({});
```

#### GitHub Enterprise config

```typescript
// gaji.config.ts
import { defineConfig } from "./generated/index.js";

export default defineConfig({
    github: {
        apiUrl: "https://github.example.com",
    },
});
```

### Config API

The generated `index.d.ts` exports a `defineConfig` function and `GajiConfig` interface:

```typescript
export interface GajiConfig {
    workflows?: string;       // default: "workflows"
    output?: string;          // default: ".github"
    generated?: string;       // default: "generated"
    watch?: {
        debounce?: number;    // default: 300 (ms)
        ignore?: string[];    // default: ["node_modules", ".git", "generated"]
    };
    build?: {
        validate?: boolean;   // default: true
        format?: boolean;     // default: true
        cacheTtlDays?: number; // default: 30
    };
    github?: {
        token?: string;
        apiUrl?: string;      // GitHub Enterprise
    };
}

export declare function defineConfig(config: GajiConfig): GajiConfig;
```

### Resolution Order

1. Environment variables (`GITHUB_TOKEN`)
2. `gaji.config.local.ts` (if exists)
3. `gaji.config.ts` (if exists)
4. Defaults

### How It Works

1. gaji strips types from `gaji.config.ts` with oxc (same as workflow files)
2. Executes the stripped JS in QuickJS
3. Reads the `default` export (the config object)
4. Merges with `gaji.config.local.ts` if present (same strip + execute flow)
5. Applies environment variable overrides

The `defineConfig` function is an identity function at runtime — it exists only for TypeScript autocomplete and validation.

### Migration from `.gaji.toml`

`gaji init` detects existing `.gaji.toml` and offers to migrate:
- Reads TOML config
- Generates equivalent `gaji.config.ts`
- If `.gaji.local.toml` exists, generates `gaji.config.local.ts`
- Removes old TOML files after confirmation

### `.gitignore` Update

The gitignore section generated by `gaji init` adds `gaji.config.local.ts`:

```
# gaji generated files
generated/
.gaji-cache.json
gaji.config.local.ts
```

## Implementation Steps

Tasks are ordered by difficulty and grouped by parallelism. Within each phase, all groups can run concurrently. Each phase depends on the previous phase completing.

```
Phase 1 (병렬 5그룹)           Phase 2 (순차, Phase 1과 병렬)
├─ 1A migration.rs 치환        templates.rs:
├─ 1B executor.rs 치환           2-1 BASE_TYPES (ActionStep Id)
├─ 1C integration.rs 치환        2-2 GET_ACTION_FALLBACK (Id generic)
├─ 1D docs/ 영문 치환             2-3 CLASS_DECLARATIONS  ★
└─ 1E docs/ko/ + README          2-4 RUNTIME             ★★
                                  2-5 defineConfig
                                       │
                     ┌─────────────────┤
                     ▼                 ▼
Phase 3 (병렬 6그룹)           Phase 4 (병렬 6그룹)
├─ 3A mod.rs getAction         ├─ 4A TOML→TS migration
├─ 3B 신규 테스트 3개           ├─ 4B config 테스트
├─ 3C EXAMPLE_TEMPLATE         ├─ 4C api.md 리라이트
├─ 3D CLAUDE.md                ├─ 4D writing-workflows.md
├─ 3E config.rs ★              ├─ 4E config 문서
└─ 3F init config              └─ 4F Korean 미러
```

Critical path: Phase 2 (2-3, 2-4) → Phase 3 (3B, 3E) → Phase 4

### Phase 1 — Mechanical Renames (all parallel)

No dependencies. Each group touches different files.

#### 1A. `src/init/migration.rs` — Class renames

- `generate_composite_action_ts()`: `CompositeAction` → `Action` in imports and `new` call
- `generate_javascript_action_ts()`: `JavaScriptAction` → `NodeAction` in imports and `new` call
- Update 6 test assertions for new class names

#### 1B. `src/executor.rs` — Test rename

- Update test `test_composite_action_pipeline` (~line 286): `new CompositeAction(` → `new Action(`

#### 1C. `tests/integration.rs` — Existing test renames only

- `test_composite_job_inheritance`: `extends CompositeJob` → `extends Job`
- `test_composite_action_migration_roundtrip`: `new CompositeAction(` → `new Action(`
- `test_javascript_action_migration_roundtrip`: `new JavaScriptAction(` → `new NodeAction(`

#### 1D. English doc renames

Each file is independent:
- `docs/examples/composite-action.md`: `CompositeAction` → `Action`, `CompositeJob` → `Job` in all code examples
- `docs/examples/javascript-action.md`: `JavaScriptAction` → `NodeAction`, `CallAction` → `ActionRef`
- `docs/guide/migration.md`: class names in migration output descriptions
- `docs/reference/actions.md`: `CompositeAction` → `Action`, `jobOutputs()` context mentions

#### 1E. Korean docs + README

- `docs/ko/reference/api.md`, `docs/ko/guide/writing-workflows.md`, `docs/ko/examples/composite-action.md`, `docs/ko/examples/javascript-action.md`, `docs/ko/guide/migration.md`, `docs/ko/reference/actions.md` — mirror 1D changes
- `README.md`: `CompositeAction` → `Action`, `CallAction` → `ActionRef`, `CallJob` → `WorkflowCall`

### Phase 2 — Core Templates (sequential, single file)

All in `src/generator/templates.rs`. Must be done in order since they're in the same file. Can run in parallel with Phase 1.

#### 2-1. `BASE_TYPES_TEMPLATE` — ActionStep Id generic + interface renames (easy)

Add `Id` generic to `ActionStep`:

```typescript
export interface ActionStep<O = {}, Id extends string = string> extends JobStep {
    readonly outputs: O;
    readonly id: Id;
}
```

Rename config/runs interfaces:
- `JavaScriptActionConfig` → `NodeActionConfig`
- `JavaScriptActionRuns` → `NodeActionRuns`

#### 2-2. `GET_ACTION_FALLBACK_DECL_TEMPLATE` — Id generic (easy)

Add `<Id extends string>` to the id-required overload:

```typescript
export declare function getAction<T extends string>(ref: T): {
    <Id extends string>(config: { id: Id; ... }): ActionStep<Record<string, string>, Id>;
    (config?: { ... }): JobStep;
};
```

#### 2-3. `CLASS_DECLARATIONS_TEMPLATE` — Full rewrite (moderate) ★

- `StepBuilder<Cx>` with 4 `add` overloads (callbacks use `output` parameter name)
- `Job<Cx, O>` with constructor-only config (`JobConfig`), `steps()` method, callback-aware `outputs()`
- `JobBuilder<Cx>` with 4 `add` overloads
- `Workflow<Cx>` with `jobs()` method
- Remove `CompositeJob`
- `Action<Cx>` (was `CompositeAction`) with `steps()` method
- `NodeAction` (was `JavaScriptAction`) — constructor uses `NodeActionConfig`, `NodeActionRuns`
- `WorkflowCall` (was `CallJob`)
- `ActionRef` (was `CallAction`) — `from()` accepts `Action<any> | NodeAction | DockerAction`
- `jobOutputs` accepts `Job<any, O>` (kept as compatibility helper)

#### 2-4. `JOB_WORKFLOW_RUNTIME_TEMPLATE` — Runtime logic (hard) ★★

- Add `StepBuilder` class: `add()` method with `_ctx` tracking, detects function arg → calls with `_ctx` → collects outputs
- `Job` constructor: accept `(runner, config?)`, apply config fields (permissions, needs, strategy, etc.)
- `Job.steps`: create `StepBuilder`, pass to callback, transfer `_ctx` back
- `Job.outputs`: detect function arg → call with `_ctx`
- Add `JobBuilder` class: `add()` method with `_ctx` tracking for job outputs
- `Workflow.jobs`: create `JobBuilder`, pass to callback, transfer `_ctx` back
- Remove `CompositeJob` class, remove `addStep`/`addJob` methods
- Rename `CompositeAction` → `Action` + add `steps()` with `StepBuilder`
- Rename `JavaScriptAction` → `NodeAction`
- Rename `CallJob` → `WorkflowCall`
- Rename `CallAction` → `ActionRef`

#### 2-5. Add config types to templates (easy)

Add `GajiConfig` interface and `defineConfig` function to `BASE_TYPES_TEMPLATE`:
- `GajiConfig` interface with all config fields (optional, with defaults)
- `defineConfig` identity function declaration

Add `defineConfig` runtime to `JOB_WORKFLOW_RUNTIME_TEMPLATE`:
```javascript
export function defineConfig(config) { return config; }
```

Add `defineConfig` declaration to `CLASS_DECLARATIONS_TEMPLATE`.

### Phase 3 — Depends on Phase 2 (all parallel)

Each group touches different files. All depend on Phase 2 for the new type/runtime shapes.

#### 3A. `src/generator/mod.rs` — getAction overloads + type renames (easy)

Update `getAction` overloads for actions WITH outputs:
- Add `<Id extends string>` generic on id-required call signature
- Return type becomes `ActionStep<Outputs, Id>`

Rename type imports/exports:
- `JavaScriptActionConfig` → `NodeActionConfig`
- `JavaScriptActionRuns` → `NodeActionRuns`

Update comment on line 288.

#### 3B. `tests/integration.rs` — New tests (moderate)

- Add `test_step_builder_callback_context`: `steps()` builder callback receives previous step outputs via `output` parameter
- Add `test_outputs_callback_context`: `outputs()` callback receives previous step outputs via `output` parameter
- Add `test_job_builder_callback_context`: `jobs()` builder callback receives previous job outputs via `output` parameter (replaces `jobOutputs` pattern)

#### 3C. `src/init/templates.rs` — EXAMPLE_WORKFLOW_TEMPLATE (easy)

- Update to use `steps(s => s.add(...))` and `jobs(j => j.add(...))` patterns

#### 3D. `CLAUDE.md` — Project documentation (easy)

- Update "Key Design Patterns" section: remove `CompositeJob`, rename classes
- Update "Runtime Class Hierarchy" table: apply renames, remove CompositeJob
- Update "Adding a new action type" and "Adding a new job type" sections
- Update "Configuration Files" table: `.gaji.toml` → `gaji.config.ts`, `.gaji.local.toml` → `gaji.config.local.ts`
- Update "Configuration hierarchy" line: `env vars > gaji.config.local.ts > gaji.config.ts > defaults`

#### 3E. `src/config.rs` — Replace TOML loading with TS execution (moderate) ★

- Remove `toml` dependency for config loading
- Add `load_from_ts()` that:
  1. Checks for `gaji.config.ts` (falls back to `.gaji.toml` for backward compat)
  2. Strips types with oxc
  3. Executes in QuickJS with a wrapper that captures `export default`
  4. Deserializes JSON result into `Config` struct
- Add `merge_local_ts()` for `gaji.config.local.ts`
- Keep field mapping: `workflows` → `workflows_dir`, `output` → `output_dir`, `generated` → `generated_dir`, `watch.debounce` → `watch.debounce_ms`, `build.cacheTtlDays` → `build.cache_ttl_days`
- Keep `resolve_token()` priority: env var > local config > config > None

#### 3F. `src/init/mod.rs` + `src/init/templates.rs` — Generate TS config (moderate)

- `gaji init` generates `gaji.config.ts` instead of `.gaji.toml`
- Add `GAJI_CONFIG_TEMPLATE` constant for default `gaji.config.ts` content
- Update `.gitignore` template: replace `.gaji.local.toml` with `gaji.config.local.ts`

### Phase 4 — Depends on Phase 3 (all parallel)

#### 4A. `src/init/migration.rs` — TOML → TS config migration (moderate)

Depends on 3E.

- Detect existing `.gaji.toml` during init
- Generate equivalent `gaji.config.ts` from TOML values
- Generate `gaji.config.local.ts` from `.gaji.local.toml` if present
- Prompt user before removing old TOML files

#### 4B. Config tests (moderate)

Depends on 3E + 3F.

- Unit tests in `config.rs`: parse TS config, merge local, env var precedence
- Integration test: full pipeline with `gaji.config.ts` instead of `.gaji.toml`

#### 4C. `docs/reference/api.md` — API reference rewrite (moderate)

Depends on Phase 2 (API shape).

- Add `StepBuilder` section with 4 `add` overloads
- Update `Job` section: `Cx` generic, constructor-only `JobConfig`, `steps()` method, callback `outputs`
- Add `JobBuilder` section with 4 `add` overloads
- Update `Workflow` section: `Cx` generic, `jobs()` method
- Remove `CompositeJob` section entirely
- Rename `CompositeAction` → `Action` section with `steps()` method
- Rename `JavaScriptAction` → `NodeAction` section
- Rename `CallJob` → `WorkflowCall` section
- Rename `CallAction` → `ActionRef` section, update `from()` signature
- Update `jobOutputs()` section: note it's a compatibility helper, show `jobs()` callback as primary pattern
- Add examples for each changed class showing callback/context usage

#### 4D. `docs/guide/writing-workflows.md` — Guide rewrite (moderate)

Depends on Phase 2 (API shape).

- Update "Steps" section: show `steps(s => s.add(...))` pattern with direct and callback forms
- Replace `CompositeJob` section: show extending `Job` directly (CompositeJob removed)
- Update `CallJob` → `WorkflowCall` in reusable workflow section
- Rewrite "Outputs" section: show `outputs(output => ...)` and `jobs(j => j.add("id", output => ...))` as primary patterns, `jobOutputs()` as compatibility helper

#### 4E. Config documentation (easy)

Depends on 3E + 3F.

- `docs/guide/writing-workflows.md`: Update configuration section with `gaji.config.ts` examples
- `docs/reference/api.md`: Add `defineConfig` and `GajiConfig` to API reference
- `docs/guide/migration.md`: Add `.gaji.toml` → `gaji.config.ts` migration instructions
- `docs/ko/` mirrors: Apply same config documentation changes to Korean docs

#### 4F. Korean doc rewrites (moderate)

Depends on 4C + 4D.

- `docs/ko/reference/api.md` — mirror 4C
- `docs/ko/guide/writing-workflows.md` — mirror 4D

### Documentation Change Examples

Below are concrete before/after examples for each major documentation change.

#### API Reference: `steps()` / `add()` section (docs/reference/api.md)

Before:
```typescript
addStep(step: JobStep): this
```
> Append a step to the job.

After:
```typescript
// Job.steps — pass steps through callback builder
steps<NewCx>(callback: (s: StepBuilder<{}>) => StepBuilder<NewCx>): Job<NewCx, O>

// StepBuilder.add — 4 overloads
add<Id extends string, StepO>(step: ActionStep<StepO, Id>): StepBuilder<Cx & Record<Id, StepO>>
add(step: JobStep): StepBuilder<Cx>
add<Id extends string, StepO>(stepFn: (output: Cx) => ActionStep<StepO, Id>): StepBuilder<Cx & Record<Id, StepO>>
add(stepFn: (output: Cx) => JobStep): StepBuilder<Cx>
```

Example:
```typescript
const checkout = getAction("actions/checkout@v5");

new Job("ubuntu-latest")
  .steps(s => s
    .add(checkout({ id: "co" }))
    .add(output => ({
      name: "Use ref",
      run: "echo " + output.co.ref,   // "${{ steps.co.outputs.ref }}"
    }))
  )
  .outputs(output => ({
    ref: output.co.ref,              // "${{ steps.co.outputs.ref }}"
  }));
```

#### Outputs section (docs/guide/writing-workflows.md)

Before:
```typescript
const checkout = getAction("actions/checkout@v5");
const step = checkout({ id: "co" });

const build = new Job("ubuntu-latest")
  .addStep(step)
  .outputs({ ref: step.outputs.ref });

const buildOutputs = jobOutputs("build", build);

const deploy = new Job("ubuntu-latest")
  .needs("build")
  .addStep({ run: "echo " + buildOutputs.ref });
```

After:
```typescript
const checkout = getAction("actions/checkout@v5");

new Workflow({ name: "CI", on: { push: {} } })
  .jobs(j => j
    .add("build",
      new Job("ubuntu-latest")
        .steps(s => s
          .add(checkout({ id: "co" }))
        )
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

#### Action section (docs/examples/composite-action.md)

Before:
```typescript
import { CompositeAction, getAction } from "../generated/index.js";

const checkout = getAction("actions/checkout@v5");
const setupNode = getAction("actions/setup-node@v4");

const action = new CompositeAction({
  name: "Setup",
  description: "Setup Node.js project",
});

action
  .addStep(checkout({}))
  .addStep(setupNode({ with: { "node-version": "20" } }))
  .addStep({ run: "npm ci", shell: "bash" });

action.build("setup");
```

After:
```typescript
import { Action, getAction } from "../generated/index.js";

const checkout = getAction("actions/checkout@v5");
const setupNode = getAction("actions/setup-node@v4");

new Action({
  name: "Setup",
  description: "Setup Node.js project",
})
  .steps(s => s
    .add(checkout({}))
    .add(setupNode({ with: { "node-version": "20" } }))
    .add({ run: "npm ci", shell: "bash" })
  )
  .build("setup");
```

#### Job inheritance section (docs/examples/composite-action.md)

Before:
```typescript
import { CompositeJob, getAction, Workflow } from "../generated/index.js";

const checkout = getAction("actions/checkout@v5");

class DeployJob extends CompositeJob {
  constructor(env: string) {
    super("ubuntu-latest");
    this.addStep(checkout({})).addStep({ run: "deploy " + env });
  }
}

new Workflow({ name: "Deploy", on: { push: { branches: ["main"] } } })
  .addJob("staging", new DeployJob("staging"))
  .addJob("production", new DeployJob("production"))
  .build("deploy");
```

After:
```typescript
import { Job, getAction, Workflow } from "../generated/index.js";

const checkout = getAction("actions/checkout@v5");

class DeployJob extends Job {
  constructor(env: string) {
    super("ubuntu-latest");
    this.steps(s => s
      .add(checkout({}))
      .add({ run: "deploy " + env })
    );
  }
}

new Workflow({ name: "Deploy", on: { push: { branches: ["main"] } } })
  .jobs(j => j
    .add("staging", new DeployJob("staging"))
    .add("production", new DeployJob("production"))
  )
  .build("deploy");
```

#### NodeAction section (docs/examples/javascript-action.md)

Before:
```typescript
import { JavaScriptAction, CallAction } from "../generated/index.js";

const action = new JavaScriptAction(
  { name: "Greet", description: "Greet someone" },
  { using: "node20", main: "index.js" }
);

action.build("greet");

// Reference in a workflow
const greet = CallAction.from(action);
```

After:
```typescript
import { NodeAction, ActionRef } from "../generated/index.js";

const action = new NodeAction(
  { name: "Greet", description: "Greet someone" },
  { using: "node20", main: "index.js" }
);

action.build("greet");

// Reference in a workflow
const greet = ActionRef.from(action);
```

#### WorkflowCall section (docs/guide/writing-workflows.md)

Before:
```typescript
import { CallJob, Workflow } from "../generated/index.js";

const tests = new CallJob("org/repo/.github/workflows/test.yml@main", {
  with: { environment: "staging" },
});

new Workflow({ name: "CI", on: { push: {} } })
  .addJob("tests", tests)
  .build("ci");
```

After:
```typescript
import { WorkflowCall, Workflow } from "../generated/index.js";

const tests = new WorkflowCall("org/repo/.github/workflows/test.yml@main", {
  with: { environment: "staging" },
});

new Workflow({ name: "CI", on: { push: {} } })
  .jobs(j => j
    .add("tests", tests)
  )
  .build("ci");
```

#### Configuration section (docs/guide/writing-workflows.md or new config page)

Before:
```toml
# .gaji.toml
[project]
workflows_dir = "workflows"
output_dir = ".github"

[build]
cache_ttl_days = 14
```

After:
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

#### Migration guide update (docs/guide/migration.md)

Before:
> When migrating `action.yml` files, gaji converts composite actions to `CompositeAction` and JavaScript actions to `JavaScriptAction`.

After:
> When migrating `action.yml` files, gaji converts composite actions to `Action` and JavaScript actions to `NodeAction`.

Before:
```typescript
// Generated from composite action.yml
import { CompositeAction, getAction } from "../generated/index.js";
const action = new CompositeAction({ name: "...", description: "..." });
```

After:
```typescript
// Generated from composite action.yml
import { Action, getAction } from "../generated/index.js";
const action = new Action({ name: "...", description: "..." });
```

## Files Modified

| File | Changes |
|------|---------|
| `src/generator/templates.rs` | All 4 template constants updated |
| `src/generator/mod.rs` | getAction overload generation, comment |
| `src/init/migration.rs` | Class name renames in codegen + 6 test assertions |
| `src/config.rs` | Replace TOML loading with TS config execution |
| `src/executor.rs` | 1 test: class name rename |
| `tests/integration.rs` | 3 existing tests updated + 3 new tests + config tests |
| `CLAUDE.md` | Class hierarchy, design patterns sections |
| `README.md` | Code examples with new class names |
| `ROADMAP.md` | Replaced with this plan |
| `docs/reference/api.md` | Full API reference rewrite for all renamed classes + context patterns |
| `docs/guide/writing-workflows.md` | Outputs section, CompositeJob section, CallJob section |
| `docs/examples/composite-action.md` | All code examples |
| `docs/examples/javascript-action.md` | All code examples |
| `docs/guide/migration.md` | Migration output class names |
| `docs/reference/actions.md` | Minor class name updates |
| `docs/ko/*` (6 files) | Korean mirrors of all English doc changes |

## Files NOT Modified

- `src/builder.rs`, `src/main.rs`, `src/cli.rs`, etc.

## Key Design Decisions

1. **`steps()` returns `Job<NewCx, O>` not `this`**: Subclass typing is lost, but acceptable since `CompositeJob` is removed. Inheritance still works — `this.steps()` in a constructor mutates internal state.
2. **Overload ordering**: ActionStep overloads before JobStep overloads so TypeScript picks the more specific one.
3. **`_ctx` not serialized**: `toJSON()` explicitly builds output from named fields; `_ctx` never leaks.
4. **Existing `getAction` output population unchanged**: `GET_ACTION_RUNTIME_TEMPLATE` already populates `step.outputs` with expression strings when `id` is present. The `_ctx` mechanism just collects these.
5. **`typeof stepOrFn === 'function'`**: Works in QuickJS for both arrow and regular functions. Action step objects are never functions, so no false positives.
6. **`JobBuilder.add` expands `Workflow<Cx>`**: When a job has typed outputs, context expands. When no outputs (`O = {}`), context gets `Record<Id, {}>` — accessing any property on `output.jobId` is a type error since `{}` has no properties. This is correct behavior.
7. **`JobBuilder._ctx` collects job outputs**: The runtime `add` checks `job._outputs` (same field `jobOutputs` reads). If present, creates expression map `{ key: "${{ needs.id.outputs.key }}" }` in `_ctx[id]`. This is identical to what `jobOutputs()` does, just integrated into the builder.
8. **Constructor-only config**: Job-level settings (permissions, needs, strategy, etc.) are passed through the `Job` constructor only. No builder methods for config — this keeps configuration separate from step definitions and avoids the multiple-call ambiguity problem.
9. **`defineConfig` is an identity function**: At runtime it just returns its argument. It exists solely for TypeScript type checking and autocomplete.
10. **TS config uses same execution pipeline**: Config files go through the same oxc strip + QuickJS execute path as workflow files. No new runtime dependency needed.
11. **Backward compatibility**: If `gaji.config.ts` is not found, gaji falls back to `.gaji.toml` for existing projects that haven't migrated.

## Verification

```bash
cargo test --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all --check
```

The 3 new integration tests validate the core callback/context behavior end-to-end through QuickJS execution:
- `test_step_builder_callback_context`: step-level context (`output` has previous step outputs)
- `test_outputs_callback_context`: `outputs()` callback with step context via `output`
- `test_job_builder_callback_context`: workflow-level context (`output` has previous job outputs, replacing `jobOutputs` pattern)
