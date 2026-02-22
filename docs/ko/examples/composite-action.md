# 예제: 컴포지트 액션

재사용 가능한 컴포지트 액션과 작업을 만듭니다.

## Job 상속

재사용 가능한 작업 템플릿을 만듭니다.

### 기본 예제

```typescript
import { Job, getAction } from "../../generated/index.js";

const checkout = getAction("actions/checkout@v5");
const setupNode = getAction("actions/setup-node@v4");

// 재사용 가능한 작업 클래스 정의
class NodeTestJob extends Job {
  constructor(nodeVersion: string) {
    super("ubuntu-latest");

    this.steps(s => s
      .add(checkout({ name: "Checkout code" }))
      .add(setupNode({
        name: `Setup Node.js ${nodeVersion}`,
        with: {
          "node-version": nodeVersion,
          cache: "npm",
        },
      }))
      .add({ name: "Install dependencies", run: "npm ci" })
      .add({ name: "Run tests", run: "npm test" })
    );
  }
}

// 워크플로우에서 사용
const workflow = new Workflow({
  name: "Test Matrix",
  on: { push: { branches: ["main"] } },
}).jobs(j => j
  .add("test-node-18", new NodeTestJob("18"))
  .add("test-node-20", new NodeTestJob("20"))
  .add("test-node-22", new NodeTestJob("22"))
);

workflow.build("test-matrix");
```

### 고급 예제: 매개변수화된 배포 작업

```typescript
class DeployJob extends Job {
  constructor(
    environment: "staging" | "production",
    region: string = "us-east-1",
    config: Record<string, unknown> = {}
  ) {
    super("ubuntu-latest", {
      env: {
        ENVIRONMENT: environment,
        REGION: region,
        API_URL: environment === "production"
          ? "https://api.example.com"
          : "https://staging.api.example.com",
      },
      ...config,
    });

    this.steps(s => s
      .add(checkout({ name: "Checkout code" }))
      .add(setupNode({
        name: "Setup Node.js",
        with: {
          "node-version": "20",
          cache: "npm",
        },
      }))
      .add({ name: "Install dependencies", run: "npm ci" })
      .add({
        name: "Build",
        run: `npm run build:${environment}`,
      })
      .add({
        name: `Deploy to ${environment}`,
        run: `npm run deploy`,
        env: {
          DEPLOY_TOKEN: "${{ secrets.DEPLOY_TOKEN }}",
          AWS_REGION: region,
        },
      })
    );
  }
}

// 워크플로우에서 사용
const workflow = new Workflow({
  name: "Deploy",
  on: { push: { tags: ["v*"] } },
}).jobs(j => j
  .add("deploy-staging-us", new DeployJob("staging", "us-east-1"))
  .add("deploy-staging-eu", new DeployJob("staging", "eu-west-1"))
  .add("deploy-production",
    new DeployJob("production", "us-east-1", {
      needs: ["deploy-staging-us", "deploy-staging-eu"],
    })
  )
);

workflow.build("deploy");
```

## 장점

Composite action과 Job 상속을 사용하면 패턴을 한 번 정의하고 여러 워크플로우에서 재사용할 수 있습니다. 액션 입력과 작업 매개변수가 타입 체크되어 리팩토링이 안전합니다. 공유 정의를 수정하면 모든 호출처에 반영됩니다.

## 더 읽어보기

- [Composite Action](https://docs.github.com/en/actions/tutorials/create-actions/create-a-composite-action)

## 다음 단계

- [API 레퍼런스](/ko/reference/api) 보기
- [매트릭스 빌드](./matrix-build.md)에 대해 알아보기
