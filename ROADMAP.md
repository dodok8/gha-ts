# gaji Development Roadmap

Type-safe GitHub Actions workflows in TypeScript - Complete Development Plan

---

## Project Overview

**Goal**: Create a CLI tool that allows developers to write GitHub Actions workflows in TypeScript with full type safety, then compile them to YAML.

**Key Features**:
- TypeScript-based workflow authoring
- Automatic type generation from action.yml files
- File watching for development
- Single binary distribution (Rust)
- npm package wrapper for easy installation
- Self-dogfooding (uses itself for CI/CD)

**Tech Stack**:
- Core: Rust
- Parser: oxc (TypeScript AST)
- CLI: clap
- File watching: notify
- HTTP: reqwest
- YAML: serde_yaml

---

## Phase 0: Project Setup

### 0.1 Repository Initialization
- [ ] Create GitHub repository
- [ ] Initialize Rust project (`cargo init`)
- [ ] Setup `.gitignore`
- [ ] Choose license (MIT/Apache-2.0)
- [ ] Write basic README.md structure

### 0.2 Development Environment
- [ ] Configure Cargo.toml dependencies
  ```toml
  [dependencies]
  # CLI
  clap = { version = "4.0", features = ["derive"] }
  
  # File watching
  notify = "6.0"
  
  # HTTP client
  reqwest = { version = "0.11", features = ["blocking", "json"] }
  
  # Serialization
  serde = { version = "1.0", features = ["derive"] }
  serde_yaml = "0.9"
  serde_json = "1.0"
  
  # Async runtime
  tokio = { version = "1", features = ["full"] }
  
  # TypeScript parsing (oxc)
  oxc_parser = "0.9"
  oxc_ast = "0.9"
  oxc_span = "0.9"
  oxc_allocator = "0.9"
  
  # Error handling
  anyhow = "1.0"
  thiserror = "1.0"
  
  # UI
  colored = "2.0"
  indicatif = "0.17"  # progress bar
  
  [dev-dependencies]
  tempfile = "3.8"
  ```

- [ ] Design project structure
  ```
  gaji/
  ├─ src/
  │  ├─ main.rs           # Entry point
  │  ├─ cli.rs            # CLI command definitions
  │  ├─ parser/
  │  │  ├─ mod.rs         # TypeScript parser module
  │  │  ├─ ast.rs         # AST visitor implementation
  │  │  └─ extractor.rs   # Action ref extraction
  │  ├─ fetcher.rs        # GitHub API client
  │  ├─ generator/
  │  │  ├─ mod.rs
  │  │  ├─ types.rs       # TypeScript type generation
  │  │  └─ templates.rs   # Type templates
  │  ├─ watcher.rs        # File watching
  │  ├─ builder.rs        # YAML building
  │  ├─ cache.rs          # Caching system
  │  ├─ config.rs         # Configuration management
  │  └─ lib.rs            # Library exports
  ├─ tests/
  │  ├─ integration/
  │  └─ fixtures/         # Test files
  ├─ examples/            # Example workflows
  └─ Cargo.toml
  ```

---

## Phase 1: Core Features Implementation

### 1.1 CLI Framework
- [ ] Setup CLI structure with `clap`
  ```rust
  // src/cli.rs
  #[derive(Parser)]
  #[command(name = "gaji")]
  #[command(about = "Type-safe GitHub Actions workflows in TypeScript")]
  struct Cli {
      #[command(subcommand)]
      command: Commands,
  }
  
  #[derive(Subcommand)]
  enum Commands {
      Init { /* ... */ },
      Dev { /* ... */ },
      Build { /* ... */ },
      Add { action: String },
      Clean,
  }
  ```
- [ ] Implement command skeletons
- [ ] Write help messages
- [ ] Add version info display
- [ ] Configure colored output

### 1.2 TypeScript File Parsing (oxc-based)
- [ ] Initialize oxc parser with basic configuration
  ```rust
  // src/parser/mod.rs
  pub struct TypeScriptParser {
      allocator: Allocator,
  }
  ```

- [ ] Implement AST Visitor pattern
  ```rust
  // src/parser/ast.rs
  pub struct ActionRefVisitor {
      pub action_refs: HashSet<String>,
  }
  
  impl<'a> Visit<'a> for ActionRefVisitor {
      fn visit_call_expression(&mut self, expr: &CallExpression<'a>) {
          // Find getAction calls
      }
  }
  ```

- [ ] Detect `getAction` call patterns
  - [ ] Direct calls: `getAction("actions/checkout@v4")`
  - [ ] Variable assignments: `const checkout = getAction("...")`
  - [ ] Inside arrays: `[getAction("..."), getAction("...")]`
  - [ ] Inside objects: `{ checkout: getAction("...") }`
  - [ ] Function arguments: `addStep(getAction("..."))`
  - [ ] Method chaining: `getAction("...")({ with: {...} })`

- [ ] Extract string literal values

- [ ] Consider template literal support (optional)
  ```typescript
  // Should we support this?
  const version = "v4"
  getAction(`actions/checkout@${version}`)
  ```

- [ ] Handle parsing errors
  ```rust
  pub enum ParserError {
      ParseFailed(String),
      InvalidSyntax(String),
      UnsupportedFeature(String),
  }
  ```

- [ ] Write unit tests
  ```rust
  #[cfg(test)]
  mod tests {
      #[test]
      fn test_simple_call() { /* ... */ }
      
      #[test]
      fn test_nested_expressions() { /* ... */ }
      
      #[test]
      fn test_array_and_object() { /* ... */ }
      
      #[test]
      fn test_invalid_syntax() { /* ... */ }
  }
  ```

### 1.3 File System Integration
- [ ] Implement single file analysis function
  ```rust
  pub async fn analyze_file(path: &Path) -> Result<HashSet<String>> {
      let source = fs::read_to_string(path)?;
      let parser = TypeScriptParser::new();
      parser.extract_action_refs(&source)
  }
  ```

- [ ] Implement recursive directory scanning
  ```rust
  pub async fn analyze_directory(dir: &Path) -> Result<HashMap<PathBuf, HashSet<String>>> {
      // Find and analyze only .ts files
  }
  ```

- [ ] Respect `.gitignore` patterns (optional)

- [ ] Show progress indicator
  ```rust
  let pb = ProgressBar::new(files.len() as u64);
  pb.set_style(ProgressStyle::default_bar()
      .template("[{elapsed_precise}] {bar:40} {pos}/{len} {msg}")
  );
  ```

### 1.4 GitHub API Integration
- [ ] Implement HTTP client with `reqwest`
  ```rust
  // src/fetcher.rs
  pub struct GitHubFetcher {
      client: reqwest::Client,
      cache: Cache,
  }
  
  impl GitHubFetcher {
      pub async fn fetch_action_metadata(&self, action_ref: &str) -> Result<String> {
          // Parse "actions/checkout@v4"
          // Download https://raw.githubusercontent.com/.../action.yml
      }
  }
  ```

- [ ] Implement `action.yml` download function

- [ ] Parse action references
  - `actions/checkout@v4`
  - `owner/repo@tag`
  - `owner/repo/path@ref`

- [ ] Error handling
  - [ ] 404 Not Found
  - [ ] Network timeout
  - [ ] Rate limiting (429)
  - [ ] Invalid action reference format

- [ ] Implement retry logic (exponential backoff)

- [ ] Consider rate limiting
  - [ ] Support GitHub API token (optional)
  - [ ] Throttle requests

- [ ] Write unit tests (mock HTTP)

### 1.5 YAML Parsing
- [ ] Parse `action.yml` with `serde_yaml`

- [ ] Define schema structures
  ```rust
  #[derive(Debug, Deserialize)]
  pub struct ActionMetadata {
      pub name: String,
      pub description: Option<String>,
      pub inputs: Option<HashMap<String, ActionInput>>,
      pub outputs: Option<HashMap<String, ActionOutput>>,
      pub runs: ActionRuns,
  }
  
  #[derive(Debug, Deserialize)]
  pub struct ActionInput {
      pub description: Option<String>,
      pub required: Option<bool>,
      pub default: Option<String>,
      #[serde(rename = "deprecationMessage")]
      pub deprecation_message: Option<String>,
  }
  ```

- [ ] Extract inputs, outputs, runs

