# 빠른 시작

이 가이드는 몇 분 안에 gaji를 시작하는 방법을 안내합니다.

## 설치

npm을 사용하여 gaji 설치 (권장):

```bash
npm install -D gaji
```

또는 다른 방법 사용:

```bash
# cargo 사용
cargo install gaji

# pnpm 사용
pnpm add -D gaji

# yarn 사용
yarn add -D gaji
```

더 많은 옵션은 [설치](./installation.md)를 참조하세요.

## 프로젝트 초기화

init 명령을 실행하여 프로젝트에서 gaji 설정:

```bash
npx gaji init
```

이것은:
- TypeScript 워크플로우를 위한 `workflows/` 디렉토리 생성
- 자동 생성된 타입을 위한 `generated/` 디렉토리 생성
- 컴파일된 YAML을 위한 `.github/workflows/` 디렉토리 생성
- 예제 워크플로우 생성 (선택 사항)
- `.gitignore` 업데이트

## 액션 추가

사용하려는 GitHub Actions 추가:

```bash
npx gaji add actions/checkout@v4
npx gaji add actions/setup-node@v4
```

이것은:
- GitHub에서 `action.yml` 가져오기
- TypeScript 타입 생성
- `generated/` 디렉토리에 저장

## 첫 번째 워크플로우 작성

`workflows/ci.ts` 생성:

```typescript
import { getAction, Job, Workflow } from "../generated/index.js";

// 완전한 타입 안전성으로 액션 가져오기
const checkout = getAction("actions/checkout@v4");
const setupNode = getAction("actions/setup-node@v4");

// 빌드 작업 정의
const build = new Job("ubuntu-latest")
  .addStep(checkout({
    name: "코드 체크아웃",
  }))
  .addStep(setupNode({
    name: "Node.js 설정",
    with: {
      "node-version": "20",  // ✅ 자동완성 사용 가능!
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
  name: "CI",
  on: {
    push: { branches: ["main"] },
    pull_request: { branches: ["main"] },
  },
}).addJob("build", build);

// YAML 파일 빌드
workflow.build("ci");
```

## 타입 생성 및 빌드

### 방법 1: 일회성 빌드

```bash
# 워크플로우에서 찾은 액션에 대한 타입 생성
npx gaji dev

# 워크플로우를 YAML로 빌드
npx gaji build
```

### 방법 2: 감시 모드 (권장)

```bash
# 감시 모드 시작 - 새 액션을 추가하면 자동으로 타입 생성
npx gaji dev --watch
```

다른 터미널에서:

```bash
# 워크플로우 빌드
npx gaji build
```

생성된 YAML은 `.github/workflows/ci.yml`에 있습니다:

```yaml
# gaji에 의해 자동 생성됨 - 수동으로 편집하지 마세요
name: CI
on:
  push:
    branches:
      - main
  pull_request:
    branches:
      - main
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - name: 코드 체크아웃
        uses: actions/checkout@v4
      - name: Node.js 설정
        uses: actions/setup-node@v4
        with:
          node-version: '20'
      - name: 의존성 설치
        run: npm ci
      - name: 테스트 실행
        run: npm test
```

## 권장 개발 워크플로우

최상의 경험을 위해 다음 워크플로우를 따르세요:

1. **감시 모드 시작**:
   ```bash
   npx gaji dev --watch
   ```
   터미널에서 계속 실행 상태로 두세요.

2. **워크플로우 편집**:
   - 에디터에서 `workflows/ci.ts` 열기
   - 단계 추가 또는 수정
   - `getAction()`으로 새 액션을 추가하면 gaji가 자동으로 타입을 가져와 생성합니다

3. **YAML로 빌드**:
   ```bash
   npx gaji build
   ```

4. **생성된 YAML 검토**:
   - `.github/workflows/ci.yml` 열기
   - 명령이 올바른지 확인
   - 모든 필수 필드가 있는지 확인

5. **두 파일 모두 커밋**:
   ```bash
   git add workflows/ci.ts .github/workflows/ci.yml
   git commit -m "CI 워크플로우 추가"
   ```

## 왜 TypeScript와 YAML 모두 커밋해야 하나요?

TypeScript 소스와 생성된 YAML을 **모두** 커밋해야 합니다:

- **TypeScript** (`workflows/*.ts`): 소스 코드, 버전 관리
- **YAML** (`.github/workflows/*.yml`): GitHub Actions가 실행하는 파일

## 중요: 자동 컴파일

::: warning 중요
push 시 TypeScript를 YAML로 자동 컴파일하는 워크플로우를 만들 수 있지만, **권장하지 않습니다**. 항상 로컬에서 컴파일하고 검토한 후 커밋하세요.

GitHub Actions 트리거의 복잡성(예: `paths` 필터링, PAT 토큰 관리, 무한 루프 방지)을 감수할 의향이 있다면, 자동 컴파일 워크플로우를 구성할 수 있습니다. 동작하는 예시는 [`workflows/update-workflows.ts`](https://github.com/dodok8/gaji/blob/main/workflows/update-workflows.ts)를 참고하세요.
:::

## 다음 단계

- [워크플로우 작성](./writing-workflows.md)에 대해 자세히 알아보기
- [CLI 레퍼런스](/ko/reference/cli) 살펴보기
- [예제](/ko/examples/simple-ci) 확인하기
