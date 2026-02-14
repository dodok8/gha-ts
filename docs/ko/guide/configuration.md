# 설정

프로젝트 루트에 `.gaji.toml` 파일을 사용하여 gaji를 설정할 수 있습니다.

## 설정 파일

프로젝트 루트에 `.gaji.toml` 생성:

```toml
[project]
workflows_dir = "workflows"
output_dir = ".github"
generated_dir = "generated"

[github]
# 선택사항: GitHub 토큰 (GITHUB_TOKEN 환경 변수로도 설정 가능)
token = "ghp_your_token_here"
# GitHub Enterprise 사용자용
api_url = "https://github.example.com"

[watch]
debounce_ms = 300
ignored_patterns = ["node_modules", ".git", "generated"]

[build]
validate = true
format = true
```

## 설정 옵션

### `[project]`

프로젝트 수준 설정:

| 옵션 | 타입 | 기본값 | 설명 |
|------|------|--------|------|
| `workflows_dir` | string | `"workflows"` | TypeScript 워크플로우가 있는 디렉토리 |
| `output_dir` | string | `".github"` | 기본 출력 디렉토리 (워크플로우는 `workflows/`, 액션은 `actions/`에 저장) |
| `generated_dir` | string | `"generated"` | 생성된 액션 타입용 디렉토리 |

**예제:**

```toml
[project]
workflows_dir = "gha"
output_dir = ".github"
generated_dir = "gha-types"
```

### `[github]`

GitHub API 설정:

| 옵션 | 타입 | 기본값 | 설명 |
|------|------|--------|------|
| `token` | string (선택) | None | GitHub 개인 액세스 토큰 |
| `api_url` | string (선택) | `"https://github.com"` | GitHub 기본 URL (Enterprise용) |

**토큰 우선순위:**
1. `GITHUB_TOKEN` 환경 변수 (최우선)
2. `.gaji.local.toml`의 `token`
3. `.gaji.toml`의 `token`

**GitHub Enterprise 예제:**

```toml
[github]
token = "ghp_your_token_here"
api_url = "https://github.example.com"
```

**설정 가능한 항목:**
- **GitHub 토큰**: API 요청 인증 (rate limit 증가, 프라이빗 저장소 접근)
- **GitHub Enterprise**: 자체 호스팅 GitHub 인스턴스 지정
- **액션 가져오기**: 프라이빗 또는 엔터프라이즈 GitHub에서 `action.yml` 가져오기

**참고:** 보안을 위해 토큰은 `.gaji.toml` 대신 `.gaji.local.toml`(gitignored)에 저장하세요

### `[watch]`

파일 감시 설정:

| 옵션 | 타입 | 기본값 | 설명 |
|------|------|--------|------|
| `debounce_ms` | integer | `300` | 밀리초 단위 디바운스 지연 |
| `ignored_patterns` | array | `["node_modules", ".git", "generated"]` | 무시할 패턴 |

`ignored_patterns` 설정은 `gaji dev` (watch 모드)와 `gaji build` 명령어 모두에서 사용됩니다. 이 패턴과 일치하는 파일은 처리에서 제외됩니다. 매칭은 단순 문자열 포함 여부로 판단됩니다 - 파일 경로에 패턴이 포함되어 있으면 해당 파일은 무시됩니다.

**예제:**

```toml
[watch]
debounce_ms = 500
ignored_patterns = ["node_modules", ".git", "generated", "dist", "coverage"]
```

**일반적으로 무시하는 패턴:**

- `node_modules` - npm 의존성
- `.git` - Git 내부 파일
- `generated` - gaji가 생성한 타입 파일
- `dist` - 빌드 출력 디렉토리
- `coverage` - 테스트 커버리지 리포트

### `[build]`

빌드 설정:

| 옵션 | 타입 | 기본값 | 설명 |
|------|------|--------|------|
| `validate` | boolean | `true` | 생성된 YAML 검증 |
| `format` | boolean | `true` | 생성된 YAML 포맷 |

**예제:**

```toml
[build]
validate = true
format = true
```

## TypeScript 설정

gaji는 표준 TypeScript 설정과 호환됩니다. `tsconfig.json`에 생성된 타입이 포함되도록 설정하세요:

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
```

**참고:** `.github/workflows/`는 무시하지 마세요. 이것은 GitHub Actions가 실제로 사용하는 워크플로우 파일입니다.

## 캐시

gaji는 액션 정의를 다시 가져오지 않도록 캐시 파일(`.gaji-cache.json`)을 사용합니다. 이 파일은 자동으로 관리되며 gitignore에 추가해야 합니다.

캐시를 지우려면:

```bash
gaji clean --cache
```

## 환경 변수

### `GITHUB_TOKEN`

인증된 요청을 위한 GitHub 토큰 설정 (rate limit 증가):

```bash
export GITHUB_TOKEN=ghp_your_token_here
gaji dev
```

## 다음 단계

- [마이그레이션](./migration.md)에 대해 알아보기
- [CLI 레퍼런스](/ko/reference/cli) 확인