- [ ] Handle YAML parsing errors

- [ ] Validate
  - [ ] Check required fields
  - [ ] Type validation

---

## Phase 2: Type Generation

### 2.1 TypeScript Type Definition Generator
- [ ] Convert ActionMetadata → TypeScript interface
  ```rust
  // src/generator/types.rs
  pub fn generate_input_interface(
      action_name: &str,
      inputs: &HashMap<String, ActionInput>
  ) -> String {
      // Generate TypeScript interface string
  }
  ```

- [ ] Generate JSDoc comments
  ```typescript
  /**
   * Checkout code from repository
   * @see https://github.com/actions/checkout
   */
  export interface CheckoutInputs {
      /**
       * Repository name with owner
       * @default ${{ github.repository }}
       */
      repository?: string;
      
      /**
       * Number of commits to fetch (0 = all history)
       * @default 1
       */
      'fetch-depth'?: number;
  }
  ```

- [ ] Handle optional/required fields
  ```rust
  let optional_marker = if input.required.unwrap_or(false) { "" } else { "?" };
  ```

- [ ] Include default values in JSDoc

- [ ] Mark deprecated fields
  ```typescript
  /** @deprecated Use 'new-field' instead */
  ```

- [ ] Type inference
  - [ ] String types
  - [ ] Detect number types (based on default value)
  - [ ] Detect boolean types
  - [ ] Union types (optional)

- [ ] Type definition template
  ```rust
  // src/generator/templates.rs
  pub const TYPE_DEFINITION_TEMPLATE: &str = r#"
  // Auto-generated from {action_ref}
  // Do not edit manually
  
  {jsdoc}
  export interface {InterfaceName}Inputs {{
      {fields}
  }}
  
  export function getAction(
      ref: '{action_ref}'
  ): (config?: {{
      name?: string;
      with?: {InterfaceName}Inputs;
      id?: string;
  }}) => JobStep;
  "#;
  ```

### 2.2 File System Management
- [ ] Create `generated/` directory
  ```rust
  pub fn ensure_generated_dir() -> Result<PathBuf> {
      let dir = PathBuf::from("generated");
      fs::create_dir_all(&dir)?;
      Ok(dir)
  }
  ```

- [ ] Save type files
  - [ ] Generate filename: `actions-checkout-v4.d.ts`
  - [ ] Sanitize filename (handle special characters)
  ```rust
  pub fn action_ref_to_filename(action_ref: &str) -> String {
      action_ref
          .replace("/", "-")
          .replace("@", "-")
          .replace(".", "-")
          + ".d.ts"
  }
  ```

- [ ] Implement overwrite logic for existing files

- [ ] Generate index.d.ts (re-export all types)
  ```typescript
  // generated/index.d.ts
  export * from './actions-checkout-v4';
  export * from './actions-setup-node-v4';
  ```

### 2.3 Type Caching
- [ ] Design cache structure
  ```rust
  // src/cache.rs
  #[derive(Serialize, Deserialize)]
  pub struct CacheEntry {
      pub action_ref: String,
      pub content_hash: String,
      pub generated_at: DateTime<Utc>,
      pub metadata: ActionMetadata,
  }
  
  pub struct Cache {
      entries: HashMap<String, CacheEntry>,
      cache_file: PathBuf,
  }
  ```

- [ ] Save to JSON file (`.gaji-cache.json`)

- [ ] Store per-action metadata
  - [ ] Content hash (SHA256 of action.yml)
  - [ ] Generation timestamp
  - [ ] Version info

- [ ] Validate cache
  ```rust
  pub fn should_regenerate(&self, action_ref: &str, new_hash: &str) -> bool {
      match self.entries.get(action_ref) {
          Some(entry) => entry.content_hash != new_hash,
          None => true,
      }
  }
  ```

- [ ] Implement incremental updates

- [ ] Cache expiration policy (optional)
  - [ ] Auto re-validate after 7 days
  - [ ] Force regenerate with `--force` flag

---

## Phase 3: File Watching

### 3.1 File System Monitoring
- [ ] Implement file watching with `notify` crate
  ```rust
  // src/watcher.rs
  use notify::{Watcher, RecursiveMode, Event, EventKind};
  
  pub async fn watch_directory(dir: &Path) -> Result<()> {
      let (tx, rx) = channel();
      let mut watcher = RecommendedWatcher::new(tx, Config::default())?;
      
      watcher.watch(dir, RecursiveMode::Recursive)?;
      
      for res in rx {
          match res {
              Ok(event) => handle_event(event).await?,
              Err(e) => eprintln!("Watch error: {}", e),
          }
      }
      
      Ok(())
  }
  ```

- [ ] Detect `.ts` file changes
- [ ] Support `.tsx` files as well
- [ ] Implement recursive directory watching
- [ ] Add debounce logic (prevent rapid consecutive changes)
  ```rust
  use std::time::{Duration, Instant};
  
  let mut last_event: Option<Instant> = None;
  const DEBOUNCE_DURATION: Duration = Duration::from_millis(300);
  
  if let Some(last) = last_event {
      if last.elapsed() < DEBOUNCE_DURATION {
          continue; // Skip events that occur too quickly
      }
  }
  ```

- [ ] Exclude specific files/directories
  - [ ] `node_modules/`
  - [ ] `.git/`
  - [ ] `generated/`
  - [ ] `.gaji-cache.json`

### 3.2 Automatic Type Generation
- [ ] Auto-analyze on file change
  ```rust
  async fn handle_file_change(path: &Path) -> Result<()> {
      println!("📝 {} changed", path.display());
      
      let action_refs = analyze_file(path).await?;
      let new_refs = filter_new_refs(&action_refs).await?;
      
      if !new_refs.is_empty() {
          generate_types_for_refs(&new_refs).await?;
      }
      
      Ok(())
  }
  ```

- [ ] Generate types only for new action references
- [ ] Trigger automatic type generation
- [ ] Show progress
  ```rust
  println!("🔍 Found {} new actions", new_refs.len());
  for action_ref in &new_refs {
      println!("  ⏳ Generating types for {}...", action_ref);
      // ...
      println!("  ✅ {}", action_ref);
  }
  ```

- [ ] Display success/failure notifications

### 3.3 Error Handling
- [ ] Retry on network errors
  ```rust
  let mut retries = 0;
  const MAX_RETRIES: u32 = 3;
  
  loop {
      match fetch_action_metadata(action_ref).await {
          Ok(data) => break Ok(data),
          Err(e) if retries < MAX_RETRIES => {
              retries += 1;
              println!("⚠️  Retry {}/{}: {}", retries, MAX_RETRIES, e);
              tokio::time::sleep(Duration::from_secs(2_u64.pow(retries))).await;
          }
          Err(e) => break Err(e),
      }
  }
  ```

- [ ] Log parsing errors

- [ ] Provide user-friendly error messages
  ```rust
  eprintln!("❌ Failed to generate types for {}", action_ref);
  eprintln!("   Reason: {}", error);
  eprintln!("   Try: gaji clean && gaji generate");
  ```

- [ ] Allow partial failures (continue even if some actions fail)

---

## Phase 4: YAML Build System

### 4.1 Workflow Execution
- [ ] Check Node.js installation
  ```rust
  fn check_node_installed() -> Result<()> {
      Command::new("node")
          .arg("--version")
          .output()
          .context("Node.js is not installed")?;
      Ok(())
  }
  ```

- [ ] Verify TypeScript execution environment
  - [ ] Check `tsx` installation or auto-install
  - [ ] Or support `ts-node`

- [ ] Execute TypeScript via subprocess
  ```rust
  // src/builder.rs
  pub async fn execute_workflow(workflow_path: &Path) -> Result<String> {
      let output = Command::new("npx")
          .args(&["tsx", workflow_path.to_str().unwrap()])
          .output()
          .await?;
      
      if !output.status.success() {
          return Err(anyhow!("Failed to execute workflow: {}",
              String::from_utf8_lossy(&output.stderr)));
      }
      
      Ok(String::from_utf8(output.stdout)?)
  }
  ```

- [ ] Serialize workflow object to JSON
  ```typescript
  // User workflow should output like this
  console.log(JSON.stringify(workflow.toYAML()));
  ```

