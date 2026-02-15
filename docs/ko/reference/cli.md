# CLI 레퍼런스

모든 gaji CLI 명령에 대한 완전한 레퍼런스입니다.

## 명령어

### `gaji init`

새 gaji 프로젝트를 초기화합니다.

```bash
gaji init [옵션]
```

**옵션.**

| 옵션 | 설명 |
|------|------|
| `--force` | 기존 파일 덮어쓰기 |
| `--skip-examples` | 예제 워크플로우 생성 건너뛰기 |
| `--migrate` | 기존 YAML 워크플로우를 TypeScript로 마이그레이션 |
| `-i, --interactive` | 프롬프트와 함께 대화형 모드 |

**예제.**

```bash
# 기본 초기화
gaji init

# 마이그레이션 포함
gaji init --migrate

# 대화형 모드
gaji init --interactive

# 강제 덮어쓰기
gaji init --force
```

**동작.**

- `workflows/` 디렉토리 생성
- `generated/` 디렉토리 생성
- `.github/workflows/` 디렉토리 생성
- `.gitignore` 업데이트
- 예제 워크플로우 생성 (`--skip-examples` 제외)
- 기존 워크플로우 마이그레이션 (`--migrate` 사용 시)

---

### `gaji dev`

워크플로우 파일을 분석하고 액션에 대한 타입을 생성합니다.

```bash
gaji dev [옵션]
```

**옵션.**

| 옵션 | 설명 |
|------|------|
| `-d, --dir <DIR>` | 스캔할 디렉토리 (기본값. `workflows`) |
| `--watch` | 초기 스캔 후 변경 사항 계속 감시 |

**예제.**

```bash
# 일회성 스캔
gaji dev

# 감시 모드 (개발에 권장)
gaji dev --watch

# 커스텀 디렉토리 스캔
gaji dev --dir src/workflows
```

**동작.**

- `workflows/`의 모든 `.ts` 파일 스캔
- `getAction()` 호출 추출
- GitHub에서 `action.yml` 가져오기
- `generated/`에 TypeScript 타입 생성
- 캐시 업데이트 (`.gaji-cache.json`)

**감시 모드.**

감시 모드에서 gaji는 워크플로우 파일을 지속적으로 모니터링합니다. `getAction()`으로 새 액션을 추가하면 타입이 자동으로 생성됩니다.

---

### `gaji build`

TypeScript 워크플로우를 YAML로 빌드합니다.

```bash
gaji build [옵션]
```

**옵션.**

| 옵션 | 설명 |
|------|------|
| `-i, --input <DIR>` | TypeScript 워크플로우가 있는 입력 디렉토리 (기본값. `workflows`) |
| `-o, --output <DIR>` | YAML 파일 출력 디렉토리 (기본값. `.github`) |
| `--dry-run` | 파일 작성 없이 YAML 출력 미리보기 |

**예제.**

```bash
# 모든 워크플로우 빌드
gaji build

# 작성 없이 미리보기
gaji build --dry-run

# 커스텀 입출력 디렉토리
gaji build --input src/workflows --output .github
```

... tip
검증과 포맷 옵션은 CLI 플래그가 아닌 `.gaji.toml`에서 설정합니다. [설정](../guide/configuration.md)을 참조하세요.
...

**동작.**

- `workflows/`의 모든 `.ts` 파일 찾기
- 내장 QuickJS 엔진으로 실행 (`npx tsx` 폴백)
- 출력을 YAML로 변환
- 워크플로우를 `.github/workflows/`에 작성
- 컴포지트 액션을 `.github/actions/<이름>/action.yml`에 작성

---

### `gaji add`

GitHub Action을 추가하고 타입을 생성합니다.

```bash
gaji add <ACTION_REF>
```

**인수.**

| 인수 | 설명 |
|------|------|
| `<ACTION_REF>` | GitHub 액션 참조 (예: `actions/checkout@v5`) |

**예제.**

```bash
# 일반 액션 추가
gaji add actions/checkout@v5
gaji add actions/setup-node@v4
gaji add actions/cache@v4

# 서드파티 액션 추가
gaji add softprops/action-gh-release@v1

# 하위 디렉토리의 액션 추가
gaji add docker/setup-buildx-action@v3
```

**동작.**

- GitHub에서 `action.yml` 가져오기
- 입력, 출력, 메타데이터 파싱
- TypeScript 타입 생성
- `generated/`에 저장
- 캐시 업데이트

---

### `gaji clean`

생성된 파일과 캐시를 정리합니다.

```bash
gaji clean [옵션]
```

**옵션.**

| 옵션 | 설명 |
|------|------|
| `--cache` | 캐시도 함께 정리 |

**예제.**

```bash
# 생성된 파일 정리
gaji clean

# 캐시도 함께 정리
gaji clean --cache
```

**동작.**

- `generated/` 디렉토리 제거
- `--cache` 사용 시. `.gaji-cache.json`도 제거

모든 타입을 처음부터 다시 생성하고 싶을 때 사용합니다.

---

### `gaji --version`

gaji 버전을 표시합니다.

```bash
gaji --version
```

---

### `gaji --help`

도움말 메시지를 표시합니다.

```bash
gaji --help

# 특정 명령의 도움말 표시
gaji init --help
gaji dev --help
```

## 일반적인 워크플로우

### 초기 설정

```bash
# 설치
npm install -D gaji

# 초기화
gaji init

# 액션 추가
gaji add actions/checkout@v5
gaji add actions/setup-node@v4

# 타입 생성
gaji dev
```

### 개발

```bash
# 터미널 1. 감시 모드
gaji dev --watch

# 터미널 2. 워크플로우 편집
# (workflows/ci.ts 편집)

# 터미널 2. 빌드
gaji build
```

### 클린 빌드

```bash
# 모든 생성된 파일 제거
gaji clean --cache

# 타입 재생성
gaji dev

# 워크플로우 빌드
gaji build
```

## 종료 코드

| 코드 | 의미 |
|------|------|
| 0 | 성공 |
| 1 | 일반 오류 |
| 2 | 파싱 오류 |
| 3 | 네트워크 오류 |
| 4 | 검증 오류 |

## 환경 변수

### `GITHUB_TOKEN`

인증된 API 요청을 위한 GitHub 토큰 설정 (rate limit 증가).

```bash
export GITHUB_TOKEN=ghp_your_token_here
gaji dev
```

## 설정 파일

명령어는 `.gaji.toml`의 설정을 따릅니다. 자세한 내용은 [설정](../guide/configuration.md)을 참조하세요.

## 문제 해결

### "Action not found"

액션 참조가 올바른지 확인하세요.

```bash
# ✅ 올바름
gaji add actions/checkout@v5

# ❌ 잘못됨
gaji add checkout  # owner와 버전이 누락됨
```

### "Network error"

인터넷 연결을 확인하세요. 프록시 뒤에 있다면 설정하세요. reqwest 크레이트에 환경변수를 인식합니다.

```bash
export HTTP_PROXY=http.//proxy.example.com.8080
export HTTPS_PROXY=http.//proxy.example.com.8080
```

### "Types not generated"

액션 추가 후 `gaji dev`를 실행했는지 확인하세요.

```bash
gaji dev  # 잊지 마세요!
```

## 다음 단계

- [TypeScript API](./api.md)에 대해 알아보기
- [설정](../guide/configuration.md) 보기
