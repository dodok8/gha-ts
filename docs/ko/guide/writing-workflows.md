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

// 2. 워크플로우와 job, 스텝 생성
new Workflow({
  name: "CI",
  on: { push: { branches: ["main"] } },
})
  .jobs(j => j
    .add("build",
      new Job("ubuntu-latest")
        .steps(s => s
          .add(checkout({}))
        )
    )
  )
  .build("ci");
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

## Job 생성

Job은 `Job` 클래스로 생성합니다:

```typescript
const job = new Job("ubuntu-latest");
```

### 지원되는 러너

```typescript
// Ubuntu
new Job("ubuntu-latest")
new Job("ubuntu-22.04")
new Job("ubuntu-20.04")

// macOS
new Job("macos-latest")
new Job("macos-13")
new Job("macos-12")

// Windows
new Job("windows-latest")
new Job("windows-2022")
new Job("windows-2019")

// Self-hosted
new Job("self-hosted")
new Job(["self-hosted", "linux", "x64"])
```

### 스텝 추가

`.steps()` 콜백과 `.add()`로 스텝을 추가합니다:

```typescript
const job = new Job("ubuntu-latest")
  .steps(s => s
    // 액션 스텝
    .add(checkout({
      name: "Checkout",
    }))

    // run 명령
    .add({
      name: "Build",
      run: "npm run build",
    })

    // 여러 줄 명령
    .add({
      name: "Install dependencies",
      run: `
        npm ci
        npm run build
        npm test
      `.trim(),
    })

    // 환경 변수 포함
    .add({
      name: "Deploy",
      run: "npm run deploy",
      env: {
        NODE_ENV: "production",
        API_KEY: "${{ secrets.API_KEY }}",
      },
    })

    // 조건부 스텝
    .add({
      name: "Upload artifacts",
      if: "success()",
      run: "npm run upload",
    })
  );
```

## 워크플로우 생성

### 기본 워크플로우

```typescript
new Workflow({
  name: "CI",
  on: {
    push: {
      branches: ["main"],
    },
  },
})
  .jobs(j => j
    .add("build", buildJob)
  )
  .build("ci");
```

### 트리거 이벤트

#### Push

```typescript
on: {
  push: {
    branches: ["main", "develop"],
    tags: ["v*"],
    paths: ["src/**", "tests/**"],
  },
}
```

#### Pull Request

```typescript
on: {
  pull_request: {
    branches: ["main"],
    types: ["opened", "synchronize", "reopened"],
  },
}
```

#### Schedule (Cron)

```typescript
on: {
  schedule: [
    { cron: "0 0 * * *" },  // 매일 자정
  ],
}
```

#### 복수 트리거

```typescript
on: {
  push: { branches: ["main"] },
  pull_request: { branches: ["main"] },
  workflow_dispatch: {},  // 수동 트리거
}
```

### 복수 Job

```typescript
const test = new Job("ubuntu-latest")
  .steps(s => s
    .add(checkout({}))
    .add({ run: "npm test" })
  );

const build = new Job("ubuntu-latest")
  .steps(s => s
    .add(checkout({}))
    .add({ run: "npm run build" })
  );

new Workflow({
  name: "CI",
  on: { push: { branches: ["main"] } },
})
  .jobs(j => j
    .add("test", test)
    .add("build", build)
  )
  .build("ci");
```

### Job 의존성

`JobConfig` 생성자 파라미터에서 `needs`를 사용합니다:

```typescript
const test = new Job("ubuntu-latest")
  .steps(s => s
    .add({ run: "npm test" })
  );

const deploy = new Job("ubuntu-latest", {
  needs: ["test"],  // test job 완료 대기
})
  .steps(s => s
    .add({ run: "npm run deploy" })
  );

new Workflow({
  name: "Deploy",
  on: { push: { branches: ["main"] } },
})
  .jobs(j => j
    .add("test", test)
    .add("deploy", deploy)
  )
  .build("deploy");
```

## 매트릭스 빌드

`JobConfig` 생성자에서 `strategy`를 사용합니다:

```typescript
const test = new Job("${{ matrix.os }}", {
  strategy: {
    matrix: {
      os: ["ubuntu-latest", "macos-latest", "windows-latest"],
      node: ["18", "20", "22"],
    },
  },
})
```

생성된 YAML이 포함된 전체 매트릭스 빌드 예제는 [매트릭스 빌드 예제](/ko/examples/matrix-build)를 참조하세요.