- [ ] Convert JSON → YAML
- [ ] Capture stdout/stderr
- [ ] Measure execution time

### 4.2 YAML Validation
- [ ] Validate generated YAML syntax
  ```rust
  pub fn validate_yaml(yaml: &str) -> Result<()> {
      serde_yaml::from_str::<serde_yaml::Value>(yaml)
          .context("Invalid YAML syntax")?;
      Ok(())
  }
  ```

- [ ] Check GitHub Actions schema compliance
  - [ ] Verify required fields (`name`, `on`, `jobs`)
  - [ ] Only use known fields

- [ ] Warnings and recommendations
  ```rust
  // Example: when job has no name
  if job.name.is_none() {
      println!("⚠️  Warning: Job '{}' has no name", job.id);
  }
  ```

- [ ] Lint rules (optional)

### 4.3 File Output
- [ ] Create `.github/workflows/` directory
  ```rust
  pub fn ensure_workflows_dir() -> Result<PathBuf> {
      let dir = PathBuf::from(".github/workflows");
      fs::create_dir_all(&dir)?;
      Ok(dir)
  }
  ```

- [ ] Save YAML file
  - [ ] Filename: workflow ID + `.yml`
  - [ ] Format with 2-space indentation

- [ ] Compare with existing file
  ```rust
  pub fn should_write_file(path: &Path, new_content: &str) -> Result<bool> {
      if !path.exists() {
          return Ok(true);
      }
      
      let old_content = fs::read_to_string(path)?;
      Ok(old_content != new_content)
  }
  ```

- [ ] Save only when changed (avoid unnecessary git diffs)

- [ ] Add comments
  ```yaml
  # Auto-generated by gaji
  # Do not edit manually - Edit workflows/ci.ts instead
  # Generated at: 2024-01-15T10:30:00Z
  
  name: CI
  # ...
  ```

---

## Phase 5: init Command (Smart Project Initialization)

### 5.0 Project State Detection
- [ ] Implement smart project detection
  ```rust
  // src/commands/init.rs
  
  pub struct InitOptions {
      pub force: bool,           // Overwrite existing files
      pub skip_examples: bool,   // Skip example generation
      pub migrate: bool,         // Migrate existing YAML workflows
      pub interactive: bool,     // Interactive mode
  }
  
  enum ProjectState {
      Empty,              // Empty directory
      ExistingNode,       // package.json exists
      HasWorkflows,       // .github/workflows/*.yml exists
  }
  
  fn detect_project_state() -> Result<ProjectState> {
      let has_package_json = Path::new("package.json").exists();
      let has_workflows = Path::new(".github/workflows")
          .read_dir()
          .ok()
          .and_then(|entries| {
              entries
                  .filter_map(|e| e.ok())
                  .any(|e| {
                      let ext = e.path().extension();
                      ext == Some(OsStr::new("yml")) || ext == Some(OsStr::new("yaml"))
                  })
                  .then_some(())
          })
          .is_some();
      
      if has_workflows {
          Ok(ProjectState::HasWorkflows)
      } else if has_package_json {
          Ok(ProjectState::ExistingNode)
      } else {
          Ok(ProjectState::Empty)
      }
  }
  ```

- [ ] Main init logic with state handling
  ```rust
  pub async fn init_project(options: InitOptions) -> Result<()> {
      println!("🚀 Initializing gaji project...\n");
      
      if options.interactive {
          return interactive_init().await;
      }
      
      let project_state = detect_project_state()?;
      
      match project_state {
          ProjectState::Empty => {
              init_new_project(options).await?;
          }
          ProjectState::ExistingNode => {
              init_in_existing_node_project(options).await?;
          }
          ProjectState::HasWorkflows => {
              init_with_migration(options).await?;
          }
      }
      
      Ok(())
  }
  ```

### 5.1 Empty Project Initialization
- [ ] Create complete new project structure
  ```rust
  async fn init_new_project(options: InitOptions) -> Result<()> {
      println!("📦 Creating new project structure...\n");
      
      // Create directories
      create_directories()?;
      println!("✓ Created project directories");
      
      // Create package.json
      create_package_json()?;
      println!("✓ Created package.json");
      
      // Create tsconfig.json
      create_tsconfig()?;
      println!("✓ Created tsconfig.json");
      
      // Create .gitignore
      create_gitignore()?;
      println!("✓ Created .gitignore");
      
      // Create example workflow
      if !options.skip_examples {
          create_example_workflow()?;
          println!("✓ Created example workflow");
      }
      
      println!("\n✨ Project initialized!\n");
      print_next_steps();
      
      Ok(())
  }
  ```

- [ ] Example workflow template
  ```typescript
  // workflows/ci.ts
  import { Workflow, Job } from 'gaji'
  import { getAction } from 'gaji/actions'
  
  const checkout = getAction('actions/checkout@v4')
  const setupNode = getAction('actions/setup-node@v4')
  
  export const ci = new Workflow('ci', {
    name: 'CI',
    on: {
      push: { branches: ['main'] },
      pull_request: { branches: ['main'] },
    },
  })
    .addJob(
      new Job('build', 'ubuntu-latest')
        .addStep(checkout({
          name: 'Checkout code',
        }))
        .addStep(setupNode({
          name: 'Setup Node.js',
          with: {
            'node-version': '20',
          },
        }))
        .addStep({
          name: 'Install dependencies',
          run: 'npm ci',
        })
        .addStep({
          name: 'Run tests',
          run: 'npm test',
        })
    )
  ```

- [ ] package.json template
  ```json
  {
    "name": "my-project",
    "version": "1.0.0",
    "scripts": {
      "gha:dev": "gaji dev",
      "gha:build": "gaji build",
      "gha:watch": "gaji watch"
    },
    "devDependencies": {
      "gaji": "^1.0.0",
      "tsx": "^4.0.0"
    }
  }
  ```

### 5.2 Existing Node.js Project Integration
- [ ] Safe integration with existing projects
  ```rust
  async fn init_in_existing_node_project(options: InitOptions) -> Result<()> {
      println!("📦 Adding gaji to existing project...\n");
      
      // Create only gaji directories
      create_directories()?;
      println!("✓ Created workflows/, generated/, .github/workflows/");
      
      // Update package.json (merge)
      if Path::new("package.json").exists() {
          update_package_json()?;
          println!("✓ Updated package.json");
      } else {
          create_package_json()?;
          println!("✓ Created package.json");
      }
      
      // Handle tsconfig.json
      handle_tsconfig(&options)?;
      
      // Update .gitignore
      update_gitignore()?;
      println!("✓ Updated .gitignore");
      
      // Optional example
      if !options.skip_examples {
          create_example_workflow()?;
          println!("✓ Created example workflow");
      }
      
      println!("\n✨ gaji added to your project!\n");
      print_next_steps();
      
      Ok(())
  }
  ```

- [ ] Smart package.json merging
  ```rust
  fn update_package_json() -> Result<()> {
      let content = fs::read_to_string("package.json")?;
      let mut package: serde_json::Value = serde_json::from_str(&content)?;
      
      // Merge scripts
      let scripts = package["scripts"].as_object_mut()
          .ok_or_else(|| anyhow!("No scripts in package.json"))?;
      
      scripts.entry("gha:dev")
          .or_insert(json!("gaji dev"));
      scripts.entry("gha:build")
          .or_insert(json!("gaji build"));
      scripts.entry("gha:watch")
          .or_insert(json!("gaji watch"));
      
      // Merge devDependencies
      let dev_deps = package["devDependencies"]
          .as_object_mut()
          .or_else(|| {
              package["devDependencies"] = json!({});
              package["devDependencies"].as_object_mut()
          })
          .ok_or_else(|| anyhow!("Failed to create devDependencies"))?;
      
      dev_deps.entry("gaji")
          .or_insert(json!("^1.0.0"));
      dev_deps.entry("tsx")
          .or_insert(json!("^4.0.0"));
      
      // Save with formatting
      let formatted = serde_json::to_string_pretty(&package)?;
      fs::write("package.json", formatted)?;
      
      Ok(())
  }
  ```

