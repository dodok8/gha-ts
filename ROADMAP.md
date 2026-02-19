# gaji 1.0 Plan

## Context

gaji 1.0 refines the TypeScript API to provide type-safe output access through a builder pattern with context tracking at two levels:

1. **Step level**: `Job<Cx>` accumulates step output types. `addStep` callbacks receive a typed context with previous step outputs.
2. **Job level**: `Workflow<Cx>` accumulates job output types. `addJob` callbacks receive a typed context with previous job outputs (replacing the standalone `jobOutputs` function pattern).

The current API requires storing step/job references in variables to access outputs; the new API accumulates output types in `Cx` generic parameters so callbacks can access previous outputs through typed context objects. Additionally, several class names are simplified, and `CompositeJob` (which is identical to `Job`) is removed.

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
| `jobOutputs` | `jobOutputs` | Keep as compatibility helper; primary pattern moves to `Workflow.addJob` context |

> Note: Config/runs interfaces (`JavaScriptActionConfig`, `JavaScriptActionRuns`, etc.) are kept as-is. They describe the underlying GitHub Actions metadata format, not user-facing concepts.

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
  .addJob("build",
    new Job("ubuntu-latest")
      .addStep(checkout({ id: "co" }))
      .addStep((cx) => ({ name: "Use ref", run: "echo " + cx.co.ref }))
      .outputs((cx) => ({ ref: cx.co.ref, sha: cx.co.commit }))
  )
  .addJob("deploy", (cx) =>
    new Job("ubuntu-latest")
      .needs("build")
      .addStep({ name: "Deploy", run: "echo " + cx.build.ref })
  )
  .build("ci");
```

Key differences:
- No need to store step/job in variables just to reference outputs
- `addStep((cx) => ...)` callback receives accumulated step outputs via `cx`
- `outputs((cx) => ...)` callback can also access step outputs
- `addJob("id", (cx) => ...)` callback receives accumulated job outputs via `cx`
- Type error if accessing `cx.co` when step has no `id`, or `cx.build.nonExistent` when output not defined

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

### Existing workflows — No changes needed
```typescript
// workflows/ci.ts — unchanged, still works
import { getAction, Job, Workflow } from "../generated/index.js";

const checkout = getAction("actions/checkout@v5");
const fmt = new Job("ubuntu-latest")
  .addStep(checkout({}))
  .addStep({ run: "cargo fmt --all --check" });

new Workflow({ name: "PR", on: { pull_request: { branches: ["main"] } } })
  .addJob("fmt", fmt)
  .build("pr");
```

### Type error examples
```typescript
const checkout = getAction("actions/checkout@v5");

new Job("ubuntu-latest")
  .addStep(checkout({}))  // no id → no context expansion
  .addStep((cx) => ({
    run: cx.co.ref  // ❌ Type error: Property 'co' does not exist on type '{}'
  }));

new Job("ubuntu-latest")
  .addStep(checkout({ id: "co" }))  // id "co" → context expands with checkout outputs
  .addStep((cx) => ({
    run: cx.co.ref      // ✅ OK: cx.co has type CheckoutOutputs
  }));
```

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

### 3. `Job<Cx, O>` tracks step context

```typescript
export declare class Job<Cx = {}, O extends Record<string, string> = {}> {
    addStep<Id extends string, StepO>(step: ActionStep<StepO, Id>): Job<Cx & Record<Id, StepO>, O>;
    addStep(step: JobStep): Job<Cx, O>;
    addStep<Id extends string, StepO>(stepFn: (cx: Cx) => ActionStep<StepO, Id>): Job<Cx & Record<Id, StepO>, O>;
    addStep(stepFn: (cx: Cx) => JobStep): Job<Cx, O>;
    outputs<T extends Record<string, string>>(outputs: T | ((cx: Cx) => T)): Job<Cx, T>;
    // ... other methods return this
}
```

### 4. `Workflow<Cx>` tracks job context

Same pattern as `Job`, but at the workflow level:

```typescript
export declare class Workflow<Cx = {}> {
    constructor(config: WorkflowConfig);

