# 예제: 매트릭스 빌드

여러 운영 체제와 Node.js 버전에서 테스트합니다.

## 워크플로우

```typescript
import { getAction, Job, Workflow } from "../../generated/index.js";

const checkout = getAction("actions/checkout@v5");
const setupNode = getAction("actions/setup-node@v4");

// 매트릭스 테스트 작업 정의
const test = new Job("${{ matrix.os }}")
  .strategy({
    matrix: {
      os: ["ubuntu-latest", "macos-latest", "windows-latest"],
      node: ["18", "20", "22"],
    },
  })
  .addStep(checkout({
    name: "Checkout code",
  }))
  .addStep(setupNode({
    name: "Setup Node.js ${{ matrix.node }}",
    with: {
      "node-version": "${{ matrix.node }}",
      cache: "npm",
    },
  }))
  .addStep({
    name: "Install dependencies",
    run: "npm ci",
  })
  .addStep({
    name: "Run tests",
    run: "npm test",
  });

// 워크플로우 생성
const workflow = new Workflow({
  name: "Matrix Test",
  on: {
    push: {
      branches: ["main"],
    },
    pull_request: {
      branches: ["main"],
    },
  },
}).addJob("test", test);

workflow.build("matrix-test");
```

이것은 **9개의 작업** (3개 OS × 3개 Node 버전)을 생성합니다.

## 다음 단계

- [컴포지트 액션](./composite-action.md) 보기
- [작업 의존성](/ko/reference/api#job)에 대해 알아보기