- [ ] Handle existing tsconfig.json
  ```rust
  fn handle_tsconfig(options: &InitOptions) -> Result<()> {
      if Path::new("tsconfig.json").exists() {
          if options.force {
              backup_and_create_tsconfig()?;
              println!("✓ Backed up and created tsconfig.json");
          } else {
              println!("⚠️  tsconfig.json already exists");
              println!("   Add this to your compilerOptions:");
              println!("   \"typeRoots\": [\"./node_modules/@types\", \"./generated\"]");
          }
      } else {
          create_tsconfig()?;
          println!("✓ Created tsconfig.json");
      }
      Ok(())
  }
  ```

- [ ] Smart .gitignore updates
  ```rust
  fn update_gitignore() -> Result<()> {
      let gitignore_path = Path::new(".gitignore");
      
      let gha_ts_entries = vec![
          "# gaji",
          "generated/",
          ".gaji-cache.json",
          "dist/",
      ];
      
      if gitignore_path.exists() {
          let mut content = fs::read_to_string(gitignore_path)?;
          
          // Check if already exists
          if !content.contains("# gaji") {
              content.push_str("\n\n");
              content.push_str(&gha_ts_entries.join("\n"));
              content.push_str("\n");
              fs::write(gitignore_path, content)?;
          }
      } else {
          let content = gha_ts_entries.join("\n") + "\n";
          fs::write(gitignore_path, content)?;
      }
      
      Ok(())
  }
  ```

### 5.3 Workflow Migration
- [ ] Detect existing YAML workflows
  ```rust
  async fn init_with_migration(options: InitOptions) -> Result<()> {
      println!("📦 Adding gaji to project with existing workflows...\n");
      
      // Discover existing workflows
      let existing_workflows = discover_workflows()?;
      println!("Found {} existing workflow(s):", existing_workflows.len());
      for workflow in &existing_workflows {
          println!("  - {}", workflow.display());
      }
      println!();
      
      if options.migrate {
          migrate_workflows(&existing_workflows).await?;
      } else {
          println!("💡 Tip: Run with --migrate to convert existing YAML workflows to TypeScript");
          println!("   gaji init --migrate\n");
      }
      
      // Continue with normal init
      init_in_existing_node_project(options).await?;
      
      Ok(())
  }
  
  fn discover_workflows() -> Result<Vec<PathBuf>> {
      let workflows_dir = Path::new(".github/workflows");
      if !workflows_dir.exists() {
          return Ok(vec![]);
      }
      
      let workflows: Vec<PathBuf> = fs::read_dir(workflows_dir)?
          .filter_map(|entry| entry.ok())
          .filter(|entry| {
              let ext = entry.path().extension();
              ext == Some(OsStr::new("yml")) || ext == Some(OsStr::new("yaml"))
          })
          .map(|entry| entry.path())
          .collect();
      
      Ok(workflows)
  }
  ```

- [ ] YAML to TypeScript migration
  ```rust
  async fn migrate_workflows(workflows: &[PathBuf]) -> Result<()> {
      println!("🔄 Migrating workflows to TypeScript...\n");
      
      for workflow_path in workflows {
          let workflow_name = workflow_path
              .file_stem()
              .and_then(|s| s.to_str())
              .ok_or_else(|| anyhow!("Invalid workflow path"))?;
          
          println!("  Migrating {}...", workflow_name);
          
          // Parse YAML
          let yaml_content = fs::read_to_string(workflow_path)?;
          let workflow: serde_yaml::Value = serde_yaml::from_str(&yaml_content)?;
          
          // Generate TypeScript code
          let ts_content = generate_typescript_from_yaml(&workflow, workflow_name)?;
          
          // Save to workflows/
          let ts_path = format!("workflows/{}.ts", workflow_name);
          fs::write(&ts_path, ts_content)?;
          println!("  ✓ Created {}", ts_path);
          
          // Backup existing YAML
          let backup_path = workflow_path.with_extension("yml.backup");
          fs::rename(workflow_path, &backup_path)?;
          println!("  ✓ Backed up to {}", backup_path.display());
      }
      
      println!("\n✨ Migration complete!");
      println!("   Review the generated TypeScript files in workflows/");
      println!("   Run 'gaji build' to regenerate YAML files\n");
      
      Ok(())
  }
  ```

- [ ] Basic YAML to TypeScript converter
  ```rust
  fn generate_typescript_from_yaml(
      workflow: &serde_yaml::Value,
      workflow_id: &str
  ) -> Result<String> {
      let mut ts = String::new();
      
      // Imports
      ts.push_str("import { Workflow, Job } from 'gaji'\n");
      ts.push_str("import { getAction } from 'gaji/actions'\n\n");
      
      // Extract workflow name
      let name = workflow["name"]
          .as_str()
          .unwrap_or(workflow_id);
      
      // Extract actions used
      let actions = extract_actions_from_yaml(workflow);
      for action in &actions {
          let var_name = action_to_var_name(action);
          ts.push_str(&format!(
              "const {} = getAction('{}')\n",
              var_name, action
          ));
      }
      ts.push_str("\n");
      
      // Workflow definition
      ts.push_str(&format!(
          "export const {} = new Workflow('{}', {{\n",
          workflow_id.replace("-", "_"),
          workflow_id
      ));
      ts.push_str(&format!("  name: '{}',\n", name));
      
      // Triggers
      if let Some(on) = workflow.get("on") {
          ts.push_str("  on: ");
          ts.push_str(&yaml_to_js_object(on, 2));
          ts.push_str(",\n");
      }
      
      ts.push_str("})\n");
      
      // Jobs (basic conversion)
      if let Some(jobs) = workflow["jobs"].as_mapping() {
          for (job_id, job_def) in jobs {
              ts.push_str(&format!("  .addJob(\n"));
              ts.push_str(&format!("    new Job('{}', '{}')\n",
                  job_id.as_str().unwrap_or("job"),
                  job_def["runs-on"].as_str().unwrap_or("ubuntu-latest")
              ));
              
              // Steps (simplified)
              if let Some(steps) = job_def["steps"].as_sequence() {
                  for step in steps {
                      ts.push_str("      .addStep({\n");
                      ts.push_str("        // TODO: Convert step\n");
                      if let Some(name) = step["name"].as_str() {
                          ts.push_str(&format!("        name: '{}',\n", name));
                      }
                      ts.push_str("      })\n");
                  }
              }
              
              ts.push_str("  )\n");
          }
      }
      
      ts.push_str("\n");
      ts.push_str("// NOTE: This is a basic conversion.\n");
      ts.push_str("// Please review and adjust as needed.\n");
      
      Ok(ts)
  }
  
  fn extract_actions_from_yaml(workflow: &serde_yaml::Value) -> Vec<String> {
      let mut actions = Vec::new();
      
      if let Some(jobs) = workflow["jobs"].as_mapping() {
          for (_, job) in jobs {
              if let Some(steps) = job["steps"].as_sequence() {
                  for step in steps {
                      if let Some(uses) = step["uses"].as_str() {
                          actions.push(uses.to_string());
                      }
                  }
              }
          }
      }
      
      actions.sort();
      actions.dedup();
      actions
  }
  
  fn action_to_var_name(action: &str) -> String {
      // "actions/checkout@v4" -> "checkout"
      action
          .split('/')
          .last()
          .unwrap_or("action")
          .split('@')
          .next()
          .unwrap_or("action")
          .replace("-", "_")
  }
  ```

### 5.4 Interactive Mode
- [ ] Implement interactive prompts
  ```rust
  use dialoguer::{Confirm, MultiSelect, theme::ColorfulTheme};
  
  async fn interactive_init() -> Result<()> {
      println!("🚀 gaji Interactive Setup\n");
      
      let project_state = detect_project_state()?;
      
      match project_state {
          ProjectState::ExistingNode => {
              println!("✓ Detected existing Node.js project\n");
          }
          ProjectState::HasWorkflows => {
              println!("✓ Detected existing GitHub Actions workflows\n");
              
              let workflows = discover_workflows()?;
              
              let should_migrate = Confirm::with_theme(&ColorfulTheme::default())
                  .with_prompt("Would you like to migrate existing workflows to TypeScript?")
                  .default(false)
                  .interact()?;
              
              if should_migrate {
                  let workflow_names: Vec<String> = workflows
                      .iter()
                      .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
                      .collect();
                  
                  let selections = MultiSelect::with_theme(&ColorfulTheme::default())
                      .with_prompt("Select workflows to migrate")
                      .items(&workflow_names)
                      .interact()?;
                  
                  let selected: Vec<PathBuf> = selections
                      .iter()
                      .map(|&i| workflows[i].clone())
                      .collect();
                  
                  migrate_workflows(&selected).await?;
              }
          }
          ProjectState::Empty => {
              println!("✓ Initializing new project\n");
          }
      }
      
      // Ask about example workflow
      let create_example = Confirm::with_theme(&ColorfulTheme::default())
          .with_prompt("Create example workflow?")
          .default(true)
          .interact()?;
      
      let options = InitOptions {
          force: false,
          skip_examples: !create_example,
          migrate: false, // Already handled above
          interactive: false,
      };
      
      init_project(options).await?;
      
      Ok(())
  }
  ```

