# 워크플로우 작성

이 가이드는 gaji를 사용하여 타입 안전한 GitHub Actions 워크플로우를 작성하는 방법을 설명합니다.

::: tip 독립 실행형 TypeScript 파일
gaji가 생성한 워크플로우 파일은 독립적으로 실행 가능합니다. TypeScript 런타임(tsx, ts-node, Deno)으로 직접 실행하여 워크플로우 JSON을 출력할 수 있습니다. 디버깅과 검사에 편리합니다.
:::

## 기본 구조

gaji 워크플로우는 세 가지 주요 구성 요소로 이루어집니다:

1. **액션**: `getAction()`을 사용하여 가져오기
2. **작업**: `Job` 클래스를 사용하여 생성
3. **워크플로우**: `Workflow` 클래스를 사용하여 생성

```ts twoslash
// @filename: workflows/example.ts
// ---cut---
import { getAction, Job, Workflow } from "../generated/index.js";

// 1. 액션 가져오기
const checkout = getAction("actions/checkout@v5");

// 2. 작업 생성
const build = new Job("ubuntu-latest")
  .addStep(checkout({}));

// 3. 워크플로우 생성
const workflow = new Workflow({
  name: "CI",
  on: { push: { branches: ["main"] } },
}).addJob("build", build);

// 4. YAML로 빌드
workflow.build("ci");
```

## 액션 사용하기

### gaji dev 실행

```bash
gaji dev --watch
```

### 액션 가져오기

`getAction()`을 사용하여 액션 가져오기:

```typescript
const checkout = getAction("actions/checkout@v5");
const setupNode = getAction("actions/setup-node@v4");
const cache = getAction("actions/cache@v4");
```

### 타입 안전성으로 액션 사용

액션은 설정을 받는 함수를 반환합니다:

```typescript
const step = checkout({
  name: "Checkout code",
  with: {
    // ✅ 모든 입력에 대한 완전한 자동완성!
    repository: "owner/repo",
    ref: "main",
    token: "${{ secrets.GITHUB_TOKEN }}",
    "fetch-depth": 0,
  },
});
```

에디터가 제공하는 것:
- ✅ 모든 액션 입력에 대한 자동완성
- ✅ 타입 체크
- ✅ action.yml의 문서
- ✅ 기본값 표시

## CompositeJob

`CompositeJob`을 사용하여 재사용 가능한 Job을 정의할 수 있습니다.

```ts twoslash
// @noErrors
// @filename: workflows/example.ts
// ---cut---
import { CompositeJob, getAction } from "../generated/index.js";

const checkout = getAction("actions/checkout@v5");
const setupNode = getAction("actions/setup-node@v4");

// 재사용 가능한 작업 클래스 정의
class NodeTestJob extends CompositeJob {
  constructor(nodeVersion: string) {
    super("ubuntu-latest");

    this
      .addStep(checkout({ name: "Checkout code" }))
      .addStep(setupNode({
        name: `Setup Node.js ${nodeVersion}`,
        with: {
          "node-version": nodeVersion,
          cache: "npm",
        },
      }))
      .addStep({ name: "Install dependencies", run: "npm ci" })
      .addStep({ name: "Run tests", run: "npm test" });
  }
}

// 워크플로우에서 사용
const workflow = new Workflow({
  name: "Test Matrix",
  on: { push: { branches: ["main"] } },
})
  .addJob("test-node-18", new NodeTestJob("18"))
  .addJob("test-node-20", new NodeTestJob("20"))
  .addJob("test-node-22", new NodeTestJob("22"));
```

## 전체 예제: CallJob과 조합하여 환경별 배포 워크플로우 구성

재사용 가능한 워크플로우(`workflow_call`)를 만든 뒤, `CallJob`으로 환경마다 호출하는 패턴입니다.

먼저, 배포 작업을 담은 재사용 가능한 워크플로우를 작성합니다. `workflow_call`의 `inputs`로 환경 이름을 받습니다.

```ts twoslash
// @noErrors
// @filename: workflows/publish.ts
// ---cut---
import { getAction, Job, Workflow } from "../generated/index.js";

const checkout = getAction("actions/checkout@v5");
const setupNode = getAction("actions/setup-node@v4");

const publish = new Job("ubuntu-latest")
  .addStep(checkout({ name: "Checkout" }))
  .addStep(setupNode({
    name: "Setup Node.js",
    with: { "node-version": "20", cache: "npm" },
  }))
  .addStep({ name: "Install dependencies", run: "npm ci" })
  .addStep({ name: "Build", run: "npm run build" })
  .addStep({
    name: "Publish",
    run: "npm run publish:${{ inputs.environment }}",
    env: {
      DEPLOY_TOKEN: "${{ secrets.DEPLOY_TOKEN }}",
    },
  });

const workflow = new Workflow({
  name: "Publish",
  on: {
    workflow_call: {
      inputs: {
        environment: {
          description: "배포 대상 환경 (alpha, staging, live)",
          required: true,
          type: "choice",
          options: ["alpha", "staging", "live"],
        },
      },
      secrets: {
        DEPLOY_TOKEN: { required: true },
      },
    },
  },
}).addJob("publish", publish);

workflow.build("publish");
```

다음으로, `CallJob`을 사용하여 이 워크플로우를 환경별로 호출합니다. `needs`로 alpha → staging → live 순서를 지정합니다:

```ts twoslash
// @noErrors
// @filename: workflows/release.ts
// ---cut---
import { CallJob, Workflow } from "../generated/index.js";

const alpha = new CallJob("./.github/workflows/publish.yml")
  .with({ environment: "alpha" })
  .secrets("inherit");

const staging = new CallJob("./.github/workflows/publish.yml")
  .with({ environment: "staging" })
  .secrets("inherit")
  .needs(["publish-alpha"]);

const live = new CallJob("./.github/workflows/publish.yml")
  .with({ environment: "live" })
  .secrets("inherit")
  .needs(["publish-staging"]);

const workflow = new Workflow({
  name: "Release",
  on: { push: { tags: ["v*"] } },
})
  .addJob("publish-alpha", alpha)
  .addJob("publish-staging", staging)
  .addJob("publish-live", live);

workflow.build("release");
```

이 구조의 장점은 배포 로직이 `publish.yml` 한 곳에만 존재한다는 점입니다. 배포 스텝을 수정해야 할 때 `publish.ts`만 고치면 세 환경 모두에 반영됩니다.

## DockerAction

[Docker 컨테이너 액션](https://docs.github.com/en/actions/sharing-automations/creating-actions/creating-a-docker-container-action)을 정의합니다. `runs.using`이 `"docker"`이며, 이미지와 entrypoint를 지정합니다.

```ts twoslash
// @noErrors
// @filename: workflows/example.ts
// ---cut---
import { DockerAction } from "../generated/index.js";

const action = new DockerAction(
  {
    name: "Lint with Super-Linter",
    description: "Run Super-Linter in a Docker container",
    inputs: {
      args: {
        description: "Linter arguments",
        required: false,
      },
    },
  },
  {
    using: "docker",
    image: "Dockerfile",
    entrypoint: "entrypoint.sh",
    args: ["--config", ".lintrc"],
    env: {
      DEFAULT_BRANCH: "main",
    },
  },
);

action.build("super-linter");
```

`DockerAction`은 `.github/actions/<id>/action.yml`을 생성합니다. `CallAction.from()`으로 워크플로우에서 참조할 수 있습니다.

Docker Hub 이미지를 직접 사용하려면 `image`에 `docker://` 접두사를 붙입니다:

```typescript
{
  using: "docker",
  image: "docker://alpine:3.19",
}
```

## 팁

### 1. 감시 모드 사용

개발 중에는 항상 `gaji dev --watch`를 사용하여 새 액션에 대한 타입을 자동으로 생성하세요.

### 2. 생성된 YAML 검토

커밋하기 전에 항상 생성된 YAML을 검토하여 정확성을 확인하세요.

### 3. 타입 안전성

TypeScript의 타입 체크를 활용하세요:

```typescript
// ❌ 타입 오류 - 알 수 없는 속성 키
setupNode({
  with: {
    "node-versoin": "20",  // 키 이름 오타! ❌
  },
});

// ❌ 타입 오류 - 잘못된 타입
setupNode({
  with: {
    "node-version": 20,  // 문자열이어야 함! ❌
  },
});

// ✅ 올바름
setupNode({
  with: {
    "node-version": "20",  // ✅ 올바른 키와 타입
    cache: "npm",
  },
});
```

**참고**: gaji는 속성 키와 타입에 대한 타입 안전성을 제공하지만, 문자열 값(예: `cache: "npn"` vs `cache: "npm"`)은 컴파일 시점에 검증할 수 없습니다. 이러한 오타를 잡으려면 항상 생성된 YAML을 검토하세요.

## 알려진 제한사항

### `getAction()`은 문자열 리터럴만 지원

gaji는 TypeScript 파일을 **실행하지 않고** 정적으로 분석하여 액션 참조를 추출합니다. 따라서 `getAction()`은 문자열 리터럴만 지원합니다:

```typescript
// ✅ 동작 - 문자열 리터럴
const checkout = getAction("actions/checkout@v5");

// ❌ 동작하지 않음 - 변수 참조
const ref = "actions/checkout@v5";
const checkout = getAction(ref);

// ❌ 동작하지 않음 - 템플릿 리터럴
const checkout = getAction(`actions/checkout@v${version}`);

// ❌ 동작하지 않음 - 객체 속성
const checkout = getAction(config.checkoutRef);
```

gaji가 액션 참조를 감지하지 못하면 `action.yml`을 가져오거나 해당 액션의 타입을 생성하지 않습니다. 항상 `owner/repo@version` 문자열을 직접 전달하세요.

### YAML 출력에서의 문자열 이스케이프

gaji는 JavaScript 문자열을 YAML로 변환하므로, JavaScript에서 이미 이스케이프된 문자가 출력에서 이중 이스케이프될 수 있습니다:

```typescript
// TypeScript에서 \n은 줄바꿈 문자
.addStep({ run: "echo \"hello\nworld\"" })
```

JS 문자열에는 실제 줄바꿈이 포함되어 있어 YAML에서 올바르게 처리됩니다. 하지만 YAML 출력에 리터럴 `\n` 문자를 그대로 유지하려면 이중 이스케이프가 필요합니다:

```typescript
// YAML에서 리터럴 \n을 유지하려면 이중 이스케이프
.addStep({ run: "echo hello\\nworld" })
```

**팁**: 여러 줄 명령어에는 이스케이프 시퀀스 대신 템플릿 리터럴을 사용하세요:

```typescript
.addStep({
  run: `
    echo hello
    echo world
  `.trim(),
})
```

## 다음 단계

- [설정](./configuration.md)에 대해 알아보기
- [예제](/ko/examples/simple-ci) 보기
- [API 레퍼런스](/ko/reference/api) 확인
