# 예제: 컴포지트 액션

재사용 가능한 컴포지트 액션과 작업을 만듭니다.

## CompositeJob

재사용 가능한 작업 템플릿을 만듭니다.

### 기본 예제

```typescript
import { CompositeJob, getAction } from "../../generated/index.js";

const checkout = getAction("actions/checkout@v4");
const setupNode = getAction("actions/setup-node@v4");

// 재사용 가능한 작업 클래스 정의
class NodeTestJob extends CompositeJob {
  constructor(nodeVersion: string) {
    super("ubuntu-latest");

    this
      .addStep(checkout({ name: "코드 체크아웃" }))
      .addStep(setupNode({
        name: `Node.js ${nodeVersion} 설정`,
        with: {
          "node-version": nodeVersion,
          cache: "npm",
        },
      }))
      .addStep({ name: "의존성 설치", run: "npm ci" })
      .addStep({ name: "테스트 실행", run: "npm test" });
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

workflow.build("test-matrix");
```

### 고급 예제: 매개변수화된 배포 작업

```typescript
class DeployJob extends CompositeJob {
  constructor(
    environment: "staging" | "production",
    region: string = "us-east-1"
  ) {
    super("ubuntu-latest");

    this
      .env({
        ENVIRONMENT: environment,
        REGION: region,
        API_URL: environment === "production"
          ? "https://api.example.com"
          : "https://staging.api.example.com",
      })
      .addStep(checkout({ name: "코드 체크아웃" }))
      .addStep(setupNode({
        name: "Node.js 설정",
        with: {
          "node-version": "20",
          cache: "npm",
        },
      }))
      .addStep({ name: "의존성 설치", run: "npm ci" })
      .addStep({
        name: "빌드",
        run: `npm run build:${environment}`,
      })
      .addStep({
        name: `${environment}에 배포`,
        run: `npm run deploy`,
        env: {
          DEPLOY_TOKEN: "${{ secrets.DEPLOY_TOKEN }}",
          AWS_REGION: region,
        },
      });
  }
}

// 워크플로우에서 사용
const workflow = new Workflow({
  name: "배포",
  on: { push: { tags: ["v*"] } },
})
  .addJob("deploy-staging-us", new DeployJob("staging", "us-east-1"))
  .addJob("deploy-staging-eu", new DeployJob("staging", "eu-west-1"))
  .addJob("deploy-production",
    new DeployJob("production", "us-east-1")
      .needs(["deploy-staging-us", "deploy-staging-eu"])
  );

workflow.build("deploy");
```

## 장점

### 코드 재사용

- 일반 패턴을 한 번 정의
- 여러 워크플로우에서 재사용
- 일관성 유지

### 타입 안전성

- 매개변수가 타입 체크됨
- 리팩토링이 더 안전
- 자동완성 작동

### 유지보수 용이

- 한 곳에서 로직 업데이트
- 변경 사항이 자동으로 전파
- 중복 감소

## 다음 단계

- [API 레퍼런스](/ko/reference/api) 보기
- [매트릭스 빌드](./matrix-build.md)에 대해 알아보기
