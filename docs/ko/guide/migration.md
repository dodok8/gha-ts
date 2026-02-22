# 마이그레이션

이 가이드는 기존 YAML 워크플로우를 gaji를 사용한 TypeScript로 마이그레이션하는 데 도움을 줍니다.

## 자동 마이그레이션

gaji는 기존 YAML 워크플로우를 자동으로 TypeScript로 변환할 수 있습니다.

```bash
gaji init --migrate
```

다음 순서로 진행됩니다.

1. `.github/workflows/`에서 기존 YAML 워크플로우 감지
2. `workflows/`에서 TypeScript로 변환
3. 원본 YAML 파일 백업 (`.yml.backup`)
4. 사용된 모든 액션에 대한 타입 생성

## 액션 마이그레이션

gaji는 로컬 액션(`.github/actions/*/action.yml`)도 TypeScript로 마이그레이션합니다.

```bash
gaji init --migrate
```

워크플로우와 액션을 자동으로 감지합니다. `runs.using` 필드에 따라 `Action`, `NodeAction`, `DockerAction` 클래스로 변환됩니다.

### 컴포지트 액션

**변환 전** (`.github/actions/setup-env/action.yml`):

```yaml
name: Setup Environment
description: Setup Node.js and install dependencies
inputs:
  node-version:
    description: Node.js version to use
    required: false
    default: "20"
outputs:
  cache-hit:
    description: Whether cache was hit
    value: ${{ steps.cache.outputs.cache-hit }}
runs:
  using: composite
  steps:
    - uses: actions/checkout@v5
    - name: Install dependencies
      run: npm ci
      shell: bash
    - name: Cache node_modules
      id: cache
      uses: actions/cache@v4
      with:
        path: node_modules
        key: ${{ runner.os }}-node-${{ hashFiles('**/package-lock.json') }}
```

**변환 후** (`workflows/action-setup-env.ts`):

```typescript
import { getAction, Action } from "../generated/index.js";

const checkout = getAction("actions/checkout@v5");
const cache = getAction("actions/cache@v4");

const action = new Action({
    name: "Setup Environment",
    description: "Setup Node.js and install dependencies",
    inputs: {
        "node-version": {
            description: "Node.js version to use",
            required: false,
            default: "20",
        },
    },
    outputs: {
        "cache-hit": {
            description: "Whether cache was hit",
            value: "${{ steps.cache.outputs.cache-hit }}",
        },
    },
});

action
    .steps(s => s
        .add(checkout({}))
        .add({
            name: "Install dependencies",
            run: "npm ci",
            shell: "bash",
        })
        .add(cache({
            id: "cache",
            name: "Cache node_modules",
            with: {
                path: "node_modules",
                key: "${{ runner.os }}-node-${{ hashFiles('**/package-lock.json') }}",
            },
        }))
    )
    .build("setup-env");
```

### JavaScript 액션

**변환 전** (`.github/actions/notify/action.yml`):

```yaml
name: Send Notification
description: Send a Slack notification
inputs:
  webhook-url:
    description: Slack webhook URL
    required: true
  message:
    description: Message to send
    required: true
runs:
  using: node20
  main: dist/index.js
  post: dist/cleanup.js
```

**변환 후** (`workflows/action-notify.ts`):

```typescript
import { NodeAction } from "../generated/index.js";

const action = new NodeAction(
    {
        name: "Send Notification",
        description: "Send a Slack notification",
        inputs: {
            "webhook-url": {
                description: "Slack webhook URL",
                required: true,
            },
            message: {
                description: "Message to send",
                required: true,
            },
        },
    },
    {
        using: "node20",
        main: "dist/index.js",
        post: "dist/cleanup.js",
    },
);

action.build("notify");
```

### Docker 액션

**변환 전** (`.github/actions/lint/action.yml`):

```yaml
name: Lint
description: Run linter in Docker
inputs:
  config:
    description: Config file path
    required: false
    default: ".lintrc"
runs:
  using: docker
  image: Dockerfile
  entrypoint: entrypoint.sh
  args:
    - --config
    - ${{ inputs.config }}
```

**변환 후** (`workflows/action-lint.ts`):

