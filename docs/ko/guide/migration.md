# 마이그레이션

이 가이드는 기존 YAML 워크플로우를 gaji를 사용한 TypeScript로 마이그레이션하는 데 도움을 줍니다.

## 자동 마이그레이션

gaji는 기존 YAML 워크플로우를 자동으로 TypeScript로 변환할 수 있습니다:

```bash
gaji init --migrate
```

이것은:
1. `.github/workflows/`에서 기존 YAML 워크플로우 감지
2. `workflows/`에서 TypeScript로 변환
3. 원본 YAML 파일 백업 (`.yml.backup`)
4. 사용된 모든 액션에 대한 타입 생성

## 수동 마이그레이션

수동으로 마이그레이션하려면 다음 단계를 따르세요:

### 1단계: YAML 분석

간단한 YAML 워크플로우로 시작:

```yaml
name: CI
on:
  push:
    branches: [main]
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: '20'
      - run: npm ci
      - run: npm test
```

### 2단계: 필요한 액션 추가

```bash
gaji add actions/checkout@v4
gaji add actions/setup-node@v4
```

### 3단계: TypeScript로 변환

`workflows/ci.ts` 생성:

```typescript
import { getAction, Job, Workflow } from "../generated/index.js";

const checkout = getAction("actions/checkout@v4");
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

# 원본과 비교
diff .github/workflows/ci.yml .github/workflows/ci.yml.backup
```

## 다음 단계

- [CLI 레퍼런스](/ko/reference/cli) 읽기
- [예제](/ko/examples/simple-ci) 보기
