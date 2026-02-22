# 설정

gaji는 TypeScript 설정 파일을 사용하여 자동완성이 되는 타입 안전한 설정을 지원합니다.

## 설정 파일

프로젝트 루트에 `gaji.config.ts`를 생성합시다. `gaji init`을 통해 자동으로 생성할 수 있습니다.

```typescript
import { defineConfig } from "./generated/index.js";

export default defineConfig({
    workflows: "workflows",
    output: ".github",
    generated: "generated",
    watch: {
        debounce: 300,
        ignore: ["node_modules", ".git", "generated"],
    },
    build: {
        validate: true,
        format: true,
    },
});
```

`defineConfig` 함수는 모든 필드에 대한 자동완성과 타입 검사를 제공합니다. 런타임에는 인자를 그대로 반환합니다.

## 설정 옵션

### 프로젝트 디렉토리

| 옵션 | 타입 | 기본값 | 설명 |
|------|------|--------|------|
| `workflows` | string | `"workflows"` | TypeScript 워크플로우가 있는 디렉토리. `gaji dev`와 `gaji build`의 `--input` 기본값으로 사용 |
| `output` | string | `".github"` | 기본 출력 디렉토리 (워크플로우는 `workflows/`, 액션은 `actions/`에 저장). `gaji build`의 `--output` 기본값으로 사용 |
| `generated` | string | `"generated"` | 생성된 액션 타입용 디렉토리 |

**예제:**

```typescript
export default defineConfig({
    workflows: "gha",
    output: ".github",
    generated: "gha-types",
});
```

### `github`

GitHub API 설정:

| 옵션 | 타입 | 기본값 | 설명 |
|------|------|--------|------|
| `token` | string (선택) | None | GitHub 개인 액세스 토큰 |
| `apiUrl` | string (선택) | `"https://github.com"` | GitHub 기본 URL (Enterprise용) |

**토큰 우선순위:**

1. `GITHUB_TOKEN` 환경 변수 (최우선)
2. `gaji.config.local.ts`의 `token`
3. `gaji.config.ts`의 `token`

**GitHub Enterprise 예제:**

```typescript
export default defineConfig({
    github: {
        token: "ghp_your_token_here",
        apiUrl: "https://github.example.com",
    },
});
```

**설정 가능한 항목:**

- **GitHub 토큰**: API 요청 인증 (rate limit 증가, 프라이빗 저장소 접근)
- **GitHub Enterprise**: 자체 호스팅 GitHub 인스턴스 지정
- **액션 가져오기**: 프라이빗 또는 엔터프라이즈 GitHub에서 `action.yml` 가져오기

**참고:** 보안을 위해 토큰은 `gaji.config.ts` 대신 `gaji.config.local.ts`(gitignored)에 저장하세요

### `watch`

파일 감시 설정:

| 옵션 | 타입 | 기본값 | 설명 |
|------|------|--------|------|
| `debounce` | number | `300` | 밀리초 단위 디바운스 지연 |
| `ignore` | string[] | `["node_modules", ".git", "generated"]` | 무시할 패턴 |

`ignore` 설정은 `gaji dev` (watch 모드)와 `gaji build` 명령어 모두에서 사용됩니다. 이 패턴과 일치하는 파일은 처리에서 제외됩니다. 매칭은 단순 문자열 포함 여부로 판단됩니다 - 파일 경로에 패턴이 포함되어 있으면 해당 파일은 무시됩니다.

**예제:**

```typescript
export default defineConfig({
    watch: {
        debounce: 500,
        ignore: ["node_modules", ".git", "generated", "dist", "coverage"],
    },
});
```

**일반적으로 무시하는 패턴:**

- `node_modules` - npm 의존성
- `.git` - Git 내부 파일
- `generated` - gaji가 생성한 타입 파일
- `dist` - 빌드 출력 디렉토리
- `coverage` - 테스트 커버리지 리포트

### `build`

빌드 설정:

| 옵션 | 타입 | 기본값 | 설명 |
|------|------|--------|------|
| `validate` | boolean | `true` | 생성된 YAML 검증 |
| `format` | boolean | `true` | 생성된 YAML 포맷 |
| `cacheTtlDays` | number | `30` | 액션 메타데이터 캐시 TTL (일 단위) |

**예제:**

```typescript
export default defineConfig({
    build: {
        validate: true,
        format: true,
        cacheTtlDays: 14,
    },
});
```

## 로컬 설정

토큰 같은 민감한 값은 `gaji.config.local.ts`에 작성합시다. 이 파일은 gitignore에 추가해야 합니다.

```typescript
import { defineConfig } from "./generated/index.js";

export default defineConfig({
    github: {
        token: "ghp_your_token_here",
        apiUrl: "https://github.example.com",
    },
});
```

## TypeScript 설정

gaji는 표준 TypeScript 설정과 호환됩니다. `tsconfig.json`에 생성된 타입이 포함되도록 설정하세요.

```json
{
  "compilerOptions": {
    "target": "ES2020",
    "module": "commonjs",
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "typeRoots": [
      "./node_modules/@types",
      "./generated"
    ]
  },
  "include": ["workflows/**/*"],
  "exclude": ["node_modules", "dist", "generated"]
}
```

## `.gitignore`

gaji가 생성한 파일을 `.gitignore`에 추가:

```gitignore
# gaji
generated/
.gaji-cache.json
gaji.config.local.ts
```

**참고:** `.github/workflows/`는 무시하지 마세요. 이것은 GitHub Actions가 실제로 사용하는 워크플로우 파일입니다.

## 하위 호환성

`gaji.config.ts`가 없으면 gaji는 `.gaji.toml`과 `.gaji.local.toml`을 읽습니다. TOML 설정을 사용하는 기존 프로젝트는 변경 없이 계속 동작합니다.

## 캐시

gaji는 액션 정의를 다시 가져오지 않도록 캐시 파일(`.gaji-cache.json`)을 사용합니다. 이 파일은 자동으로 관리되며 gitignore에 추가해야 합니다.

캐시를 지우려면 `clean` 명령어를 사용하세요.

```bash
gaji clean --cache
```

## 다음 단계

- [마이그레이션](./migration.md)에 대해 알아보기
- [CLI 레퍼런스](/ko/reference/cli) 확인
