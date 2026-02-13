# TypeScript API 레퍼런스

gaji의 TypeScript API에 대한 완전한 레퍼런스입니다.

## 핵심 클래스

### `Workflow`

GitHub Actions 워크플로우를 나타냅니다.

```typescript
class Workflow {
  constructor(config: WorkflowConfig)
  addJob(id: string, job: Job | CompositeJob | CallJob): this
  static fromObject(def: WorkflowDefinition, id?: string): Workflow
  build(filename?: string): void
  toJSON(): WorkflowDefinition
}
```

| 메서드 | 설명 |
|--------|------|
| `addJob(id, job)` | 워크플로우에 job을 추가합니다. `Job`, `CompositeJob`, `CallJob`을 받습니다. |
| `fromObject(def, id?)` | `WorkflowDefinition` 객체로부터 Workflow를 생성합니다. 기존 YAML 형태의 정의를 래핑할 때 유용합니다. |
| `build(filename?)` | 워크플로우를 YAML로 컴파일하여 출력합니다. |
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

워크플로우의 작업을 나타냅니다.

```typescript
class Job {
  constructor(runsOn: string | string[], options?: Partial<JobDefinition>)
  addStep(step: Step): this
  needs(jobs: string | string[]): this
  env(variables: Record<string, string>): this
  when(condition: string): this
  permissions(perms: Permissions): this
  outputs(outputs: Record<string, string>): this
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
| `outputs(outputs)` | job 출력을 정의합니다. |
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

### `CompositeAction`

재사용 가능한 컴포지트 액션을 만듭니다.

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

#### 예제

```typescript
import { CompositeAction } from "../generated/index.js";

const setupEnv = new CompositeAction({
  name: "환경 설정",
  description: "Node.js 설정 및 의존성 설치",
  inputs: {
    "node-version": {
      description: "Node.js 버전",
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

### `JavaScriptAction`

Node.js 기반 GitHub Actions를 만듭니다.

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

#### 예제

```typescript
import { JavaScriptAction } from "../generated/index.js";

const action = new JavaScriptAction(
  {
    name: "Hello World",
    description: "인사하고 시간을 기록합니다",
    inputs: {
      "who-to-greet": {
        description: "인사할 대상",
        required: true,
        default: "World",
      },
    },
    outputs: {
      time: {
        description: "인사한 시간",
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

`.github/actions/hello-world/action.yml`이 생성됩니다.

`CallAction.from()`으로 워크플로우에서 참조할 수 있습니다:

```typescript
const step = {
  id: "hello",
  ...CallAction.from(action).toJSON(),
  with: { "who-to-greet": "Mona the Octocat" },
};
```

---

### `CompositeJob`

TypeScript 클래스 상속을 통해 재사용 가능한 작업 템플릿을 만듭니다. `CompositeJob`은 `Job`을 상속하므로 모든 `Job` 메서드를 사용할 수 있습니다.

```typescript
class CompositeJob extends Job {
  constructor(runsOn: string | string[], options?: Partial<JobDefinition>)
}
```

`Job`과 달리 `CompositeJob`은 `extends`로 서브클래싱하여 도메인별 파라미터화된 job 템플릿을 만들 때 사용합니다. YAML 출력은 일반 `Job`과 동일합니다.

::: tip CompositeJob vs Job
일회성 job에는 `Job`을 직접 사용하세요. 공통 패턴을 파라미터와 함께 캡슐화한 **재사용 가능한 클래스**를 만들 때 `CompositeJob`을 사용하세요.
:::

#### 예제

```typescript
import { CompositeJob } from "../generated/index.js";

// 재사용 가능한 작업 템플릿 정의
class NodeTestJob extends CompositeJob {
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
  name: "테스트 매트릭스",
  on: { push: { branches: ["main"] } },
})
  .addJob("test-node-18", new NodeTestJob("18"))
  .addJob("test-node-20", new NodeTestJob("20"))
  .addJob("test-node-22", new NodeTestJob("22"));
```

더 복잡한 재사용 가능한 작업도 만들 수 있습니다:

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
        name: "배포",
        run: `npm run deploy:${environment}`,
        env: {
          DEPLOY_TOKEN: "${{ secrets.DEPLOY_TOKEN }}",
        },
      });
  }
}

// 워크플로우에서 사용
const workflow = new Workflow({
  name: "배포",
  on: { push: { tags: ["v*"] } },
})
  .addJob("deploy-staging", new DeployJob("staging"))
  .addJob("deploy-production", new DeployJob("production").needs(["deploy-staging"]));
```

---

### `CallJob`

다른 리포지토리나 파일에 정의된 [재사용 가능한 워크플로우](https://docs.github.com/en/actions/using-workflows/reusing-workflows)를 호출합니다. `Job`과 달리 `CallJob`은 `steps`가 없으며, `uses`를 통해 참조된 워크플로우에 위임합니다.

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

| 메서드 | 설명 |
|--------|------|
| `with(inputs)` | 재사용 워크플로우에 입력값을 전달합니다. |
| `secrets(s)` | 시크릿을 명시적으로 전달하거나, `'inherit'`로 모든 시크릿을 전달합니다. |
| `needs(deps)` | job 의존성을 설정합니다. |
| `when(condition)` | job의 `if` 조건을 설정합니다. |
| `permissions(perms)` | job 수준의 권한을 설정합니다. |

#### 예제

```typescript
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

### `CallAction`

gaji로 빌드한 로컬 composite 또는 JavaScript 액션을 job의 스텝으로 참조합니다.

```typescript
class CallAction {
  constructor(uses: string)
  static from(action: CompositeAction | JavaScriptAction): CallAction
  toJSON(): Step
}
```

| 메서드 | 설명 |
|--------|------|
| `from(action)` | `CompositeAction` 또는 `JavaScriptAction` 인스턴스로부터 `CallAction`을 생성합니다. `.github/actions/<id>` 경로를 자동으로 해석합니다. |

#### 예제

```typescript
import { CompositeAction, CallAction, Job } from "../generated/index.js";

const setupEnv = new CompositeAction({
  name: "Setup",
  description: "환경 설정",
});
setupEnv.build("setup-env");

const job = new Job("ubuntu-latest")
  .addStep({
    ...CallAction.from(setupEnv).toJSON(),
    with: { "node-version": "20" },
  });
```

---

## 함수

### `getAction()`

타입이 지정된 액션 함수를 가져옵니다.

```typescript
function getAction<T extends string>(
  ref: T
): (config?: ActionConfig) => Step
```

#### 예제

```typescript
const checkout = getAction("actions/checkout@v4");
const setupNode = getAction("actions/setup-node@v4");

// 완전한 타입 안전성으로 사용
const step = checkout({
  name: "코드 체크아웃",
  with: {
    // ✅ 자동완성 사용 가능!
    repository: "owner/repo",
    ref: "main",
    "fetch-depth": 0,
  },
});
```

---

## 타입 정의

### `Step`

워크플로우 스텝입니다.

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

액션 입력 정의 (CompositeAction용)입니다.

```typescript
interface ActionInput {
  description: string
  required?: boolean
  default?: string
}
```

### `ActionOutput`

액션 출력 정의 (CompositeAction용)입니다.

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

## 다음 단계

- [예제](/ko/examples/simple-ci) 보기
- [액션](./actions.md)에 대해 알아보기
