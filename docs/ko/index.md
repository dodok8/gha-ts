---
layout: home

hero:
  name: gaji
  text: 타입 안전한 GitHub Actions
  tagline: TypeScript로 완전한 타입 안전성을 갖춘 GitHub Actions 워크플로우 작성
  image:
    src: /logo.png
    alt: gaji
  actions:
    - theme: brand
      text: 시작하기
      link: /ko/guide/getting-started
    - theme: alt
      text: GitHub에서 보기
      link: https://github.com/dodok8/gaji

features:
  - icon: 🔒
    title: 완전한 타입 안전성
    details: 모든 액션의 입력과 출력에 대한 자동완성 및 타입 체크와 함께 TypeScript로 워크플로우를 작성합니다.

  - icon: ⚡
    title: 빠른 개발
    details: 파일 감시 기능으로 자동 타입 생성. 워크플로우를 변경하면 즉시 결과를 확인할 수 있습니다.

  - icon: 🎯
    title: 제로 설정
    details: action.yml 자동 가져오기 및 타입 생성. getAction()만 사용하면 바로 시작할 수 있습니다.

  - icon: 🦀
    title: 단일 바이너리
    details: 최고의 성능을 위해 Rust로 빌드. QuickJS를 내장하여 Node.js나 외부 JS 런타임 없이 실행 가능.

  - icon: 📄
    title: 독립 실행형 TypeScript
    details: 생성된 워크플로우 파일은 완전히 독립적입니다. 어떤 TS 런타임(tsx, ts-node, Deno)으로도 실행하여 JSON 출력 가능.

  - icon: 📦
    title: 모든 곳에서 작동
    details: 어떤 언어나 빌드 도구와도 함께 사용 가능. 기존 프로젝트 구조 변경 불필요.

  - icon: 🔄
    title: 쉬운 마이그레이션
    details: 단일 명령으로 기존 YAML 워크플로우를 TypeScript로 마이그레이션.
---

## gaji란?

**gaji**는 **G**itHub **A**ctions **J**ustified **I**mprovements의 약자입니다.

이름은 또한 한국어 "가지"에서 따왔습니다 🍆 - 어떤 요리든 더 맛있게 만드는 다재다능한 재료처럼, 이 도구도 GitHub Actions 워크플로우를 더 좋게 만듭니다!

## 왜 gaji인가?

YAML로 GitHub Actions 워크플로우를 작성하는 것은 오류가 발생하기 쉽고 타입 안전성이 없습니다. gaji를 사용하면 TypeScript로 워크플로우를 작성할 수 있으며:

### 이전 (YAML)

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
          node-versoin: '20'  # 키 이름 오타! 런타임까지 오류 없음 ❌
          cache: 'npm'

      - run: npm ci
      - run: npm test
```

### 이후 (gaji를 사용한 TypeScript)

```typescript
import { getAction, Job, Workflow } from "../generated/index.js";

const checkout = getAction("actions/checkout@v4");
const setupNode = getAction("actions/setup-node@v4");

const build = new Job("ubuntu-latest")
  .addStep(checkout({}))
  .addStep(setupNode({
    with: {
      "node-version": "20",  // ✅ 올바른 키 이름, 컴파일 시점에 오류 포착
      cache: "npm",          // ✅ 완전한 자동완성 및 타입 체크
    },
  }))
  .addStep({ run: "npm ci" })
  .addStep({ run: "npm test" });

const workflow = new Workflow({
  name: "CI",
  on: { push: { branches: ["main"] } },
}).addJob("build", build);

workflow.build("ci");
```

## 빠른 시작

### 설치

```bash
# npm으로 설치
npm install -D gaji

# 또는 cargo로 설치
cargo install gaji
```

::: tip JS 런타임 불필요
gaji는 QuickJS를 내장하고 있어 Node.js나 외부 JavaScript 런타임이 필요하지 않습니다. 어떤 언어나 빌드 도구와도 함께 사용 가능합니다!
:::

### 설정

```bash
# 초기화
gaji init

# 액션 추가
gaji add actions/checkout@v4
gaji add actions/setup-node@v4

# 타입 생성
gaji dev

# 워크플로우 빌드
gaji build
```

생성된 YAML이 `.github/workflows/`에 나타나며 바로 사용할 수 있습니다!

::: tip 독립 실행형 TypeScript 파일
워크플로우 TypeScript 파일은 완전히 독립적이고 자체 포함됩니다. 어떤 TypeScript 런타임으로도 직접 실행하여 워크플로우 JSON을 볼 수 있습니다:

```bash
# tsx 사용
npx tsx workflows/ci.ts

# ts-node 사용
npx ts-node workflows/ci.ts

# Deno 사용
deno run workflows/ci.ts
```

디버깅, 검사 또는 다른 도구와의 통합이 쉬워집니다!
:::

## 권장 워크플로우

1. **감시 모드 시작**: `gaji dev --watch`
2. **TypeScript 워크플로우 편집** (`workflows/*.ts`)
3. **YAML로 빌드**: `gaji build`
4. **생성된 YAML 검토** (`.github/workflows/`)
5. **TypeScript와 YAML 모두 커밋**

::: warning 중요
push 시 TypeScript를 YAML로 자동 컴파일하는 워크플로우를 만들 수 있지만, **권장하지 않습니다**. 항상 로컬에서 컴파일하고 검토한 후 커밋하세요.

GitHub Actions 트리거의 복잡성(예: `paths` 필터링, PAT 토큰 관리, 무한 루프 방지)을 감수할 의향이 있다면, 자동 컴파일 워크플로우를 구성할 수 있습니다. 동작하는 예시는 [`workflows/update-workflows.ts`](https://github.com/dodok8/gaji/blob/main/workflows/update-workflows.ts)를 참고하세요.
:::

## 더 알아보기

- [시작하기 가이드](/ko/guide/getting-started)
- [CLI 레퍼런스](/ko/reference/cli)
- [예제](/ko/examples/simple-ci)
