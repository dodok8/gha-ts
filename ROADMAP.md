# gaji Development Roadmap

Type-safe GitHub Actions workflows in TypeScript

---

## Current Status (v0.3.0)

gaji is a working CLI tool with all core features implemented. It is self-dogfooding (uses itself for its own CI/CD workflows) and has been released on both crates.io and npm.

**Completed milestones:**
- Core pipeline: TypeScript -> Parse (oxc) -> Execute (QuickJS) -> YAML
- Type generation from `action.yml` files with caching
- File watching with debounce for development workflow
- `init` command with interactive mode, migration, project state detection
- npm distribution with platform-specific binary packages
- CI/CD automation via release-plz
- VitePress documentation site with English/Korean i18n (https://gaji.gaebalgom.work)
- 96 tests (92 unit + 4 integration)
- JavaScriptAction class for node-based GitHub Actions
- npx support for running without installation
- Build timing, progress bars, cache expiration policy
- CompositeJob class for reusable job templates via class inheritance

---

## Completed Phases

### Phase 0-4: Core (v0.1.0)
- Project setup, CLI framework (clap), TypeScript parsing (oxc AST visitor)
- GitHub API integration (fetch action.yml, retry, rate limiting, GitHub Enterprise)
- Type generation (.d.ts) with JSDoc, caching (.gaji-cache.json)
- File watching (notify crate, debounce, exclude patterns)
- YAML build system (QuickJS execution, JSON->YAML, validation)

### Phase 5: init Command (v0.1.1)
- Smart project state detection (empty / existing Node / has workflows)
- Interactive mode (dialoguer), YAML->TypeScript migration
- Configuration file generation (.gaji.toml, tsconfig.json, .gitignore)

### Phase 6: npm Package (v0.1.2-v0.2.4)
- Platform-specific binary packages (@aspect8/gaji-{platform}-{arch})
- postinstall binary download, execution wrapper
- TypeScript runtime library (Workflow, Job, CallAction, CompositeAction, JavaScriptAction)

### Phase 7: CI/CD Self-Dogfooding (v0.2.0+)
- 7 workflow files: ci, release, release-plz, audit, js, update-workflows, vitepress
- All CI/CD workflows written in TypeScript using gaji itself

### Phase 8: Testing
- 93 tests across all modules (builder, watcher, cache, config, fetcher, parser, generator, executor, init, migration, integration)

### Phase 9: Documentation (v0.2.6)
- VitePress site deployed at https://gaji.gaebalgom.work
- Full English and Korean documentation (guide, reference, examples)

### Phase 10: Polish & UX (v0.3.0)
- Execution time measurement for builds (`Instant` timer in build pipeline)
- Progress indicators with `indicatif` (action download, type generation)
- Cache expiration policy (enforce `is_expired()` with configurable TTL via `build.cache_ttl_days`)
- CompositeJob class for reusable job templates via TypeScript class inheritance

---

## Remaining Work

### Polish & Quality
- [ ] Parallel action.yml downloads (futures::stream::buffer_unordered)
- [ ] Memory/LRU cache for action metadata
- [ ] Better error messages with suggestions ("Did you mean...?")
- [ ] YAML lint rules and warnings (unnamed jobs, unknown fields)
- [ ] Template literal support in `getAction()` calls
- [ ] Respect `.gitignore` patterns when scanning

### Documentation Gaps (implemented but undocumented)
- [x] Document `CallJob` class (reusable workflow calls via `uses`)
- [x] Document `Job.when()`, `Job.permissions()`, `Job.continueOnError()`, `Job.timeoutMinutes()` methods
- [x] Document `Workflow.fromObject()` static method
- [x] Document `Job` constructor optional `options` parameter
- [x] Document `CompositeJob` class in guide/examples
- [ ] Docs: search, hover to show type

### Distribution
- [ ] Shell completion scripts (bash, zsh, fish via clap)

### Feature
- [ ] Action migration (`action.yml` → TypeScript using CompositeAction/JavaScriptAction)

### Community
- [x] GitHub issue templates (bug report, feature request)
- [x] GitHub PR template
- [x] CONTRIBUTING.md
- [x] Code of Conduct

### Marketing & Launch
- [ ] Blog post article
- [ ] Hacker News / Reddit / Twitter announcement

### Future Ideas
- [ ] Plugin system (Another Language Support, in 1.0)
- [ ] Union type inference from action.yml inputs
- [ ] UPX binary compression for smaller npm packages

---

## Version History

| Version | Date | Highlights |
|---------|------|------------|
| v0.3.0 | 2026-02-14 | Build timing, progress bars, cache expiration policy |
| v0.2.8 | 2026-02-13 | npx support |
| v0.2.7 | 2026-02-13 | JavaScriptAction class |
| v0.2.6 | 2026-02-13 | Documentation site, expression escaping fix |
| v0.2.5 | 2026-02-12 | Output directory restructuring |
| v0.1.x | 2026-02 | Initial releases, npm distribution, core features |
