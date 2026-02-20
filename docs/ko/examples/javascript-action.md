# 예제: JavaScript 액션

Node.js 기반 GitHub Actions를 만들고 워크플로우에서 사용합니다.

## JavaScript 액션 정의

`workflows/hello.ts` 생성:

```typescript
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

빌드:

```bash
gaji build
```

`.github/actions/hello-world/action.yml`이 생성됩니다.

## 워크플로우에서 액션 사용

같은 파일에서 액션 정의와 이를 사용하는 워크플로우를 함께 작성할 수 있습니다:

```typescript
import { ActionRef, NodeAction, Job, Workflow } from "../generated/index.js";

// 액션 정의
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

// 워크플로우에서 액션 사용
const helloWorldJob = new Job("ubuntu-latest")
  .addStep({
    name: "Hello world action step",
    id: "hello",
    ...ActionRef.from(action).toJSON(),
    with: {
      "who-to-greet": "Mona the Octocat",
    },
  })
  .addStep({
    name: "Get the output time",
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
const action = new NodeAction(
  {
    name: "Setup and Cleanup",
    description: "Action with pre and post scripts",
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

## `ActionRef`으로 참조

`ActionRef.from()`을 사용하여 로컬에서 정의한 액션을 워크플로우 스텝에서 참조할 수 있습니다:

```typescript
const step = {
  name: "Run my action",
  id: "my-step",
  ...ActionRef.from(action).toJSON(),
  with: { input1: "value1" },
};
```

이는 `uses: ./.github/actions/<action-id>`로 변환됩니다.

## 다음 단계

- [API 레퍼런스](/ko/reference/api) 보기
- [컴포지트 액션](./composite-action.md)에 대해 알아보기
