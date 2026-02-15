# 설치

선호도와 환경에 따라 여러 방법으로 gaji를 설치할 수 있습니다.

::: tip JS 런타임 불필요
gaji는 QuickJS를 내장하고 있어 Node.js나 외부 JavaScript 런타임을 설치할 필요가 없습니다. 독립 실행형 바이너리로 작동합니다!
:::

## npm

```bash
npm install -D gaji
```

이렇게 하면 gaji가 개발 의존성으로 설치됩니다:

```bash
gaji --help
```

### 다른 패키지 매니저

```bash
# pnpm
pnpm add -D gaji

# yarn
yarn add -D gaji
```

## Cargo

Rust가 설치되어 있다면 crates.io에서 직접 gaji를 설치할 수 있습니다:

```bash
cargo install gaji
```

이렇게 하면 gaji 바이너리가 전역으로 설치됩니다:

```bash
gaji --help
```

## 바이너리 다운로드

[GitHub Releases](https://github.com/dodok8/gaji/releases)에서 미리 빌드된 바이너리를 다운로드합니다.

### Linux (x64)

```bash
curl -L https://github.com/dodok8/gaji/releases/latest/download/gaji-linux-x64.tar.gz | tar xz
sudo mv gaji /usr/local/bin/
```

### macOS (ARM64)

```bash
curl -L https://github.com/dodok8/gaji/releases/latest/download/gaji-darwin-arm64.tar.gz | tar xz
sudo mv gaji /usr/local/bin/
```

### macOS (x64)

```bash
curl -L https://github.com/dodok8/gaji/releases/latest/download/gaji-darwin-x64.tar.gz | tar xz
sudo mv gaji /usr/local/bin/
```

### Windows

릴리스 페이지에서 `gaji-win32-x64.tar.gz`를 다운로드하고 PATH의 디렉토리에 압축을 풉니다.

## 설치 확인

```bash
gaji --version #가지 버전이 출력됩니다.
```

## 요구사항

gaji는 **런타임 의존성이 없습니다**:

- ✅ Node.js 불필요 — gaji는 QuickJS를 내장하고 있습니다
- ✅ 외부 JavaScript 런타임 불필요
- ✅ 어떤 언어나 빌드 도구와도 함께 작동

npm으로 설치하는 경우에만 패키지 매니저가 필요합니다:

- **npm/pnpm/yarn** (npm 설치 시에만)

## 업데이트

### npm

```bash
npm update gaji
```

### Cargo

```bash
cargo install gaji --force
```

### 바이너리

GitHub Releases에서 최신 버전을 다운로드하여 바이너리를 교체합니다.

## 다음 단계

설치가 완료되면 [빠른 시작](./getting-started.md)으로 이동하여 첫 번째 프로젝트를 설정하세요.
