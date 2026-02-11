use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "gha-ts")]
#[command(author = "gha-ts contributors")]
#[command(version)]
#[command(about = "Type-safe GitHub Actions workflows in TypeScript", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new gha-ts project
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

    /// Start development mode with file watching
    Dev {
        /// Directory to watch
        #[arg(short, long, default_value = "workflows")]
        dir: String,
    },

    /// Build TypeScript workflows to YAML
    Build {
        /// Input directory containing TypeScript workflows
        #[arg(short, long, default_value = "workflows")]
        input: String,

        /// Output directory for YAML files
        #[arg(short, long, default_value = ".github/workflows")]
        output: String,
    },

    /// Watch for file changes and regenerate types
    Watch {
        /// Directory to watch
        #[arg(short, long, default_value = "workflows")]
        dir: String,
    },

    /// Add a new action and generate types
    Add {
        /// Action reference (e.g., actions/checkout@v4)
        action: String,
    },

    /// Clean generated files
    Clean {
        /// Also clean cache
        #[arg(long)]
        cache: bool,
    },
}
