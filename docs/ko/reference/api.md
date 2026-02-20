# TypeScript API 레퍼런스

gaji의 TypeScript API에 대한 레퍼런스입니다.

## 핵심 클래스

### `Workflow`

GitHub Actions 워크플로우를 표현합니다.

```typescript
class Workflow {
  constructor(config: WorkflowConfig)
  addJob(id: string, job: Job<any> | WorkflowCall): this
  static fromObject(def: WorkflowDefinition, id?: string): Workflow
  build(filename?: string): void
  toJSON(): WorkflowDefinition
}
```

| 메서드 | 설명 |
|--------|------|
| `addJob(id, job)` | 워크플로우에 job을 추가합니다. `Job`, `WorkflowCall`을 받습니다. |
| `fromObject(def, id?)` | `WorkflowDefinition` 객체로부터 Workflow를 생성합니다. 기존 YAML 형태의 정의를 래핑할 때 유용합니다. |
| `build(filename?)` | 워크플로우를 YAML로 컴파일합니다. |
| `toJSON()` | `WorkflowDefinition` 객체로 직렬화합니다. |

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
  .addJob("test", testJob)
  .addJob("build", buildJob);

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

워크플로우의 작업을 나타냅니다. 타입 파라미터 `O`는 `jobOutputs()`를 통한 타입 안전한 job 간 참조를 위해 출력 키를 추적합니다.

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

| 메서드 | 설명 |
|--------|------|
| `addStep(step)` | job에 스텝을 추가합니다. |
| `needs(jobs)` | job 의존성을 설정합니다. |
| `env(variables)` | 환경 변수를 설정합니다. |
| `when(condition)` | job의 `if` 조건을 설정합니다 (예: `"github.ref == 'refs/heads/main'"`). |
| `permissions(perms)` | job 수준의 권한을 설정합니다 (예: `{ contents: 'read' }`). |
| `outputs(outputs)` | job 출력을 정의합니다. 출력 키를 캡처한 `Job<T>`를 반환합니다. |
| `strategy(strategy)` | 매트릭스 전략을 설정합니다. |
| `continueOnError(v)` | `continue-on-error` 플래그를 설정합니다. |
| `timeoutMinutes(m)` | `timeout-minutes` 값을 설정합니다. |
| `toJSON()` | `JobDefinition` 객체로 직렬화합니다. |

생성자의 `options` 파라미터로 모든 옵션을 한번에 설정할 수 있습니다:

```typescript
const job = new Job("ubuntu-latest", {
  needs: ["test"],
  env: { NODE_ENV: "production" },
  "timeout-minutes": 30,
});
```

#### 예제

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

### `Action`

재사용 가능한 [컴포지트 액션](https://docs.github.com/en/actions/sharing-automations/creating-actions/creating-a-composite-action)을 만듭니다.

```typescript
class Action {
  constructor(config: ActionConfig)
  addStep(step: Step): this
  build(filename: string): void
}
```

#### `ActionConfig`

```typescript
interface ActionConfig {
  name: string
  description: string
  inputs?: Record<string, ActionInput>
  outputs?: Record<string, ActionOutput>
}
```

#### 예제

```ts twoslash
// @noErrors
// @filename: workflows/example.ts
// ---cut---
import { Action } from "../generated/index.js";

const setupEnv = new Action({
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

생성된 `action.yml`은 다음과 같이 사용할 수 있습니다:

```typescript
// 다른 워크플로우에서
const setupEnv = getAction("./setup-env");

const job = new Job("ubuntu-latest")
  .addStep(setupEnv({
    with: {
      "node-version": "20",
    },
  }));
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
// @noErrors
// @filename: workflows/example.ts
// ---cut---
import { Job } from "../generated/index.js";

// 재사용 가능한 작업 템플릿 정의
class NodeTestJob extends Job {
  constructor(nodeVersion: string) {
    super("ubuntu-latest");

    this
      .addStep(checkout({}))
      .addStep(setupNode({
        with: { "node-version": nodeVersion },
      }))
      .addStep({ run: "npm ci" })
      .addStep({ run: "npm test" });
  }
}

// 워크플로우에서 사용
const workflow = new Workflow({
  name: "Test Matrix",
  on: { push: { branches: ["main"] } },
})
  .addJob("test-node-18", new NodeTestJob("18"))
  .addJob("test-node-20", new NodeTestJob("20"))
  .addJob("test-node-22", new NodeTestJob("22"));
```

더 복잡한 재사용 가능한 작업도 만들 수 있습니다:

```typescript
class DeployJob extends Job {
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

// 워크플로우에서 사용
const workflow = new Workflow({
  name: "Deploy",
  on: { push: { tags: ["v*"] } },
})
  .addJob("deploy-staging", new DeployJob("staging"))
  .addJob("deploy-production", new DeployJob("production").needs(["deploy-staging"]));
```

---

### `WorkflowCall`

다른 리포지토리나 파일에 정의된 [재사용 가능한 워크플로우](https://docs.github.com/en/actions/using-workflows/reusing-workflows)를 호출합니다. `Job`과 달리 `WorkflowCall`은 `steps`가 없으며, `uses`를 통해 참조된 워크플로우에 위임합니다.

```typescript
class WorkflowCall {
  constructor(uses: string)
  with(inputs: Record<string, unknown>): this
  secrets(s: Record<string, unknown> | 'inherit'): this
  needs(deps: string | string[]): this
  when(condition: string): this
  permissions(perms: Permissions): this
  toJSON(): object
}
```

| 메서드 | 설명 |
|--------|------|
| `with(inputs)` | 재사용 워크플로우에 입력값을 전달합니다. |
| `secrets(s)` | 시크릿을 명시적으로 전달하거나, `'inherit'`로 모든 시크릿을 전달합니다. |
| `needs(deps)` | job 의존성을 설정합니다. |
| `when(condition)` | job의 `if` 조건을 설정합니다. |
| `permissions(perms)` | job 수준의 권한을 설정합니다. |

#### 예제

```ts twoslash
// @filename: workflows/example.ts
// ---cut---
import { WorkflowCall, Workflow } from "../generated/index.js";

const deploy = new WorkflowCall("octo-org/deploy/.github/workflows/deploy.yml@main")
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

const job = new Job("ubuntu-latest")
  .addStep({
    ...ActionRef.from(setupEnv).toJSON(),
    with: { "node-version": "20" },
  });
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
const setupNode = getAction("actions/setup-node@v4");

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

`jobOutputs()`를 활용한 전체 typed outputs 예제는 [워크플로우 작성의 출력 섹션](../guide/writing-workflows.md#출력)을 참조하세요.

---

### `jobOutputs()`

다운스트림 job에서 사용할 타입이 지정된 job 출력 참조를 생성합니다. `Job` 객체의 `.outputs()` 호출에서 출력 키를 읽어 <code v-pre>${{ needs.&lt;jobId&gt;.outputs.&lt;key&gt; }}</code> 표현식을 생성합니다.

```typescript
function jobOutputs<O extends Record<string, string>>(
  jobId: string,
  job: Job<O>,
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
interface ActionStep<O = {}> extends JobStep {
  readonly outputs: O
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

```typescript
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

## 다음 단계

- [예제](/ko/examples/simple-ci) 보기
- [액션](./actions.md)에 대해 알아보기