### 5.5 CLI Interface
- [ ] Add init command with flags
  ```rust
  // src/cli.rs
  
  #[derive(Parser)]
  struct InitCommand {
      /// Overwrite existing files
      #[arg(long)]
      force: bool,
      
      /// Skip example workflow creation
      #[arg(long)]
      skip_examples: bool,
      
      /// Migrate existing YAML workflows to TypeScript
      #[arg(long)]
      migrate: bool,
      
      /// Interactive mode (ask questions)
      #[arg(short, long)]
      interactive: bool,
  }
  ```

### 5.6 Configuration File Generation
- [ ] Generate tsconfig.json
  ```json
  {
    "compilerOptions": {
      "target": "ES2020",
      "module": "commonjs",
      "lib": ["ES2020"],
      "outDir": "./dist",
      "rootDir": "./workflows",
      "strict": true,
      "esModuleInterop": true,
      "skipLibCheck": true,
      "forceConsistentCasingInFileNames": true,
      "typeRoots": ["./node_modules/@types", "./generated"]
    },
    "include": ["workflows/**/*"],
    "exclude": ["node_modules", "dist", "generated"]
  }
  ```

- [ ] Optional .gaji.toml config file
  ```toml
  [project]
  workflows_dir = "workflows"
  output_dir = ".github/workflows"
  generated_dir = "generated"
  
  [watch]
  debounce_ms = 300
  ignored_patterns = ["node_modules", ".git"]
  
  [build]
  validate = true
  format = true
  ```

### 5.7 Initial Type Generation
- [ ] Auto-generate types for example actions
  ```rust
  async fn generate_initial_types() -> Result<()> {
      println!("\n🔍 Analyzing workflow files...");
      
      let workflow_files: Vec<PathBuf> = fs::read_dir("workflows")?
          .filter_map(|e| e.ok())
          .filter(|e| e.path().extension() == Some(OsStr::new("ts")))
          .map(|e| e.path())
          .collect();
      
      if workflow_files.is_empty() {
          return Ok(());
      }
      
      let mut all_refs = HashSet::new();
      for file in &workflow_files {
          let refs = analyze_file(file).await?;
          all_refs.extend(refs);
      }
      
      if !all_refs.is_empty() {
          println!("📦 Generating types for {} actions...", all_refs.len());
          generate_types_for_refs(&all_refs).await?;
          println!("✨ Types generated!\n");
      }
      
      Ok(())
  }
  ```

### 5.8 Success Messages and Next Steps
- [ ] Display appropriate next steps based on project state
  ```rust
  fn print_next_steps() {
      println!("Next steps:");
      println!("  1. Run: npm install");
      println!("  2. Run: npm run gha:dev");
      println!("  3. Edit workflows/*.ts");
      println!("  4. Run: npm run gha:build");
      println!();
      println!("Learn more: https://github.com/your-org/gaji");
  }
  ```

### Usage Examples

```bash
# New project (empty directory)
gaji init

# Existing Node.js project (safe, non-destructive)
cd my-existing-project
gaji init

# Force overwrite existing files
gaji init --force

# Initialize without example
gaji init --skip-examples

# Migrate existing YAML workflows
gaji init --migrate

# Interactive mode with prompts
gaji init --interactive
gaji init -i

# Combine flags
gaji init --migrate --skip-examples
```

### Example Output Messages

**Existing Node.js Project:**
```
📦 Adding gaji to existing project...

✓ Created workflows/
✓ Created generated/
✓ Created .github/workflows/
✓ Updated package.json
⚠️  tsconfig.json already exists
   Add this to your compilerOptions:
   "typeRoots": ["./node_modules/@types", "./generated"]
✓ Updated .gitignore
✓ Created example workflow

🔍 Analyzing workflow files...
📦 Generating types for 2 actions...
✨ Types generated!

✨ gaji added to your project!

Next steps:
  1. Run: npm install
  2. Run: npm run gha:dev
  3. Edit workflows/ci.ts
  4. Run: npm run gha:build

Learn more: https://github.com/your-org/gaji
```

**With Existing Workflows:**
```
📦 Adding gaji to project with existing workflows...

Found 3 existing workflow(s):
  - .github/workflows/ci.yml
  - .github/workflows/release.yml
  - .github/workflows/test.yml

💡 Tip: Run with --migrate to convert existing YAML workflows to TypeScript
   gaji init --migrate

✓ Created workflows/
✓ Updated package.json
✓ Updated .gitignore

✨ gaji added to your project!
```

**With Migration:**
```
🔄 Migrating workflows to TypeScript...

  Migrating ci...
  ✓ Created workflows/ci.ts
  ✓ Backed up to .github/workflows/ci.yml.backup
  
  Migrating release...
  ✓ Created workflows/release.ts
  ✓ Backed up to .github/workflows/release.yml.backup

✨ Migration complete!
   Review the generated TypeScript files in workflows/
   Run 'gaji build' to regenerate YAML files
```

---

## Phase 6: NPM Package (NPM Wrapper)

### 6.1 npm Package Structure
- [ ] Create `gaji-npm/` directory
  ```
  gaji-npm/
  ├─ package.json
  ├─ install.js
  ├─ bin/
  │  └─ gaji.js
  ├─ lib/
  │  └─ index.js      # TypeScript runtime library
  └─ README.md
  ```

- [ ] Write `package.json`
  ```json
  {
    "name": "gaji",
    "version": "1.0.0",
    "description": "Type-safe GitHub Actions workflows in TypeScript",
    "bin": {
      "gaji": "./bin/gaji.js"
    },
    "main": "./lib/index.js",
    "types": "./lib/index.d.ts",
    "scripts": {
      "postinstall": "node install.js"
    },
    "files": [
      "bin/",
      "lib/",
      "install.js"
    ],
    "keywords": [
      "github-actions",
      "typescript",
      "ci-cd",
      "workflow",
      "type-safe"
    ],
    "repository": {
      "type": "git",
      "url": "https://github.com/your-org/gaji"
    },
    "os": ["darwin", "linux", "win32"],
    "cpu": ["x64", "arm64"]
  }
  ```

### 6.2 Binary Download Script
- [ ] Implement `install.js`
  - [ ] Detect platform (OS + arch)
  - [ ] Download from GitHub Releases
  - [ ] Show progress
  - [ ] Grant execute permission (Unix)
  - [ ] Error handling

- [ ] Fallback on installation failure
  ```javascript
  console.error('Failed to download binary.');
  console.error('Please install manually:');
  console.error('  cargo install gaji');
  ```

### 6.3 Execution Wrapper
- [ ] Implement `bin/gaji.js`
  ```javascript
  #!/usr/bin/env node
  
  const { spawn } = require('child_process');
  const { join } = require('path');
  
  const binPath = join(
    __dirname,
    process.platform === 'win32' ? 'gaji.exe' : 'gaji'
  );
  
  const child = spawn(binPath, process.argv.slice(2), {
    stdio: 'inherit',
    windowsHide: true,
  });
  
  child.on('exit', (code) => {
    process.exit(code);
  });
  ```