    // Job with typed outputs → expands context
    addJob<Id extends string, O extends Record<string, string>>(
        id: Id, job: Job<any, O>
    ): Workflow<Cx & Record<Id, JobOutputs<O>>>;
    // WorkflowCall or Job without specific outputs → preserves context
    addJob(id: string, job: Job<any, any> | WorkflowCall): Workflow<Cx>;
    // Callback returning Job with typed outputs → expands context
    addJob<Id extends string, O extends Record<string, string>>(
        id: Id, jobFn: (cx: Cx) => Job<any, O>
    ): Workflow<Cx & Record<Id, JobOutputs<O>>>;
    // Callback returning any job → preserves context
    addJob(id: string, jobFn: (cx: Cx) => Job<any, any> | WorkflowCall): Workflow<Cx>;

    static fromObject(def: WorkflowDefinition, id?: string): Workflow;
    toJSON(): WorkflowDefinition;
    build(id?: string): void;
}
```

Usage:
```typescript
new Workflow({ name: "CI", on: { push: {} } })
  .addJob("build", buildJob)  // buildJob has .outputs({ ref: ... })
  .addJob("deploy", (cx) =>   // cx.build.ref = "${{ needs.build.outputs.ref }}"
    new Job("ubuntu-latest")
      .needs("build")
      .addStep({ run: "echo " + cx.build.ref })
  )
