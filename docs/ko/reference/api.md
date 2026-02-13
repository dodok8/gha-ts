# TypeScript API 레퍼런스

gaji의 TypeScript API에 대한 완전한 레퍼런스입니다.

## 핵심 클래스

### `Workflow`

GitHub Actions 워크플로우를 나타냅니다.

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

---

### `Job`

워크플로우의 작업을 나타냅니다.

```typescript
class Job {
  constructor(runsOn: string | string[])
  addStep(step: Step): this
  needs(jobs: string[]): this
  strategy(strategy: JobStrategy): this
  env(variables: Record<string, string>): this
  outputs(outputs: Record<string, string>): this
  when(condition: string): this
  permissions(p: Permissions): this
  continueOnError(v: boolean): this
  timeoutMinutes(m: number): this
}
```

#### 예제

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

재사용 가능한 작업 템플릿을 만듭니다.

```typescript
class CompositeJob {
  constructor(runsOn: string | string[])
  addStep(step: Step): this
  needs(jobs: string[]): this
  strategy(strategy: JobStrategy): this
  env(variables: Record<string, string>): this
}
```

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