### 6.4 TypeScript Runtime Library
- [ ] `lib/index.js` - Workflow builder classes
  ```typescript
  // lib/index.ts
  export class Workflow {
    constructor(
      public id: string,
      public config: WorkflowConfig
    ) {}
    
    addJob(job: Job): this {
      this.jobs.set(job.id, job);
      return this;
    }
    
    toYAML(): object {
      return {
        name: this.config.name,
        on: this.config.on,
        jobs: Object.fromEntries(
          Array.from(this.jobs.entries())
            .map(([id, job]) => [id, job.toYAML()])
        ),
      };
    }
  }
  
  export class Job {
    steps: Step[] = [];
    
    constructor(
      public id: string,
      public runsOn: string = 'ubuntu-latest'
    ) {}
    
    addStep(step: Step): this {
      this.steps.push(step);
      return this;
    }
    
    toYAML(): object {
      return {
        'runs-on': this.runsOn,
        steps: this.steps.map(s => s.toYAML()),
      };
    }
  }
  
  export interface Step {
    name?: string;
    uses?: string;
    with?: Record<string, any>;
    run?: string;
    toYAML(): object;
  }
  ```

- [ ] Create type definition files
- [ ] Build and bundle

### 6.5 Platform-Specific Packages (Optional)
- [ ] `@gaji/darwin-x64`
- [ ] `@gaji/darwin-arm64`
- [ ] `@gaji/linux-x64`
- [ ] `@gaji/linux-arm64`
- [ ] `@gaji/win32-x64`
- [ ] Configure optionalDependencies

---

## Phase 7: CI/CD (Release Automation) - Using gaji itself!

