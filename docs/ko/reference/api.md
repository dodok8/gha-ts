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

#### 예제

```typescript
const workflow = new Workflow({
  name: "CI",
  on: {
    push: { branches: ["main"] },
    pull_request: { branches: ["main"] },
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
}
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
  .addStep({ run: "npm ci" });

setupEnv.build("setup-env");
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
  .addJob("test-node-20", new NodeTestJob("20"));
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
  },
});
```

## 다음 단계

- [예제](/ko/examples/simple-ci) 보기
- [액션](./actions.md)에 대해 알아보기