```typescript
import { DockerAction } from "../generated/index.js";

const action = new DockerAction(
    {
        name: "Lint",
        description: "Run linter in Docker",
        inputs: {
            config: {
                description: "Config file path",
                required: false,
                default: ".lintrc",
            },
        },
    },
    {
        using: "docker",
        image: "Dockerfile",
        entrypoint: "entrypoint.sh",
        args: ["--config", "${{ inputs.config }}"],
    },
);

action.build("lint");
```

### 지원되는 액션 타입

| 타입       | `runs.using`                 | 변환 대상           |
| ---------- | ---------------------------- | ------------------- |
| Composite  | `composite`                  | `Action`            |
| JavaScript | `node12`, `node16`, `node20` | `NodeAction`        |
| Docker     | `docker`                     | `DockerAction`      |

## 수동 마이그레이션

수동으로 마이그레이션하려면 다음 과정을 따르세요.

### 1단계: YAML 분석

간단한 YAML 워크플로우를 예시로 들겠습니다.

```yaml
name: CI
on:
  push:
    branches: [main]
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v5
      - uses: actions/setup-node@v4
        with:
          node-version: '20'
      - run: npm ci
      - run: npm test
```

### 2단계: 필요한 액션 추가

```bash
gaji add actions/checkout@v5
gaji add actions/setup-node@v4
```

### 3단계: TypeScript로 변환

`workflows/ci.ts` 생성:

```ts twoslash
// @filename: workflows/example.ts
// ---cut---
import { getAction, Job, Workflow } from "../generated/index.js";

// 액션 가져오기
const checkout = getAction("actions/checkout@v5");
const setupNode = getAction("actions/setup-node@v4");

// 워크플로우와 job, 스텝 생성
new Workflow({
  name: "CI",
  on: {
    push: {
      branches: ["main"],
    },
  },
})
  .jobs(j => j
    .add("build",
      new Job("ubuntu-latest")
        .steps(s => s
          .add(checkout({}))
          .add(setupNode({
            with: { "node-version": "20" },
          }))
          .add({ run: "npm ci" })
          .add({ run: "npm test" })
        )
    )
  )
  .build("ci");
```

### 4단계: 빌드 및 검증

```bash
# TypeScript를 YAML로 빌드
gaji build
```

### 5단계: 정리

검증이 끝나면 백업 파일을 삭제합니다:

```bash
rm .github/workflows/ci.yml.backup
```

## 자주 사용되는 마이그레이션 패턴

### 복수 Job

**YAML:**
```yaml
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - run: npm test

  build:
    needs: test
    runs-on: ubuntu-latest
    steps:
      - run: npm run build
```

**TypeScript:**
```typescript
const test = new Job("ubuntu-latest")
  .steps(s => s
    .add({ run: "npm test" })
  );

const build = new Job("ubuntu-latest", {
  needs: ["test"],
})
  .steps(s => s
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

### 매트릭스 전략

**YAML:**
```yaml
jobs:
  test:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest]
        node: [18, 20, 22]
```

**TypeScript:**
```typescript
const test = new Job("${{ matrix.os }}", {
  strategy: {
    matrix: {
      os: ["ubuntu-latest", "macos-latest"],
      node: ["18", "20", "22"],
    },
  },
})
  .steps(s => s
    .add(checkout({}))
  );
```

### 환경 변수

**YAML:**
```yaml
env:
  NODE_ENV: production

jobs:
  deploy:
    runs-on: ubuntu-latest
    env:
      API_KEY: ${{ secrets.API_KEY }}
```

**TypeScript:**
```typescript
new Workflow({
  name: "Deploy",
  on: { push: { branches: ["main"] } },
  env: {
    NODE_ENV: "production",
  },
})
  .jobs(j => j
    .add("deploy",
      new Job("ubuntu-latest", {
        env: {
          API_KEY: "${{ secrets.API_KEY }}",
        },
      })
    )
  )
  .build("deploy");
```

### 조건부 스텝

**YAML:**
```yaml
steps:
  - name: Deploy
    if: github.ref == 'refs/heads/main'
    run: npm run deploy