## 컴포지트 액션

`Action`을 사용하여 재사용 가능한 [컴포지트 액션](https://docs.github.com/en/actions/sharing-automations/creating-actions/creating-a-composite-action)을 만듭니다. 입력을 정의하고, `.steps()`로 스텝을 추가한 뒤, `.build()`로 리포지토리에 `action.yml`을 생성합니다.

전체 예제는 [컴포지트 액션 예제](/ko/examples/composite-action)를 참조하세요. 전체 API는 [Action](/ko/reference/api#action)을 참조하세요.

## Job 상속

`Job`을 상속하여 재사용 가능한 파라미터화된 Job 템플릿을 만들 수 있습니다:

```ts twoslash
// @filename: workflows/example.ts
// ---cut---
import { Job, getAction, Workflow } from "../generated/index.js";

const checkout = getAction("actions/checkout@v5");
const setupNode = getAction("actions/setup-node@v4");

class NodeTestJob extends Job {
  constructor(nodeVersion: string) {
    super("ubuntu-latest");
    this.steps(s => s
      .add(checkout({}))
      .add(setupNode({ with: { "node-version": nodeVersion } }))
      .add({ run: "npm ci" })
      .add({ run: "npm test" })
    );
  }
}
```

전체 API 레퍼런스와 고급 패턴(예: `DeployJob`)은 [Job 상속](/ko/reference/api#job-상속)을 참조하세요.

## 전체 예제: WorkflowCall과 조합하여 환경별 배포 워크플로우 구성

재사용 가능한 워크플로우(`workflow_call`)를 만든 뒤, `WorkflowCall`로 환경마다 호출하는 패턴입니다.

먼저, 배포 작업을 담은 재사용 가능한 워크플로우를 작성합니다. `workflow_call`의 `inputs`로 환경 이름을 받습니다.

```ts twoslash
// @noErrors
// @filename: workflows/publish.ts
// ---cut---
import { getAction, Job, Workflow } from "../generated/index.js";

const checkout = getAction("actions/checkout@v5");
const setupNode = getAction("actions/setup-node@v4");

new Workflow({
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
})
  .jobs(j => j
    .add("publish",
      new Job("ubuntu-latest")
        .steps(s => s
          .add(checkout({ name: "Checkout" }))
          .add(setupNode({
            name: "Setup Node.js",
            with: { "node-version": "20", cache: "npm" },
          }))
          .add({ name: "Install dependencies", run: "npm ci" })
          .add({ name: "Build", run: "npm run build" })
          .add({
            name: "Publish",
            run: "npm run publish:${{ inputs.environment }}",
            env: {
              DEPLOY_TOKEN: "${{ secrets.DEPLOY_TOKEN }}",
            },
          })
        )
    )
  )
  .build("publish");
```

다음으로, `WorkflowCall`을 사용하여 이 워크플로우를 환경별로 호출합니다. `needs`로 alpha → staging → live 순서를 지정합니다:

```ts twoslash
// @noErrors
// @filename: workflows/release.ts
// ---cut---
import { WorkflowCall, Workflow } from "../generated/index.js";

const alpha = new WorkflowCall("./.github/workflows/publish.yml", {
  with: { environment: "alpha" },
  secrets: "inherit",
});

const staging = new WorkflowCall("./.github/workflows/publish.yml", {
  with: { environment: "staging" },
  secrets: "inherit",
  needs: ["publish-alpha"],
});

const live = new WorkflowCall("./.github/workflows/publish.yml", {
  with: { environment: "live" },
  secrets: "inherit",
  needs: ["publish-staging"],
});

new Workflow({
  name: "Release",
  on: { push: { tags: ["v*"] } },
})
  .jobs(j => j
    .add("publish-alpha", alpha)
    .add("publish-staging", staging)
    .add("publish-live", live)
  )
  .build("release");
```

이 구조의 장점은 배포 로직이 `publish.yml` 한 곳에만 존재한다는 점입니다. 배포 스텝을 수정해야 할 때 `publish.ts`만 고치면 세 환경 모두에 반영됩니다.

## DockerAction

`DockerAction`으로 [Docker 컨테이너 액션](https://docs.github.com/en/actions/sharing-automations/creating-actions/creating-a-docker-container-action)을 정의합니다. `Dockerfile` 또는 `docker://` 접두사를 붙인 Docker Hub 이미지를 지정할 수 있습니다.

전체 API와 예제는 [DockerAction API](/ko/reference/api#dockeraction)를 참조하세요.

## 환경 변수

### 워크플로우 수준

```typescript
new Workflow({
  name: "CI",
  on: { push: { branches: ["main"] } },
  env: {
    NODE_ENV: "production",
  },
});
```

### Job 수준

```typescript
new Job("ubuntu-latest", {
  env: {
    DATABASE_URL: "${{ secrets.DATABASE_URL }}",
  },
});
```

### 스텝 수준

```typescript
.add({
  run: "npm run deploy",
  env: {
    API_KEY: "${{ secrets.API_KEY }}",
  },
})
```

## 출력

### 타입이 지정된 스텝 출력

액션 스텝에 `id`를 제공하면, gaji는 타입이 지정된 출력 속성이 있는 `ActionStep`을 반환합니다:

```typescript
const checkout = getAction("actions/checkout@v5");

// id를 제공하면 타입이 지정된 출력을 사용 가능
const step = checkout({ id: "my-checkout" });
step.outputs.ref     // "${{ steps.my-checkout.outputs.ref }}"
step.outputs.commit  // "${{ steps.my-checkout.outputs.commit }}"
```

### Job 간 출력 전달

기본 패턴은 `.jobs()` 콜백을 사용하는 것으로, job 출력 컨텍스트가 자동으로 전달됩니다:

```typescript
const checkout = getAction("actions/checkout@v5");

new Workflow({ name: "CI", on: { push: {} } })
  .jobs(j => j
    .add("build",
      new Job("ubuntu-latest")
        .steps(s => s
          .add(checkout({ id: "co" }))
        )
        .outputs(output => ({ ref: output.co.ref }))
    )
    .add("deploy", output =>
      new Job("ubuntu-latest", { needs: ["build"] })
        .steps(s => s
          .add({ run: "echo " + output.build.ref })
        )
    )
  )
  .build("ci");
```

`deploy` 콜백의 `output` 파라미터는 `build` job이 선언한 출력에 대한 타입이 지정된 접근을 제공하며, <code v-pre>${{ needs.build.outputs.ref }}</code> 표현식을 생성합니다.

Job을 별도의 변수로 정의할 때는 `jobOutputs()`를 호환성 헬퍼로 사용할 수 있습니다:

```typescript
const build = new Job("ubuntu-latest")
  .steps(s => s.add(checkout({ id: "co" })))
  .outputs(output => ({ ref: output.co.ref }));

const buildOutputs = jobOutputs("build", build);
// buildOutputs.ref → "${{ needs.build.outputs.ref }}"
```

수동으로 정의된 출력(`$GITHUB_OUTPUT`에 쓰는 `run` 스텝 등)도 사용할 수 있습니다:

```typescript
new Workflow({ name: "CI", on: { push: { tags: ["v*"] } } })
  .jobs(j => j
    .add("setup",
      new Job("ubuntu-latest")
        .steps(s => s
          .add({ id: "version", run: 'echo "value=1.0.0" >> $GITHUB_OUTPUT' })
        )
        .outputs({
          version: "${{ steps.version.outputs.value }}",
        })
    )
    .add("deploy", output =>
      new Job("ubuntu-latest", { needs: ["setup"] })
        .steps(s => s
          .add({ run: "deploy --version " + output.setup.version })
        )
    )
  )
  .build("ci");
```

## 팁

### 1. 감시 모드 사용

개발 중에는 항상 `gaji dev --watch`를 사용하여 새 액션에 대한 타입을 자동으로 생성하세요.

### 2. 생성된 YAML 검토

커밋하기 전에 항상 생성된 YAML을 검토하여 정확성을 확인하세요.

### 3. 타입 안전성

gaji는 액션 입력 키의 오타와 잘못된 값 타입을 컴파일 시점에 잡아냅니다. 예제는 [타입 안전성](/ko/reference/actions#타입-안전성)을 참조하세요.

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
.add({ run: "echo \"hello\nworld\"" })
```

JS 문자열에는 실제 줄바꿈이 포함되어 있어 YAML에서 올바르게 처리됩니다. 하지만 YAML 출력에 리터럴 `\n` 문자를 그대로 유지하려면 이중 이스케이프가 필요합니다:

```typescript
// YAML에서 리터럴 \n을 유지하려면 이중 이스케이프
.add({ run: "echo hello\\nworld" })
```

**팁**: 여러 줄 명령어에는 이스케이프 시퀀스 대신 템플릿 리터럴을 사용하세요:

```typescript
.add({
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
