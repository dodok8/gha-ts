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

## 액션 참조 형식

액션은 표준 GitHub 형식으로 참조합니다:

```
owner/repo@version
```

예시:
- `actions/checkout@v4`
- `actions/setup-node@v4`
- `docker/setup-buildx-action@v3`
- `softprops/action-gh-release@v1`

### 버전

다음을 사용할 수 있습니다:
- **태그**: `@v4`, `@v1.2.3`
- **브랜치**: `@main`, `@develop`
- **커밋**: `@a1b2c3d`

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

### 문서

입력 위에 마우스를 올리면 설명과 기본값을 볼 수 있습니다:

```typescript
setupNode({
  with: {
    "node-version": "20",  // 📝 호버 시 설명 표시
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
    token: "${{ secrets.GITHUB_TOKEN }}",
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

### actions/cache

의존성 캐시:

```bash
gaji add actions/cache@v4
```

```typescript
const cache = getAction("actions/cache@v4");

.addStep(cache({
  with: {
    path: "node_modules",
    key: "${{ runner.os }}-node-${{ hashFiles('**/package-lock.json') }}",
    "restore-keys": "${{ runner.os }}-node-",
  },
}))
```

### actions/upload-artifact

빌드 아티팩트 업로드:

```bash
gaji add actions/upload-artifact@v4
```

```typescript
const uploadArtifact = getAction("actions/upload-artifact@v4");

.addStep(uploadArtifact({
  with: {
    name: "build-output",
    path: "dist/",
  },
}))
```

### actions/download-artifact

아티팩트 다운로드:

```bash
gaji add actions/download-artifact@v4
```

```typescript
const downloadArtifact = getAction("actions/download-artifact@v4");

.addStep(downloadArtifact({
  with: {
    name: "build-output",
    path: "dist/",
  },
}))
```

## 서드파티 액션

gaji는 모든 GitHub Action과 호환됩니다:

```bash
# Docker
gaji add docker/setup-buildx-action@v3
gaji add docker/build-push-action@v5

# Rust
gaji add dtolnay/rust-toolchain@stable

# GitHub
gaji add softprops/action-gh-release@v1
```

예제:

```typescript
const setupBuildx = getAction("docker/setup-buildx-action@v3");
const buildPush = getAction("docker/build-push-action@v5");

const job = new Job("ubuntu-latest")
  .addStep(checkout({}))
  .addStep(setupBuildx({}))
  .addStep(buildPush({
    with: {
      context: ".",
      push: true,
      tags: "user/app:latest",
    },
  }));
```

## 로컬 액션

로컬 컴포지트 액션 참조:

```typescript
const myAction = getAction("./my-action");

.addStep(myAction({
  with: {
    input: "value",
  },
}))
```

먼저 액션을 생성해야 합니다. [CompositeAction](./api.md#compositeaction)을 참조하세요.

## 액션 출력

후속 스텝에서 액션 출력을 사용하세요:

```typescript
const setupNode = getAction("actions/setup-node@v4");

.addStep(setupNode({
  id: "setup-node",
  with: {
    "node-version": "20",
  },
}))
.addStep({
  run: "echo Node path: ${{ steps.setup-node.outputs.node-path }}",
})
```

## 액션 업데이트

액션 타입을 업데이트하려면 캐시를 지우고 재생성하세요:

```bash
# 캐시 정리 후 재생성
gaji clean --cache
gaji dev
```

## 문제 해결

### "Action not found"

액션을 추가했는지 확인하세요:

```bash
gaji add actions/checkout@v4
gaji dev
```

### "Types not updated"

캐시를 지우고 재생성하세요:

```bash
gaji clean --cache
gaji dev
```

### "Rate limit exceeded"

GitHub 토큰을 설정하세요:

```bash
export GITHUB_TOKEN=ghp_your_token_here
gaji dev
```

## 다음 단계

- [예제](/ko/examples/simple-ci) 보기
- [CLI 레퍼런스](./cli.md) 확인
