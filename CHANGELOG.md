# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.5](https://github.com/dodok8/gaji/compare/v0.2.4...v0.2.5) - 2026-02-12

### Other

- change output_dir to .github with type-based subdirectories ([#20](https://github.com/dodok8/gaji/pull/20))

## [0.2.4](https://github.com/dodok8/gaji/compare/v0.2.3...v0.2.4) - 2026-02-12

### Fixed

- ensure gaji binary has execute permission after npm install ([#18](https://github.com/dodok8/gaji/pull/18))

## [0.2.3](https://github.com/dodok8/gaji/compare/v0.2.2...v0.2.3) - 2026-02-13

### Fixed

- fix Windows build in release workflow

## [0.2.2](https://github.com/dodok8/gaji/compare/v0.2.1...v0.2.2) - 2026-02-13

### Fixed

- fix npm Trusted Publishers configuration

## [0.2.1](https://github.com/dodok8/gaji/compare/v0.2.0...v0.2.1) - 2026-02-13

### Fixed

- fix npm OIDC auth by removing registry-url from setup-node

## [0.2.0](https://github.com/dodok8/gaji/compare/v0.1.4...v0.2.0) - 2026-02-13

### Other

- add private config file (.gaji.local.toml) for secure token storage
- add tests
- fix CI/CD pipeline issues
- bump version and clean up release process

## [0.1.4](https://github.com/dodok8/gaji/compare/v0.1.3...v0.1.4) - 2026-02-12

### Other

- update Cargo.lock dependencies

## [0.1.3](https://github.com/dodok8/gaji/compare/v0.1.2...v0.1.3) - 2026-02-12

### Other

- fix workflow

## [0.1.2](https://github.com/dodok8/gaji/compare/v0.1.1...v0.1.2) - 2026-02-12

### Fixed

- *(ci)* run version sync after artifact copy in release workflow

## [0.1.1](https://github.com/dodok8/gaji/compare/v0.1.0...v0.1.1) - 2026-02-12

### Added

- add npm package distribution and release automation ([#7](https://github.com/dodok8/gaji/pull/7))
- *(init)* add init command ([#4](https://github.com/dodok8/gaji/pull/4))
- *(build)* add --dry-run to build ([#3](https://github.com/dodok8/gaji/pull/3))
- build YAML ([#2](https://github.com/dodok8/gaji/pull/2))
- remove index.d.ts
- add runtime template for getAction function and update imports
- enhance index file generation with action references and overloads
- make base.d.ts
- implement file watcher and YAML builder (Phase 3-4)
- implement TypeScript type generation (Phase 2)
- implement GitHub API client and caching (Phase 1.4-1.5)
- implement TypeScript parser with oxc (Phase 1.2-1.3)
- implement CLI framework and configuration (Phase 1.1)

### Other

- remove duplicated type signature ([#6](https://github.com/dodok8/gaji/pull/6))
- update generated workflows
- update to checkout@v5
- update generated workflows
- *(audit)* add audit
- give permission to gaji workflow
- fix ci and gaji workflows ([#5](https://github.com/dodok8/gaji/pull/5))
- cargo clippy
- add hello-ouput.ts
- cargo clippy
- mise add deno
- update hello-checkout.ts
- add hello-checkout example
- setup project structure and dependencies (Phase 0)
- Add dependencies
- Initial Commit
