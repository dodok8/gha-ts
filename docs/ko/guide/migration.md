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

워크플로우와 액션을 자동으로 감지합니다. `runs.using` 필드에 따라 `CompositeAction`, `JavaScriptAction`, `DockerAction` 클래스로 변환됩니다.

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
import { getAction, CompositeAction } from "../generated/index.js";

const checkout = getAction("actions/checkout@v5");
const cache = getAction("actions/cache@v4");

const action = new CompositeAction({
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
    .addStep(checkout({}))
    .addStep({
        name: "Install dependencies",
        run: "npm ci",
        shell: "bash",
    })
    .addStep(cache({
        id: "cache",
        name: "Cache node_modules",
        with: {
            path: "node_modules",
            key: "${{ runner.os }}-node-${{ hashFiles('**/package-lock.json') }}",
        },
    }));

action.build("setup-env");
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
import { JavaScriptAction } from "../generated/index.js";

const action = new JavaScriptAction(
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
| Composite  | `composite`                  | `CompositeAction`   |
| JavaScript | `node12`, `node16`, `node20` | `JavaScriptAction`  |
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

const checkout = getAction("actions/checkout@v5");
const setupNode = getAction("actions/setup-node@v4");

const build = new Job("ubuntu-latest")
  .addStep(checkout({}))
  .addStep(setupNode({
    with: {
      "node-version": "20",
    },
  }))
  .addStep({ run: "npm ci" })
  .addStep({ run: "npm test" });

const workflow = new Workflow({
  name: "CI",
  on: {
    push: {
      branches: ["main"],
    },
  },
}).addJob("build", build);

workflow.build("ci");
```

### 4단계: 빌드 및 검증

```bash
# TypeScript를 YAML로 빌드
gaji build
```

## 다음 단계

- [CLI 레퍼런스](/ko/reference/cli) 읽기
- [예제](/ko/examples/simple-ci) 보기