### 7.1 Write Release Workflow with Our Own Product
- [ ] Create `workflows/release.ts`
  ```typescript
  // workflows/release.ts
  import { Workflow, Job } from 'gaji'
  import { getAction } from 'gaji/actions'
  
  const checkout = getAction('actions/checkout@v4')
  const setupRust = getAction('dtolnay/rust-toolchain@stable')
  const uploadArtifact = getAction('actions/upload-artifact@v4')
  const downloadArtifact = getAction('actions/download-artifact@v4')
  const createRelease = getAction('softprops/action-gh-release@v1')
  const setupNode = getAction('actions/setup-node@v4')
  
  // Build matrix definition
  const buildMatrix = {
    include: [
      { os: 'ubuntu-latest', target: 'x86_64-unknown-linux-gnu', name: 'linux-x64' },
      { os: 'ubuntu-latest', target: 'aarch64-unknown-linux-gnu', name: 'linux-arm64' },
      { os: 'macos-latest', target: 'x86_64-apple-darwin', name: 'darwin-x64' },
      { os: 'macos-latest', target: 'aarch64-apple-darwin', name: 'darwin-arm64' },
      { os: 'windows-latest', target: 'x86_64-pc-windows-msvc', name: 'win32-x64' },
    ]
  }
  
  export const release = new Workflow('release', {
    name: 'Release',
    on: {
      push: {
        tags: ['v*']
      }
    },
  })
    // Job 1: Cross-platform build
    .addJob(
      new Job('build', '${{ matrix.os }}')
        .setStrategy({
          matrix: buildMatrix
        })
        .addStep(checkout({
          name: 'Checkout code'
        }))
        .addStep(setupRust({
          name: 'Install Rust',
          with: {
            toolchain: 'stable',
            targets: '${{ matrix.target }}'
          }
        }))
        .addStep({
          name: 'Build binary',
          run: 'cargo build --release --target ${{ matrix.target }}'
        })
        .addStep({
          name: 'Strip binary (Unix)',
          if: "runner.os != 'Windows'",
          run: 'strip target/${{ matrix.target }}/release/gaji'
        })
        .addStep({
          name: 'Compress binary',
          run: `
            cd target/\${{ matrix.target }}/release
            tar czf gaji-\${{ matrix.name }}.tar.gz gaji*
          `
        })
        .addStep(uploadArtifact({
          name: 'Upload artifact',
          with: {
            name: 'gaji-${{ matrix.name }}',
            path: 'target/${{ matrix.target }}/release/gaji-${{ matrix.name }}.tar.gz'
          }
        }))
    )
    // Job 2: Create GitHub Release
    .addJob(
      new Job('release', 'ubuntu-latest')
        .setNeeds(['build'])
        .addStep(downloadArtifact({
          name: 'Download all artifacts',
          with: {
            path: 'artifacts'
          }
        }))
        .addStep({
          name: 'Generate checksums',
          run: `
            cd artifacts
            for dir in */; do
              cd "\$dir"
              sha256sum * > checksums.txt
              cd ..
            done
          `
        })
        .addStep(createRelease({
          name: 'Create Release',
          with: {
            files: 'artifacts/**/*',
            generate_release_notes: true
          },
          env: {
            GITHUB_TOKEN: '${{ secrets.GITHUB_TOKEN }}'
          }
        }))
    )
    // Job 3: Publish to npm
    .addJob(
      new Job('publish-npm', 'ubuntu-latest')
        .setNeeds(['release'])
        .addStep(checkout({
          name: 'Checkout code'
        }))
        .addStep(setupNode({
          name: 'Setup Node.js',
          with: {
            'node-version': '20',
            'registry-url': 'https://registry.npmjs.org'
          }
        }))
        .addStep({
          name: 'Update package version',
          run: `
            VERSION=\${GITHUB_REF#refs/tags/v}
            cd gaji-npm
            npm version \$VERSION --no-git-tag-version
          `
        })
        .addStep({
          name: 'Publish to npm',
          run: `
            cd gaji-npm
            npm publish
          `,
          env: {
            NODE_AUTH_TOKEN: '${{ secrets.NPM_TOKEN }}'
          }
        })
    )
    // Job 4: Publish to crates.io
    .addJob(
      new Job('publish-crates', 'ubuntu-latest')
        .setNeeds(['release'])
        .addStep(checkout({
          name: 'Checkout code'
        }))
        .addStep(setupRust({
          name: 'Install Rust',
          with: {
            toolchain: 'stable'
          }
        }))
        .addStep({
          name: 'Publish to crates.io',
          run: 'cargo publish --token ${{ secrets.CARGO_TOKEN }}'
        })
    )
  ```

### 7.2 Implement Additional Helper Methods
- [ ] `Job.setStrategy()` method
  ```typescript
  // lib/index.ts
  export class Job {
    strategy?: {
      matrix?: any;
      'fail-fast'?: boolean;
      'max-parallel'?: number;
    };
    
    setStrategy(strategy: Job['strategy']): this {
      this.strategy = strategy;
      return this;
    }
    
    toYAML(): object {
      return {
        'runs-on': this.runsOn,
        ...(this.strategy && { strategy: this.strategy }),
        steps: this.steps.map(s => s.toYAML()),
      };
    }
  }
  ```

- [ ] `Job.setNeeds()` method
  ```typescript
  export class Job {
    needs?: string[];
    
    setNeeds(jobs: string[]): this {
      this.needs = jobs;
      return this;
    }
    
    toYAML(): object {
      return {
        ...(this.needs && { needs: this.needs }),
        'runs-on': this.runsOn,
        steps: this.steps.map(s => s.toYAML()),
      };
    }
  }
  ```

- [ ] Support `env` in Step
  ```typescript
  export interface Step {
    name?: string;
    uses?: string;
    with?: Record<string, any>;
    run?: string;
    if?: string;
    env?: Record<string, string>;
    toYAML(): object;
  }
  ```

### 7.3 Write CI Workflow with Our Own Product
- [ ] Create `workflows/ci.ts` - Regular CI/CD
  ```typescript
  // workflows/ci.ts
  import { Workflow, Job } from 'gaji'
  import { getAction } from 'gaji/actions'
  
  const checkout = getAction('actions/checkout@v4')
  const setupRust = getAction('dtolnay/rust-toolchain@stable')
  const cache = getAction('actions/cache@v3')
  const codecov = getAction('codecov/codecov-action@v3')
  
  export const ci = new Workflow('ci', {
    name: 'CI',
    on: {
      push: {
        branches: ['main']
      },
      pull_request: {
        branches: ['main']
      }
    },
  })
    .addJob(
      new Job('test', 'ubuntu-latest')
        .addStep(checkout({
          name: 'Checkout code'
        }))
        .addStep(setupRust({
          name: 'Install Rust',
          with: {
            toolchain: 'stable',
            components: 'rustfmt, clippy'
          }
        }))
        .addStep(cache({
          name: 'Cache Cargo',
          with: {
            path: `
              ~/.cargo/bin/
              ~/.cargo/registry/index/
              ~/.cargo/registry/cache/
              ~/.cargo/git/db/
              target/
            `,
            key: "${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}"
          }
        }))
        .addStep({
          name: 'Check formatting',
          run: 'cargo fmt -- --check'
        })
        .addStep({
          name: 'Run Clippy',
          run: 'cargo clippy -- -D warnings'
        })
        .addStep({
          name: 'Run tests',
          run: 'cargo test --all-features'
        })
        .addStep({
          name: 'Test with coverage',
          run: 'cargo tarpaulin --out Xml'
        })
        .addStep(codecov({
          name: 'Upload coverage',
          with: {
            file: './cobertura.xml'
          }
        }))
    )
    .addJob(
      new Job('build', 'ubuntu-latest')
        .setNeeds(['test'])
        .addStep(checkout({
          name: 'Checkout code'
        }))
        .addStep(setupRust({
          name: 'Install Rust'
        }))
        .addStep({
          name: 'Build',
          run: 'cargo build --release'
        })
    )
  ```

### 7.4 Build Script for YAML Generation
- [ ] Create `scripts/build-workflows.sh`
  ```bash
  #!/bin/bash
  set -e
  
  echo "🔨 Building workflows..."
  
  # Convert TypeScript workflows to YAML
  gaji build
  
  echo "✅ Workflows built successfully!"
  echo ""
  echo "Generated files:"
  ls -lh .github/workflows/
  ```

- [ ] Or implement directly in Rust
  ```rust
  // examples/build_workflows.rs
  use gha_ts::builder::build_all_workflows;
  
  #[tokio::main]
  async fn main() -> Result<(), Box<dyn std::error::Error>> {
      println!("🔨 Building workflows...");
      
      build_all_workflows("workflows").await?;
      
      println!("✅ Workflows built successfully!");
      
      Ok(())
  }
  ```

### 7.5 Workflow to Auto-commit YAML
- [ ] Create `workflows/update-yaml.ts` - Auto-update YAML
  ```typescript
  // workflows/update-yaml.ts
  import { Workflow, Job } from 'gaji'
  import { getAction } from 'gaji/actions'
  
  const checkout = getAction('actions/checkout@v4')
  
  export const updateYaml = new Workflow('update-yaml', {
    name: 'Update YAML Workflows',
    on: {
      push: {
        branches: ['main'],
        paths: ['workflows/**/*.ts']
      }
    },
  })
    .addJob(
      new Job('update', 'ubuntu-latest')
        .addStep(checkout({
          name: 'Checkout code',
          with: {
            token: '${{ secrets.GITHUB_TOKEN }}'
          }
        }))
        .addStep({
          name: 'Setup gaji',
          run: 'cargo install gaji'
        })
        .addStep({
          name: 'Build workflows',
          run: 'gaji build'
        })
        .addStep({
          name: 'Check for changes',
          id: 'changes',
          run: `
            if git diff --quiet .github/workflows/; then
              echo "changed=false" >> $GITHUB_OUTPUT
            else
              echo "changed=true" >> $GITHUB_OUTPUT
            fi
          `
        })
        .addStep({
          name: 'Commit and push',
          if: "steps.changes.outputs.changed == 'true'",
          run: `
            git config user.name "github-actions[bot]"
            git config user.email "github-actions[bot]@users.noreply.github.com"
            git add .github/workflows/
            git commit -m "chore: update generated YAML workflows"
            git push
          `
        })
    )
  ```

### 7.6 Documentation Update
- [ ] Highlight dogfooding in README
  ```markdown
  ## Dogfooding
  
  This project uses itself to manage its own GitHub Actions workflows!
  
  Check out our workflow definitions:
  - [CI Workflow](workflows/ci.ts)
  - [Release Workflow](workflows/release.ts)
  - [Auto-update YAML](workflows/update-yaml.ts)
  
  The generated YAML files are in [.github/workflows/](.github/workflows/).
  ```

### 7.7 Usage During Development
- [ ] Use `workflows/` directory from early development
- [ ] Test with real workflows after each Phase completion
- [ ] Immediately improve issues discovered
- [ ] Document as real-world use cases

---

## Phase 8: Testing (코드 레벨 유닛/통합 테스트)

### 현재 상태 (52 tests)
| 모듈 | 테스트 수 | 상태 |
|------|----------|------|
| init/mod.rs | 17 | Good |
| init/migration.rs | 11 | Good |
| executor.rs | 7 | Good |
| parser/mod.rs | 5 | OK |
| fetcher.rs | 4 | Partial |
| generator/types.rs | 3 | Partial |
| generator/mod.rs | 2 | Minimal |
| cache.rs | 2 | Minimal |
| config.rs | 1 | Minimal |
| **builder.rs** | **0** | **Critical** |
| **watcher.rs** | **0** | **Critical** |

### 8.1 builder.rs 유닛 테스트 (~12개)
- [ ] `json_to_yaml()` - 정상 JSON→YAML 변환
- [ ] `json_to_yaml()` - 잘못된 JSON 에러 처리
- [ ] `validate_workflow_yaml()` - 정상 워크플로우 통과
- [ ] `validate_workflow_yaml()` - `on` 필드 누락 에러
- [ ] `validate_workflow_yaml()` - `jobs` 필드 누락 에러
- [ ] `validate_workflow_yaml()` - mapping이 아닌 YAML 에러
- [ ] `should_write_file()` - 새 파일 (존재하지 않음) → true
- [ ] `should_write_file()` - 동일 내용 → false
- [ ] `should_write_file()` - 변경된 내용 → true
- [ ] `timestamp_now()` - ISO 8601 형식 검증
- [ ] `find_workflow_files()` - .ts 파일 탐색, .d.ts 제외
- [ ] `build_all()` - 빈 디렉토리에서 빈 결과

### 8.2 watcher.rs 유닛 테스트 (~6개)
- [ ] `should_process_event()` - .ts Create 이벤트 → true
- [ ] `should_process_event()` - .tsx Modify 이벤트 → true
- [ ] `should_process_event()` - .rs 파일 → false
- [ ] `should_process_event()` - node_modules 내 .ts → false
- [ ] `should_process_event()` - generated/ 내 .ts → false
- [ ] `should_process_event()` - Delete 이벤트 → false

### 8.3 기존 모듈 테스트 보강
- [ ] cache.rs: `load_or_create()` tempdir 기본값 생성 (+1)
- [ ] cache.rs: `should_regenerate()` 해시 비교 (+1)
- [ ] cache.rs: `save()` → `load()` 직렬화 왕복 (+1)
- [ ] config.rs: TOML 문자열 파싱 (+1)
- [ ] config.rs: 부분 설정 시 기본값 폴백 (+1)
- [ ] fetcher.rs: 추가 에러 케이스 (+1)
- [ ] fetcher.rs: 경로 포함 action ref (+1)

### 8.4 통합 테스트 (`tests/integration.rs`)
- [ ] builder + executor 파이프라인: TS → QuickJS → JSON → YAML 검증
- [ ] build_all 다중 출력: 여러 workflow.build() 호출시 다중 YAML
- [ ] YAML 유효성: 생성된 YAML이 GitHub Actions 스키마 준수

**검증**: `cargo test` 전체 통과, 테스트 ~77개+ (현재 52 + 약 25 추가)

---

## Phase 9: Documentation (VitePress + i18n)

### 9.1 VitePress 프로젝트 설정
- [ ] `docs/` 디렉토리 생성
- [ ] `docs/package.json` 생성 (VitePress 의존성)
- [ ] `docs/.vitepress/config.ts` 생성 (i18n: 영어 + 한국어)
- [ ] `docs/public/logo.jpg` - 프로젝트 로고 배치 (사이트 로고/파비콘/Hero)

### 9.2 영어 문서 (docs/en/)
- [ ] `en/index.md` - 랜딩 페이지 (Hero + Features + 코드 비교)
- [ ] `en/guide/getting-started.md` - 퀵스타트 (init → add → build)
- [ ] `en/guide/installation.md` - 설치법 (npm, cargo, binary)
- [ ] `en/guide/writing-workflows.md` - 워크플로우 작성 (Workflow, Job, getAction, CompositeAction)
- [ ] `en/guide/configuration.md` - `.gaji.toml` 설정
- [ ] `en/guide/migration.md` - 기존 YAML 마이그레이션
- [ ] `en/reference/cli.md` - CLI 명령어 레퍼런스
- [ ] `en/reference/api.md` - TypeScript API (Workflow, Job, CompositeAction, CallAction)
- [ ] `en/reference/actions.md` - getAction() 및 타입 생성
- [ ] `en/examples/simple-ci.md` - 간단한 CI 예제
- [ ] `en/examples/matrix-build.md` - 매트릭스 빌드
- [ ] `en/examples/composite-action.md` - 컴포지트 액션

### 9.3 한국어 문서 (docs/ko/)
- [ ] `ko/index.md` - 랜딩 페이지
- [ ] `ko/guide/getting-started.md` - 빠른 시작
- [ ] `ko/guide/installation.md` - 설치
- [ ] `ko/guide/writing-workflows.md` - 워크플로우 작성
- [ ] `ko/guide/configuration.md` - 설정
- [ ] `ko/guide/migration.md` - 마이그레이션
- [ ] `ko/reference/cli.md` - CLI 레퍼런스
- [ ] `ko/reference/api.md` - API 레퍼런스
- [ ] `ko/reference/actions.md` - 액션 레퍼런스
- [ ] `ko/examples/simple-ci.md` - CI 예제
- [ ] `ko/examples/matrix-build.md` - 매트릭스 빌드
- [ ] `ko/examples/composite-action.md` - 컴포지트 액션

### 9.4 프로젝트 연동
- [ ] `.gitignore`에 VitePress 캐시/빌드/node_modules 추가
- [ ] `README.md`에 문서 사이트 링크 추가

**검증**: `cd docs && npm install && npm run docs:dev` → 로컬 서버에서 en/ko 전환 확인

---

## Phase 10: Optimization and Polish

### 10.1 Performance Optimization
- [ ] Parallel downloads
  ```rust
  use futures::stream::{self, StreamExt};
  
  let futures = action_refs.iter().map(|ref_| {
      fetch_and_generate_types(ref_)
  });
  
  stream::iter(futures)
      .buffer_unordered(5) // 5 concurrent
      .collect::<Vec<_>>()
      .await;
  ```

- [ ] Improve cache performance
  - [ ] Add memory cache
  - [ ] LRU cache

- [ ] Optimize binary size
  ```toml
  [profile.release]
  opt-level = "z"     # Optimize for size
  lto = true          # Link Time Optimization
  codegen-units = 1
  strip = true        # Strip symbols
  ```

- [ ] UPX compression (optional)
  ```bash
  upx --best --lzma target/release/gaji
  ```

### 10.2 User Experience
- [ ] Improve progress indicators
  ```rust
  use indicatif::{ProgressBar, ProgressStyle};
  
  let pb = ProgressBar::new(total as u64);
  pb.set_style(
      ProgressStyle::default_bar()
          .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}")
          .progress_chars("##-")
  );
  ```

- [ ] Colorful output
  ```rust
  use colored::*;
  
  println!("{}", "✓ Success!".green());
  println!("{}", "✗ Error!".red());
  println!("{}", "⚠ Warning!".yellow());
  ```

- [ ] Better error messages
  ```rust
  use thiserror::Error;
  
  #[derive(Error, Debug)]
  pub enum GhaError {
      #[error("Failed to parse TypeScript file: {0}")]
      ParseError(String),
      
      #[error("Action not found: {0}\nDid you mean: {1}?")]
      ActionNotFound(String, String),
      
      #[error("Network error: {0}\nPlease check your internet connection")]
      NetworkError(#[from] reqwest::Error),
  }
  ```

- [ ] Interactive prompts (optional)
  ```rust
  use dialoguer::{Confirm, Input, Select};
  
  let confirmed = Confirm::new()
      .with_prompt("Overwrite existing files?")
      .interact()?;
  ```

### 10.3 Additional Features
- [ ] Generate shell autocompletion scripts
  ```rust
  // clap supports this automatically
  gaji completions bash > /usr/local/etc/bash_completion.d/gaji
  gaji completions zsh > ~/.zfunc/_gaji
  gaji completions fish > ~/.config/fish/completions/gaji.fish
  ```

- [ ] Support configuration file
  ```toml
  # .gaji.toml
  [project]
  workflows_dir = "workflows"
  output_dir = ".github/workflows"
  
  [github]
  token = "ghp_..."  # optional
  
  [watch]
  debounce_ms = 300
  ```

- [ ] Plugin system (future)
  - [ ] Custom action type generators
  - [ ] YAML post-processing hooks

- [ ] VS Code extension (optional)

### 10.4 Quality Improvements
- [ ] Fix all Clippy warnings
  ```bash
  cargo clippy -- -D warnings
  ```

- [ ] Apply `cargo fmt`
- [ ] Update dependencies
- [ ] Security audit
  ```bash
  cargo audit
  ```

---

## Phase 11: Community and Launch

### 11.1 Beta Testing
- [ ] Recruit beta testers (GitHub Discussions)
- [ ] Collect feedback
  - [ ] GitHub Issues templates
  - [ ] Bug report form
  - [ ] Feature request form
- [ ] Fix bugs
- [ ] Beta release (v0.9.0)

### 11.2 Public Release
- [ ] Deploy to crates.io
  ```bash
  cargo publish
  ```

- [ ] Deploy to npm
  ```bash
  cd gaji-npm
  npm publish
  ```

- [ ] GitHub Release (v1.0.0)

- [ ] Homebrew formula (optional)
  ```ruby
  class GhaTs < Formula
    desc "Type-safe GitHub Actions workflows in TypeScript"
    homepage "https://github.com/your-org/gaji"
    url "https://github.com/your-org/gaji/archive/v1.0.0.tar.gz"
    sha256 "..."
    
    depends_on "rust" => :build
    
    def install
      system "cargo", "install", *std_cargo_args
    end
  end
  ```

- [ ] Official release announcement

### 11.3 Marketing
- [ ] Post to Hacker News
  - [ ] "Show HN: Type-safe GitHub Actions workflows in TypeScript"

- [ ] Post to Reddit
  - [ ] r/rust
  - [ ] r/github
  - [ ] r/programming

- [ ] Share on Twitter/X

- [ ] Dev.to blog post
  - [ ] "Building Type-Safe CI/CD with TypeScript"

- [ ] Submit to Product Hunt

- [ ] Monitor GitHub Trending

### 11.4 Community Building
- [ ] Enable GitHub Discussions
- [ ] Discord server (optional)
- [ ] Publish regular updates
- [ ] Contributor guide
- [ ] Code of Conduct

---

## Timeline Estimation

```
Phase 0:  1-2 days   (Project setup)
Phase 1:  5-7 days   (Core features - including oxc parsing)
Phase 2:  3-4 days   (Type generation)
Phase 3:  2-3 days   (File watching)
Phase 4:  2-3 days   (YAML build)
Phase 5:  1-2 days   (init command)
Phase 6:  2-3 days   (npm package)
Phase 7:  2-3 days   (CI/CD - dogfooding!)
Phase 8:  3-4 days   (Testing)
Phase 9:  2-3 days   (Documentation)
Phase 10: 2-3 days   (Optimization)
Phase 11: 1-2 days   (Launch)

Total estimate: 26-40 days (approximately 4-6 weeks)
```

---

## Milestones

```
M1: MVP (Phase 0-2) - 2 weeks
   └─ oxc parsing + type generation working

M2: Alpha (Phase 3-5) - 1 week
   └─ File watching + init + actually usable

M3: Beta (Phase 6-8) - 1-2 weeks
   └─ npm + CI/CD + testing complete

M4: v1.0 (Phase 9-11) - 1 week
   └─ Documentation + launch + marketing
```

---

## Benefits of Dogfooding (Phase 7)

✅ **Real-world examples**: Users can learn how to write complex workflows
✅ **Proof of reliability**: Demonstrates that it actually works
✅ **Bug discovery**: Naturally finds edge cases during development
✅ **Marketing**: "We use it ourselves!" - powerful message
✅ **Continuous improvement**: As the project evolves, workflows improve too

---

## Next Steps

1. **Choose starting point**: Begin with MVP (Phase 0-2)?
2. **Set up repository**: Create GitHub repo and initial structure
3. **Start coding**: Implement Phase 0 and Phase 1.1
4. **Iterate rapidly**: Build → Test → Improve

Let's build something awesome! 🚀
