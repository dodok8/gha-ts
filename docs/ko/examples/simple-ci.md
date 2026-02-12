# 예제: 간단한 CI

push 및 pull request에서 테스트를 실행하는 기본 CI 워크플로우입니다.

## 워크플로우

```typescript
import { getAction, Job, Workflow } from "../../generated/index.js";

const checkout = getAction("actions/checkout@v4");
const setupNode = getAction("actions/setup-node@v4");

// 테스트 작업 정의
const test = new Job("ubuntu-latest")
  .addStep(checkout({
    name: "코드 체크아웃",
  }))
  .addStep(setupNode({
    name: "Node.js 설정",
    with: {
      "node-version": "20",
      cache: "npm",
    },
  }))
  .addStep({
    name: "의존성 설치",
    run: "npm ci",
  })
  .addStep({
    name: "린터 실행",
    run: "npm run lint",
  })
  .addStep({
    name: "테스트 실행",
    run: "npm test",
  });

// 워크플로우 생성
const workflow = new Workflow({
  name: "CI",
  on: {
    push: {
      branches: ["main"],
    },
    pull_request: {
      branches: ["main"],
    },
  },
}).addJob("test", test);

// YAML로 빌드
workflow.build("ci");
```

## 설정

1. **필요한 액션 추가**:
   ```bash
   gaji add actions/checkout@v4
   gaji add actions/setup-node@v4
   ```

2. **타입 생성**:
   ```bash
   gaji dev
   ```

3. **워크플로우 생성**:
   위 코드로 `workflows/ci.ts` 생성.

4. **빌드**:
   ```bash
   gaji build
   ```

## 다음 단계

- 여러 버전 테스트를 위한 [매트릭스 빌드](./matrix-build.md) 보기
- [컴포지트 액션](./composite-action.md)에 대해 알아보기
