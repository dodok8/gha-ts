# CLI 레퍼런스

모든 gaji CLI 명령에 대한 완전한 레퍼런스입니다.

## 명령어

### `gaji init`

새 gaji 프로젝트를 초기화합니다.

```bash
gaji init [옵션]
```

**옵션:**

| 옵션 | 설명 |
|------|------|
| `--force` | 기존 파일 덮어쓰기 |
| `--skip-examples` | 예제 워크플로우 생성 건너뛰기 |
| `--migrate` | 기존 YAML 워크플로우를 TypeScript로 마이그레이션 |
| `-i, --interactive` | 프롬프트와 함께 대화형 모드 |

**예제:**

```bash
# 기본 초기화
gaji init

# 마이그레이션 포함
gaji init --migrate

# 대화형 모드
gaji init --interactive
```

---

### `gaji dev`

워크플로우 파일을 분석하고 액션에 대한 타입을 생성합니다.

```bash
gaji dev [옵션]
```

**옵션:**

| 옵션 | 설명 |
|------|------|
| `--watch` | 초기 스캔 후 변경 사항 계속 감시 |

**예제:**

```bash
# 일회성 스캔
gaji dev

# 감시 모드 (개발에 권장)
gaji dev --watch
```

---

### `gaji build`

TypeScript 워크플로우를 YAML로 빌드합니다.

```bash
gaji build
```

---

### `gaji add`

GitHub Action을 추가하고 타입을 생성합니다.

```bash
gaji add <ACTION_REF>
```

**예제:**

```bash
# 일반 액션 추가
gaji add actions/checkout@v4
gaji add actions/setup-node@v4
```

---

### `gaji clean`

생성된 파일과 캐시를 정리합니다.

```bash
gaji clean
```

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
```

## 다음 단계

- [TypeScript API](./api.md)에 대해 알아보기
- [설정](../guide/configuration.md) 보기
