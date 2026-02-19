use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "gaji")]
#[command(author = "gaji contributors")]
#[command(version)]
#[command(about = "Type-safe GitHub Actions workflows in TypeScript", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new gaji project
    Init {
        /// Overwrite existing files
        #[arg(long)]
        force: bool,

        /// Skip example workflow creation
        #[arg(long)]
        skip_examples: bool,

        /// Migrate existing YAML workflows to TypeScript
        #[arg(long)]
        migrate: bool,

        /// Interactive mode
        #[arg(short, long)]
        interactive: bool,
    },

    /// Start development mode (one-time scan by default)
    Dev {
        /// Workflow directories or individual .ts files
        #[arg(short, long, num_args = 1..)]
        input: Vec<String>,

        /// Keep watching for changes after the initial scan
        #[arg(long)]
        watch: bool,
    },

    /// Build TypeScript workflows to YAML
    Build {
        /// Workflow directories or individual .ts files
        #[arg(short, long, num_args = 1..)]
        input: Vec<String>,

        /// Output directory for YAML files
        #[arg(short, long)]
        output: Option<String>,

        /// Preview YAML output without writing files
        #[arg(long)]
        dry_run: bool,
    },

    /// Add a new action and generate types
    Add {
        /// Action reference (e.g., actions/checkout@v5)
        action: String,
    },

    /// Clean generated files
    Clean {
        /// Also clean cache
        #[arg(long)]
        cache: bool,
    },

    /// Generate shell completions
    Completions {
        /// Shell type (bash, zsh, fish, powershell, elvish)
        shell: String,
    },
}
