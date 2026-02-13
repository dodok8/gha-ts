# 예제: JavaScript 액션

Node.js 기반 GitHub Actions를 만들고 워크플로우에서 사용합니다.

## JavaScript 액션 정의

`workflows/hello.ts` 생성:

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

빌드:

```bash
gaji build
```

`.github/actions/hello-world/action.yml`이 생성됩니다.

## 워크플로우에서 액션 사용

같은 파일에서 액션 정의와 이를 사용하는 워크플로우를 함께 작성할 수 있습니다:

```typescript
import { CallAction, JavaScriptAction, Job, Workflow } from "../generated/index.js";

// 액션 정의
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

// 워크플로우에서 액션 사용
const helloWorldJob = new Job("ubuntu-latest")
  .addStep({
    name: "Hello world 액션 스텝",
    id: "hello",
    ...CallAction.from(action).toJSON(),
    with: {
      "who-to-greet": "Mona the Octocat",
    },
  })
  .addStep({
    name: "출력 시간 확인",
    run: 'echo "The time was ${{ steps.hello.outputs.time }}"',
  });

const workflow = new Workflow({
  name: "Use JavaScript Action",
  on: {
    push: { branches: ["main"] },
  },
}).addJob("hello_world_job", helloWorldJob);

workflow.build("use-js-action");
```

다음 두 파일이 모두 생성됩니다:
- `.github/actions/hello-world/action.yml` - 액션 정의
- `.github/workflows/use-js-action.yml` - 액션을 사용하는 워크플로우

## Pre/Post 스크립트

JavaScript 액션은 라이프사이클 훅을 지원합니다:

```typescript
const action = new JavaScriptAction(
  {
    name: "Setup and Cleanup",
    description: "pre/post 스크립트가 있는 액션",
  },
  {
    using: "node20",
    main: "dist/index.js",
    pre: "dist/setup.js",
    post: "dist/cleanup.js",
    "post-if": "always()",
  },
);

action.build("setup-cleanup");
```

## `CallAction`으로 참조

`CallAction.from()`을 사용하여 로컬에서 정의한 액션을 워크플로우 스텝에서 참조할 수 있습니다:

```typescript
const step = {
  name: "내 액션 실행",
  id: "my-step",
  ...CallAction.from(action).toJSON(),
  with: { input1: "value1" },
};
```

이는 `uses: ./.github/actions/<action-id>`로 변환됩니다.

## 다음 단계

- [API 레퍼런스](/ko/reference/api) 보기
- [컴포지트 액션](./composite-action.md)에 대해 알아보기
