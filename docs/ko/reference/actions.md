# 액션 레퍼런스

gaji와 함께 GitHub Actions를 사용하는 방법입니다.

## 액션 추가

워크플로우에서 액션을 사용하려면 먼저 추가하세요:

```bash
gaji add actions/checkout@v4
```

이것은 액션의 `action.yml`을 가져와 TypeScript 타입을 생성합니다.

## 액션 사용

`getAction()`으로 액션을 가져와 사용하세요:

```typescript
import { getAction } from "../generated/index.js";

const checkout = getAction("actions/checkout@v4");
const setupNode = getAction("actions/setup-node@v4");

// 워크플로우에서 사용
const job = new Job("ubuntu-latest")
  .addStep(checkout({}))
  .addStep(setupNode({
    with: {
      "node-version": "20",  // ✅ 타입 안전!
    },
  }));
```

## 타입 안전성

gaji는 액션의 `action.yml`에서 타입을 생성하여 다음을 제공합니다:

### 자동완성

에디터가 사용 가능한 모든 입력을 표시합니다:

```typescript
setupNode({
  with: {
    // Ctrl+Space를 눌러 모든 옵션 보기:
    // - node-version
    // - cache
    // - cache-dependency-path
    // - architecture
    // 등등.
  },
})
```

### 타입 체크

잘못된 입력은 즉시 잡힙니다:

```typescript
// ❌ 타입 오류 - 알 수 없는 속성
setupNode({
  with: {
    "node-versoin": "20",  // 오타!
  },
})

// ✅ 올바름
setupNode({
  with: {
    "node-version": "20",
  },
})
```

## 일반 액션

### actions/checkout

저장소 체크아웃:

```bash
gaji add actions/checkout@v4
```

```typescript
const checkout = getAction("actions/checkout@v4");

// 기본 사용
.addStep(checkout({}))

// 옵션과 함께
.addStep(checkout({
  with: {
    repository: "owner/repo",
    ref: "main",
    "fetch-depth": 0,
  },
}))
```

### actions/setup-node

Node.js 설정:

```bash
gaji add actions/setup-node@v4
```

```typescript
const setupNode = getAction("actions/setup-node@v4");

.addStep(setupNode({
  with: {
    "node-version": "20",
    cache: "npm",
  },
}))
```

## 다음 단계

- [예제](/ko/examples/simple-ci) 보기
- [CLI 레퍼런스](./cli.md) 확인