```

**TypeScript:**
```typescript
.add({
  name: "Deploy",
  if: "github.ref == 'refs/heads/main'",
  run: "npm run deploy",
})
```

### Job 출력

**YAML:**
```yaml
jobs:
  build:
    outputs:
      version: ${{ steps.version.outputs.value }}
    steps:
      - id: version
        run: echo "value=1.0.0" >> $GITHUB_OUTPUT
```

**TypeScript:**
```typescript
const build = new Job("ubuntu-latest")
  .steps(s => s
    .add({
      id: "version",
      run: 'echo "value=1.0.0" >> $GITHUB_OUTPUT',
    })
  )
  .outputs({
    version: "${{ steps.version.outputs.value }}",
  });
```

## 설정 마이그레이션

이전 `.gaji.toml` 설정 파일을 사용하는 프로젝트는 `gaji.config.ts`로 마이그레이션할 수 있습니다.

### 자동 마이그레이션

`.gaji.toml`이 있는 프로젝트에서 `gaji init`을 실행하면 gaji가 파일을 감지하고 마이그레이션을 제안합니다:

```bash
gaji init
# Detected .gaji.toml configuration file.
# Migrate to gaji.config.ts? [y/N]
```

확인하면 다음 작업이 수행됩니다:

1. `.gaji.toml`을 읽고 `defineConfig()`을 사용하는 `gaji.config.ts` 파일 생성
2. `.gaji.local.toml`이 있으면 `gaji.config.local.ts`도 생성
3. 마이그레이션 성공 후 기존 TOML 파일 삭제

### 수동 마이그레이션

수동으로 마이그레이션하려면 프로젝트 루트에 `gaji.config.ts`를 생성합니다:

**변환 전** (`.gaji.toml`):

```toml
workflows = "src/workflows"
output = ".github"

[build]
cache_ttl_days = 14

[github]
token = "ghp_xxx"
```

**변환 후** (`gaji.config.ts`):

```typescript
import { defineConfig } from "./generated/index.js";

export default defineConfig({
  workflows: "src/workflows",
  build: {
    cacheTtlDays: 14,
  },
});
```

TOML 키는 `snake_case`이고 TypeScript 설정은 `camelCase`를 사용합니다. `github.token` 같은 비밀 값은 `gaji.config.local.ts`에 넣어야 합니다 (`.gitignore`에 추가):

```typescript
// gaji.config.local.ts
import { defineConfig } from "./generated/index.js";

export default defineConfig({
  github: {
    token: "ghp_xxx",
  },
});
```

## 마이그레이션 체크리스트

1. gaji 설치
2. 프로젝트 초기화
3. 워크플로우에 사용된 모든 액션 추가
4. YAML을 TypeScript로 변환
5. 생성된 YAML 빌드 및 검증
6. 브랜치에서 워크플로우 테스트
7. 백업 파일 삭제
8. 문서 업데이트

## 팁

### 1. 작은 것부터 시작

가장 간단한 워크플로우부터 하나씩 마이그레이션하세요.

### 2. 자동 마이그레이션 활용

복잡한 워크플로우는 gaji의 자동 변환을 먼저 사용하세요:

```bash
gaji init --migrate
```

그런 다음 생성된 TypeScript를 다듬으세요.

### 3. 브랜치에서 테스트

마이그레이션한 워크플로우는 병합 전에 항상 기능 브랜치에서 테스트하세요.

### 4. 전환 기간에 양쪽 유지

마이그레이션 중에는 YAML과 TypeScript 버전을 함께 유지할 수 있습니다:
- 기존 워크플로우는 `.backup` 확장자로 백업 (예: `ci.yml.backup`)
- 새 워크플로우: `workflows/*.ts`에서 `.github/workflows/*.yml`로 생성

## 문제 해결

### 타입이 생성되지 않음

모든 액션이 추가되었는지 확인하세요:

```bash
gaji add actions/checkout@v5
gaji dev
```

### 빌드 실패

TypeScript 오류를 확인하세요:

```bash
npx tsc --noEmit
```

### 생성된 YAML이 원본과 다름

사소한 포맷 차이는 정상입니다. 포맷이 아닌 기능을 검증하세요.

## 다음 단계

- [CLI 레퍼런스](/ko/reference/cli) 읽기
- [예제](/ko/examples/simple-ci) 보기
