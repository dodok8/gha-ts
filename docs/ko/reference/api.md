# TypeScript API 레퍼런스

gaji의 TypeScript API에 대한 레퍼런스입니다.

## 핵심 클래스

### `Workflow`

GitHub Actions 워크플로우를 표현합니다.

```typescript
class Workflow<Cx = {}> {
  constructor(config: WorkflowConfig)
  jobs<NewCx>(callback: (j: JobBuilder<{}>) => JobBuilder<NewCx>): Workflow<NewCx>
  static fromObject(def: WorkflowDefinition, id?: string): Workflow
  build(filename?: string): void
  toJSON(): WorkflowDefinition
}
```

| 메서드 | 설명 |
|--------|------|
| `jobs(callback)` | `JobBuilder` 콜백을 통해 워크플로우 job을 정의합니다. 콜백은 빈 `JobBuilder`를 받아 `.add()`로 job을 추가한 뒤 반환합니다. |
| `fromObject(def, id?)` | `WorkflowDefinition` 객체로부터 Workflow를 생성합니다. 기존 YAML 형태의 정의를 래핑할 때 유용합니다. |
| `build(filename?)` | 워크플로우를 YAML로 컴파일합니다. |
| `toJSON()` | `WorkflowDefinition` 객체로 직렬화합니다. |

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

#### 예제

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

#### `Workflow.fromObject()` 예제

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

워크플로우의 작업을 나타냅니다. 두 개의 타입 파라미터를 가집니다: `Cx`는 `.steps()`에서 누적된 스텝 출력 컨텍스트를, `O`는 `.outputs()`에서 선언된 출력 키를 추적합니다.

```typescript
class Job<Cx = {}, O extends Record<string, string> = {}> {
  constructor(runsOn: string | string[], config?: JobConfig)
  steps<NewCx>(callback: (s: StepBuilder<{}>) => StepBuilder<NewCx>): Job<NewCx, O>
  outputs<T extends Record<string, string>>(outputs: T | ((output: Cx) => T)): Job<Cx, T>
  toJSON(): JobDefinition
}
```

| 메서드 | 설명 |
|--------|------|
| `steps(callback)` | `StepBuilder` 콜백을 통해 스텝을 정의합니다. 콜백은 빈 `StepBuilder`를 받아 `.add()`로 스텝을 추가한 뒤 반환합니다. |
| `outputs(outputs)` | job 출력을 정의합니다. 일반 객체 또는 스텝 출력 컨텍스트(`Cx`)를 받는 콜백을 받습니다. |
| `toJSON()` | `JobDefinition` 객체로 직렬화합니다. |

#### `JobConfig`

모든 job 설정은 생성자의 `config` 파라미터로 전달합니다:

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

#### 예제

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

`.steps()` 콜백 내에서 스텝을 누적합니다. `.add()` 호출마다 스텝을 추가하고, `id`와 타입이 지정된 출력이 있는 스텝의 경우 출력 컨텍스트 `Cx`를 갱신합니다.

```typescript
class StepBuilder<Cx = {}> {
  add<Id extends string, StepO>(step: ActionStep<StepO, Id>): StepBuilder<Cx & Record<Id, StepO>>
  add(step: JobStep): StepBuilder<Cx>
  add<Id extends string, StepO>(stepFn: (output: Cx) => ActionStep<StepO, Id>): StepBuilder<Cx & Record<Id, StepO>>
  add(stepFn: (output: Cx) => JobStep): StepBuilder<Cx>
}
```

네 가지 오버로드:

| 오버로드 | 설명 |
|----------|------|
| `add(actionStep)` | 타입이 지정된 출력이 있는 `ActionStep`을 추가합니다 (`getAction()`에 `id`를 전달하여 반환). `Cx`에 출력을 병합합니다. |
| `add(jobStep)` | 일반 `JobStep`을 추가합니다 (run 명령 또는 `id` 없는 액션). `Cx` 변경 없음. |
| `add(output => actionStep)` | 콜백 형태 — 이전 스텝 출력(`Cx`)을 받아 `ActionStep`을 반환합니다. |
| `add(output => jobStep)` | 콜백 형태 — 이전 스텝 출력(`Cx`)을 받아 `JobStep`을 반환합니다. |

#### 예제

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

`.jobs()` 콜백 내에서 job을 누적합니다. `.add()` 호출마다 job을 등록하고, 출력이 선언된 job의 경우 출력 컨텍스트 `Cx`를 갱신합니다.

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

네 가지 오버로드:

| 오버로드 | 설명 |
|----------|------|
| `add(id, job)` | 출력이 있는 `Job`을 추가합니다. `Cx`에 출력을 병합합니다. |
| `add(id, job)` | 출력 추적 없이 `Job` 또는 `WorkflowCall`을 추가합니다. |
| `add(id, output => job)` | 콜백 형태 — 이전 job 출력(`Cx`)을 받아 `Job`을 반환합니다. |
| `add(id, output => job)` | 콜백 형태 — 이전 job 출력(`Cx`)을 받아 `Job` 또는 `WorkflowCall`을 반환합니다. |

#### 예제

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

재사용 가능한 [컴포지트 액션](https://docs.github.com/en/actions/sharing-automations/creating-actions/creating-a-composite-action)을 만듭니다.

```typescript
class Action<Cx = {}> {
  constructor(config: { name: string; description: string; inputs?: Record<string, unknown>; outputs?: Record<string, unknown> })
  steps<NewCx>(callback: (s: StepBuilder<{}>) => StepBuilder<NewCx>): Action<NewCx>
  outputMapping<T extends Record<string, string>>(mapping: (output: Cx) => T): Action<Cx>
  build(filename: string): void
  toJSON(): object
}
```

| 메서드 | 설명 |
|--------|------|
| `steps(callback)` | `StepBuilder` 콜백을 통해 액션 스텝을 정의합니다. |
| `outputMapping(fn)` | 스텝 출력을 액션 출력에 매핑합니다. |
| `build(filename)` | 액션을 `action.yml`로 컴파일합니다. |

#### 예제

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

생성된 `action.yml`은 다음과 같이 사용할 수 있습니다:

```typescript
// 다른 워크플로우에서
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

[Node.js 기반 GitHub Actions](https://docs.github.com/en/actions/sharing-automations/creating-actions/creating-a-javascript-action)를 만듭니다.

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

#### 예제

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

`.github/actions/hello-world/action.yml`이 생성됩니다. [`ActionRef.from()`](#actionref)으로 워크플로우에서 참조할 수 있습니다.

전체 예제는 [NodeAction 예제](/ko/examples/javascript-action)를 참조하세요.

---

### `DockerAction`

[Docker 컨테이너 액션](https://docs.github.com/en/actions/sharing-automations/creating-actions/creating-a-docker-container-action)을 만듭니다.

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

#### 예제

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

`.github/actions/greeting/action.yml`이 생성됩니다.

Docker Hub 이미지를 직접 사용하려면 `image`에 `docker://` 접두사를 붙입니다:

```typescript
{
  using: "docker",
  image: "docker://alpine:3.19",
}
```

---

### Job 상속

TypeScript 클래스 상속을 통해 재사용 가능한 작업 템플릿을 만듭니다. `Job`을 직접 상속하세요.

::: tip
일회성 job에는 `Job`을 직접 사용하세요. 공통 패턴을 파라미터와 함께 캡슐화한 **재사용 가능한 클래스**를 만들 때 `Job`을 상속하세요.
:::

#### 예제

```ts twoslash
// @filename: workflows/example.ts
// ---cut---
import { Job, getAction, Workflow } from "../generated/index.js";

const checkout = getAction("actions/checkout@v5");
const setupNode = getAction("actions/setup-node@v4");

// 재사용 가능한 작업 템플릿 정의
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

// 워크플로우에서 사용
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

더 복잡한 재사용 가능한 작업도 만들 수 있습니다:

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

// 워크플로우에서 사용
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

다른 리포지토리나 파일에 정의된 [재사용 가능한 워크플로우](https://docs.github.com/en/actions/using-workflows/reusing-workflows)를 호출합니다. `Job`과 달리 `WorkflowCall`은 `steps`가 없으며, `uses`를 통해 참조된 워크플로우에 위임합니다.

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

모든 옵션은 생성자의 `config` 파라미터로 전달합니다.

#### 예제

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

생성되는 YAML:

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

gaji로 빌드한 로컬 composite 또는 JavaScript 액션을 job의 스텝으로 참조합니다.

```typescript
class ActionRef {
  constructor(uses: string)
  static from(action: Action | NodeAction | DockerAction): ActionRef
  toJSON(): Step
}
```

| 메서드 | 설명 |
|--------|------|
| `from(action)` | `Action`, `NodeAction`, `DockerAction` 인스턴스로부터 `ActionRef`를 생성합니다. `.github/actions/<id>` 경로를 자동으로 해석합니다. |

#### 예제

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

## 함수

### `getAction()`

타입이 지정된 액션 함수를 가져옵니다. 출력이 정의된 액션의 경우, `getAction()`은 두 가지 오버로드를 가진 callable을 반환합니다:

- `id` 필수: 타입이 지정된 출력 속성이 있는 `ActionStep<Outputs>` 반환
- `id` 선택: `JobStep` 반환

```typescript
// 출력이 있는 액션 (예: actions/checkout@v5)
function getAction(ref: 'actions/checkout@v5'): {
  (config: { id: string; with?: Inputs; ... }): ActionStep<Outputs>
  (config?: { id?: string; with?: Inputs; ... }): JobStep
}

// 출력이 없는 액션
function getAction(ref: 'actions/setup-node@v4'):
  (config?: { with?: Inputs; ... }) => JobStep

// 알 수 없는 액션에 대한 폴백
function getAction<T extends string>(ref: T): {
  (config: { id: string; ... }): ActionStep<Record<string, string>>
  (config?: { ... }): JobStep
}
```

#### 예제

```typescript
const checkout = getAction("actions/checkout@v5");

// 완전한 타입 안전성으로 사용
const step = checkout({
  with: {
    repository: "owner/repo",
    ref: "main",
    "fetch-depth": 0,
  },
});

// 타입이 지정된 스텝 출력 (id 필수)
const checkoutStep = checkout({ id: "my-checkout" });
// checkoutStep.outputs.ref → "${{ steps.my-checkout.outputs.ref }}"
```

전체 typed outputs 예제는 [워크플로우 작성의 출력 섹션](../guide/writing-workflows.md#출력)을 참조하세요.

---

### `jobOutputs()`

다운스트림 job에서 사용할 타입이 지정된 job 출력 참조를 생성합니다. `Job` 객체의 `.outputs()` 호출에서 출력 키를 읽어 <code v-pre>${{ needs.&lt;jobId&gt;.outputs.&lt;key&gt; }}</code> 표현식을 생성합니다.

이 함수는 호환성 헬퍼입니다. 기본 패턴은 `.jobs()` 콜백을 사용하는 것이며, 여기서 job 출력 컨텍스트가 자동으로 전달됩니다.

```typescript
function jobOutputs<O extends Record<string, string>>(
  jobId: string,
  job: Job<any, O>,
): JobOutputs<O>
```

#### 예제

```typescript
const buildOutputs = jobOutputs("build", build);
// buildOutputs.ref → "${{ needs.build.outputs.ref }}"
// buildOutputs.sha → "${{ needs.build.outputs.sha }}"
```

전체 예제는 [워크플로우 작성의 출력 섹션](../guide/writing-workflows.md#출력)을 참조하세요.

---

### `defineConfig()`

`gaji.config.ts`용 타입 안전한 설정 헬퍼입니다.

```typescript
function defineConfig(config: GajiConfig): GajiConfig
```

#### `GajiConfig`

```typescript
interface GajiConfig {
  workflows?: string        // 기본값: "workflows"
  output?: string           // 기본값: ".github"
  generated?: string        // 기본값: "generated"
  watch?: {
    debounce?: number       // 기본값: 300 (ms)
    ignore?: string[]       // 기본값: ["node_modules", ".git", "generated"]
  }
  build?: {
    validate?: boolean      // 기본값: true
    format?: boolean        // 기본값: true
    cacheTtlDays?: number   // 기본값: 30
  }
  github?: {
    token?: string
    apiUrl?: string         // GitHub Enterprise용
  }
}
```

#### 예제

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

## 타입 정의

### `JobStep`

워크플로우 스텝입니다.

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

`id`를 제공했을 때 `getAction()`이 반환하는 스텝입니다. `JobStep`을 확장하여 타입이 지정된 출력 접근을 제공합니다.

```typescript
interface ActionStep<O = {}, Id extends string = string> extends JobStep {
  readonly outputs: O
  readonly id: Id
}
```

출력 속성은 표현식 문자열로 해석됩니다:

```typescript
const step = checkout({ id: "co" });
step.outputs.ref  // "${{ steps.co.outputs.ref }}"
```

### `Step`

스텝 타입의 유니온입니다.

```typescript
type Step = JobStep | ActionStep<any>
```

### `JobOutputs<T>`

타입이 지정된 job 출력 참조를 위한 매핑 타입입니다. 각 키는 <code v-pre>${{ needs.&lt;jobId&gt;.outputs.&lt;key&gt; }}</code> 표현식으로 해석됩니다.

```typescript
type JobOutputs<T extends Record<string, string>> = {
  readonly [K in keyof T]: string
}
```

### `WorkflowTriggers`

워크플로우 트리거 이벤트입니다.

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

작업 매트릭스 전략입니다.

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

액션 입력 정의 (Action용)입니다.

```typescript
interface ActionInput {
  description: string
  required?: boolean
  default?: string
}
```

### `ActionOutput`

액션 출력 정의 (Action용)입니다.

```typescript
interface ActionOutput {
  description: string
  value: string
}
```

## 전체 예제

### 완전한 워크플로우

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

## 다음 단계

- [예제](/ko/examples/simple-ci) 보기
- [액션](./actions.md)에 대해 알아보기
