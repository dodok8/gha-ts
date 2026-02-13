# gaji Development Roadmap

Type-safe GitHub Actions workflows in TypeScript

---

## Current Status (v0.2.7)

gaji is a working CLI tool with all core features implemented. It is self-dogfooding (uses itself for its own CI/CD workflows) and has been released on both crates.io and npm.

**Completed milestones:**
- Core pipeline: TypeScript -> Parse (oxc) -> Execute (QuickJS) -> YAML
- Type generation from `action.yml` files with caching
- File watching with debounce for development workflow
- `init` command with interactive mode, migration, project state detection
- npm distribution with platform-specific binary packages
- CI/CD automation via release-plz
- VitePress documentation site with English/Korean i18n (https://gaji.gaebalgom.work)
- 93 tests (90 unit + 3 integration)
- JavaScriptAction class for node-based GitHub Actions

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

---

## Remaining Work

### Polish & Quality
- [ ] Parallel action.yml downloads (futures::stream::buffer_unordered)
- [ ] Memory/LRU cache for action metadata
- [ ] Cache expiration policy (auto re-validate after N days)
- [ ] Better error messages with suggestions ("Did you mean...?")
- [ ] Improved progress indicators (indicatif)
- [ ] Execution time measurement for builds
- [ ] YAML lint rules and warnings (unnamed jobs, unknown fields)
- [ ] Template literal support in `getAction()` calls
- [ ] Respect `.gitignore` patterns when scanning

### Distribution
- [ ] Shell completion scripts (bash, zsh, fish via clap)

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
| v0.2.7 | 2026-02-13 | JavaScriptAction class |
| v0.2.6 | 2026-02-13 | Documentation site, expression escaping fix |
| v0.2.5 | 2026-02-12 | Output directory restructuring |
| v0.1.x | 2026-02 | Initial releases, npm distribution, core features |