```

### 5. Runtime `_cx` mechanism

**Job/Action**: maintain `_cx` for step outputs. On `addStep`, if step has `id` and `outputs`, collect into `_cx[step.id]`. Callbacks receive `_cx`. `outputs()` also accepts a callback.

**Workflow**: maintain `_cx` for job outputs. On `addJob(id, job)`, if `job._outputs` exists, create `_cx[id]` with `${{ needs.id.outputs.key }}` expressions (same logic as current `jobOutputs` function). If argument is a function, call with `_cx` first.

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

### Step 1: `src/generator/templates.rs` — BASE_TYPES_TEMPLATE

Add `Id` generic to `ActionStep`:

```typescript
export interface ActionStep<O = {}, Id extends string = string> extends JobStep {
    readonly outputs: O;
    readonly id: Id;
}
```

### Step 2: `src/generator/templates.rs` — GET_ACTION_FALLBACK_DECL_TEMPLATE

Add `<Id extends string>` to the id-required overload:

```typescript
export declare function getAction<T extends string>(ref: T): {
    <Id extends string>(config: { id: Id; ... }): ActionStep<Record<string, string>, Id>;
    (config?: { ... }): JobStep;
};
```

### Step 3: `src/generator/templates.rs` — CLASS_DECLARATIONS_TEMPLATE

Full rewrite:
- `Job<Cx, O>` with 4 addStep overloads + callback-aware `outputs`
- Remove `CompositeJob`
- `Workflow<Cx>` with 4 addJob overloads (context tracks job outputs as `JobOutputs<O>` expressions)
- `Action<Cx>` (was `CompositeAction`) with 4 addStep overloads
- `NodeAction` (was `JavaScriptAction`)
- `WorkflowCall` (was `CallJob`)
- `ActionRef` (was `CallAction`) — `from()` accepts `Action<any> | NodeAction | DockerAction`
- `jobOutputs` accepts `Job<any, O>` (kept as compatibility helper)

### Step 4: `src/generator/templates.rs` — JOB_WORKFLOW_RUNTIME_TEMPLATE

- `Job` constructor: add `this._cx = {}`
- `Job.addStep`: detect function arg → call with `_cx` → collect outputs into `_cx`
- `Job.outputs`: detect function arg → call with `_cx`
- `Workflow` constructor: add `this._cx = {}`
- `Workflow.addJob`: detect function arg → call with `_cx` → if job has `_outputs`, populate `_cx[id]` with `${{ needs.id.outputs.key }}` expressions
- Remove `CompositeJob` class
- Rename `CompositeAction` → `Action` + add `_cx`/callback support to `addStep`
- Rename `JavaScriptAction` → `NodeAction`
- Rename `CallJob` → `WorkflowCall`
- Rename `CallAction` → `ActionRef`

### Step 5: `src/generator/mod.rs` — generate_index_dts()

Update `getAction` overloads for actions WITH outputs:
- Add `<Id extends string>` generic on id-required call signature
- Return type becomes `ActionStep<Outputs, Id>`

Update comment on line 288.

### Step 6: `src/init/migration.rs`

- `generate_composite_action_ts()`: `CompositeAction` → `Action` in imports and `new` call
- `generate_javascript_action_ts()`: `JavaScriptAction` → `NodeAction` in imports and `new` call
- Update 6 test assertions for new class names

### Step 7: `src/executor.rs`

- Update test `test_composite_action_pipeline` (~line 286): `new CompositeAction(` → `new Action(`

### Step 8: `tests/integration.rs`

- `test_composite_job_inheritance`: `extends CompositeJob` → `extends Job`
- `test_composite_action_migration_roundtrip`: `new CompositeAction(` → `new Action(`
- `test_javascript_action_migration_roundtrip`: `new JavaScriptAction(` → `new NodeAction(`
- Add `test_addstep_callback_context`: step callback receives `cx` with previous step outputs
- Add `test_outputs_callback_context`: `outputs()` callback receives `cx`
- Add `test_addjob_callback_context`: workflow `addJob` callback receives `cx` with previous job outputs (replaces `jobOutputs` pattern)

### Step 10: TypeScript Configuration

#### 10a. `src/generator/templates.rs` — Add config types

Add `GajiConfig` interface and `defineConfig` function to `BASE_TYPES_TEMPLATE`:
- `GajiConfig` interface with all config fields (optional, with defaults)
- `defineConfig` identity function declaration

Add `defineConfig` runtime to `JOB_WORKFLOW_RUNTIME_TEMPLATE`:
```javascript
export function defineConfig(config) { return config; }
```

Add `defineConfig` declaration to `CLASS_DECLARATIONS_TEMPLATE`.

#### 10b. `src/config.rs` — Replace TOML loading with TS execution

- Remove `toml` dependency for config loading
- Add `load_from_ts()` that:
  1. Checks for `gaji.config.ts` (falls back to `.gaji.toml` for backward compat)
  2. Strips types with oxc
  3. Executes in QuickJS with a wrapper that captures `export default`
  4. Deserializes JSON result into `Config` struct
- Add `merge_local_ts()` for `gaji.config.local.ts`
- Keep field mapping: `workflows` → `workflows_dir`, `output` → `output_dir`, `generated` → `generated_dir`, `watch.debounce` → `watch.debounce_ms`, `build.cacheTtlDays` → `build.cache_ttl_days`
- Keep `resolve_token()` priority: env var > local config > config > None

#### 10c. `src/init/mod.rs` + `src/init/templates.rs` — Generate TS config

- `gaji init` generates `gaji.config.ts` instead of `.gaji.toml`
- Add `GAJI_CONFIG_TEMPLATE` constant for default `gaji.config.ts` content
- Update `.gitignore` template: replace `.gaji.local.toml` with `gaji.config.local.ts`

#### 10d. `src/init/migration.rs` — TOML → TS config migration

- Detect existing `.gaji.toml` during init
- Generate equivalent `gaji.config.ts` from TOML values
- Generate `gaji.config.local.ts` from `.gaji.local.toml` if present
- Prompt user before removing old TOML files

#### 10e. Tests

- Unit tests in `config.rs`: parse TS config, merge local, env var precedence
- Integration test: full pipeline with `gaji.config.ts` instead of `.gaji.toml`

### Step 9: Documentation and Examples

#### 9a. `CLAUDE.md` — Update project documentation
- Update "Key Design Patterns" section: remove `CompositeJob`, rename classes
- Update "Runtime Class Hierarchy" table: apply renames, remove CompositeJob
- Update "Adding a new action type" and "Adding a new job type" sections
- Update "Configuration Files" table: `.gaji.toml` → `gaji.config.ts`, `.gaji.local.toml` → `gaji.config.local.ts`
- Update "Configuration hierarchy" line: `env vars > gaji.config.local.ts > gaji.config.ts > defaults`

#### 9b. `src/init/templates.rs` — `EXAMPLE_WORKFLOW_TEMPLATE`
- No class name changes needed (only uses `Job`, `Workflow`, `getAction`)
- Optionally add a comment showing the callback pattern as an alternative

#### 9c. `docs/reference/api.md` (English API Reference) — Major rewrite
- Update `Workflow` section: `addJob` now accepts `Job<any, any> | WorkflowCall`, add `Cx` generic, add 4 addJob overloads, add callback examples
- Update `Job` section: add `Cx` generic, 4 addStep overloads, callback `outputs`, add context examples
- Remove `CompositeJob` section entirely
- Rename `CompositeAction` → `Action` section with `Cx` generic and 4 addStep overloads
- Rename `JavaScriptAction` → `NodeAction` section
- Rename `CallJob` → `WorkflowCall` section
- Rename `CallAction` → `ActionRef` section, update `from()` signature
- Update `jobOutputs()` section: note it's a compatibility helper, show `addJob` callback as primary pattern
- Add examples for each changed class showing callback/context usage

#### 9d. `docs/guide/writing-workflows.md` (English Guide) — Section rewrites
- Update "Steps" section: show both direct and callback `addStep` patterns
- Replace `CompositeJob` section: show extending `Job` directly (CompositeJob removed)
- Update `CallJob` → `WorkflowCall` in reusable workflow section
- Rewrite "Outputs" section: show `outputs((cx) => ...)` and `addJob("id", (cx) => ...)` as primary patterns, `jobOutputs()` as compatibility helper

#### 9e. `docs/examples/composite-action.md` (English Examples)
- Rename `CompositeAction` → `Action` in all code examples
- Replace `CompositeJob` examples with `Job` inheritance (extend `Job` directly)
- Show callback patterns for step output access

#### 9f. `docs/examples/javascript-action.md` (English Examples)
- Rename `JavaScriptAction` → `NodeAction` in all code examples
- Rename `CallAction` → `ActionRef` in all code examples

#### 9g. `docs/guide/migration.md` (English Migration Guide)
- Update: actions converted to `Action` (not `CompositeAction`), `NodeAction` (not `JavaScriptAction`), `DockerAction`
- Update code examples with new class names

#### 9h. `docs/reference/actions.md` (English Actions Reference)
- Update mentions of `CompositeAction` → `Action`, `jobOutputs()` context

#### 9i. Korean documentation (`docs/ko/`) — Mirror all English changes
- `docs/ko/reference/api.md`
- `docs/ko/guide/writing-workflows.md`
- `docs/ko/examples/composite-action.md`
- `docs/ko/examples/javascript-action.md`
- `docs/ko/guide/migration.md`
- `docs/ko/reference/actions.md`

#### 9j. `README.md`
- Update `CompositeAction` → `Action`, `CallAction` → `ActionRef` in examples
- Update `CallJob` → `WorkflowCall` in reusable workflow example
- Update configuration section: show `gaji.config.ts` instead of `.gaji.toml`

#### 9k. Documentation — TypeScript Configuration
- `docs/guide/writing-workflows.md`: Update configuration section with `gaji.config.ts` examples
- `docs/reference/api.md`: Add `defineConfig` and `GajiConfig` to API reference
- `docs/guide/migration.md`: Add `.gaji.toml` → `gaji.config.ts` migration instructions
- `docs/ko/` mirrors: Apply same config documentation changes to Korean docs

### Documentation Change Examples

Below are concrete before/after examples for each major documentation change.

#### API Reference: `addStep` section (docs/reference/api.md)

Before:
```typescript
addStep(step: JobStep): this
```
> Append a step to the job.

After:
```typescript
// Direct step — no context change
addStep(step: JobStep): Job<Cx, O>
// Action step with id — expands context with step outputs
addStep<Id extends string, StepO>(step: ActionStep<StepO, Id>): Job<Cx & Record<Id, StepO>, O>
// Callback — access previous step outputs via cx
addStep(stepFn: (cx: Cx) => JobStep): Job<Cx, O>
addStep<Id extends string, StepO>(stepFn: (cx: Cx) => ActionStep<StepO, Id>): Job<Cx & Record<Id, StepO>, O>
```

Example:
```typescript
const checkout = getAction("actions/checkout@v5");

new Job("ubuntu-latest")
  .addStep(checkout({ id: "co" }))
  .addStep((cx) => ({
    name: "Use ref",
    run: "echo " + cx.co.ref,   // "${{ steps.co.outputs.ref }}"
  }))
  .outputs((cx) => ({
    ref: cx.co.ref,              // "${{ steps.co.outputs.ref }}"
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
  .addJob("build",
    new Job("ubuntu-latest")
      .addStep(checkout({ id: "co" }))
      .outputs((cx) => ({ ref: cx.co.ref }))
  )
  .addJob("deploy", (cx) =>
    new Job("ubuntu-latest")
      .needs("build")
      .addStep({ run: "echo " + cx.build.ref })
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

const action = new Action({
  name: "Setup",
  description: "Setup Node.js project",
});

action
  .addStep(checkout({}))
  .addStep(setupNode({ with: { "node-version": "20" } }))
  .addStep({ run: "npm ci", shell: "bash" });

action.build("setup");
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
    this.addStep(checkout({})).addStep({ run: "deploy " + env });
  }
}

new Workflow({ name: "Deploy", on: { push: { branches: ["main"] } } })
  .addJob("staging", new DeployJob("staging"))
  .addJob("production", new DeployJob("production"))
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
  .addJob("tests", tests)
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

- `workflows/*.ts` — only import `Job`, `Workflow`, `getAction` (unchanged)
- `src/init/templates.rs` — only uses `Job`, `Workflow`, `getAction`
- `src/builder.rs`, `src/main.rs`, `src/cli.rs`, etc.

## Key Design Decisions

1. **`addStep` returns `Job<NewCx, O>` not `this`**: Subclass typing is lost, but acceptable since `CompositeJob` is removed.
2. **Overload ordering**: ActionStep overloads before JobStep overloads so TypeScript picks the more specific one.
3. **`_cx` not serialized**: `toJSON()` explicitly builds output from named fields; `_cx` never leaks.
4. **Existing `getAction` output population unchanged**: `GET_ACTION_RUNTIME_TEMPLATE` already populates `step.outputs` with expression strings when `id` is present. The `_cx` mechanism just collects these.
5. **`typeof stepOrFn === 'function'`**: Works in QuickJS for both arrow and regular functions. Action step objects are never functions, so no false positives.
6. **`Workflow.addJob` returns `Workflow<NewCx>`**: Same pattern as `Job.addStep`. When a job has typed outputs, context expands. When no outputs (`O = {}`), context gets `Record<Id, {}>` — accessing any property on `cx.jobId` is a type error since `{}` has no properties. This is correct behavior.
7. **Workflow `_cx` collects job outputs**: The runtime `addJob` checks `job._outputs` (same field `jobOutputs` reads). If present, creates expression map `{ key: "${{ needs.id.outputs.key }}" }` in `_cx[id]`. This is identical to what `jobOutputs()` does, just integrated into the builder.
8. **`defineConfig` is an identity function**: At runtime it just returns its argument. It exists solely for TypeScript type checking and autocomplete.
9. **TS config uses same execution pipeline**: Config files go through the same oxc strip + QuickJS execute path as workflow files. No new runtime dependency needed.
10. **Backward compatibility**: If `gaji.config.ts` is not found, gaji falls back to `.gaji.toml` for existing projects that haven't migrated.

## Verification

```bash
cargo test --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all --check
```

The 3 new integration tests validate the core callback/context behavior end-to-end through QuickJS execution:
- `test_addstep_callback_context`: step-level context (cx has previous step outputs)
- `test_outputs_callback_context`: `outputs()` callback with step context
- `test_addjob_callback_context`: workflow-level context (cx has previous job outputs, replacing `jobOutputs` pattern)
