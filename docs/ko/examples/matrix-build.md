# 예제: 매트릭스 빌드

여러 운영 체제와 Node.js 버전에서 테스트합니다.

## 워크플로우

```typescript
import { getAction, Job, Workflow } from "../../generated/index.js";

const checkout = getAction("actions/checkout@v4");
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
    name: "코드 체크아웃",
  }))
  .addStep(setupNode({
    name: "Node.js ${{ matrix.node }} 설정",
    with: {
      "node-version": "${{ matrix.node }}",
      cache: "npm",
    },
  }))
  .addStep({
    name: "의존성 설치",
    run: "npm ci",
  })
  .addStep({
    name: "테스트 실행",
    run: "npm test",
  });

// 워크플로우 생성
const workflow = new Workflow({
  name: "매트릭스 테스트",
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
