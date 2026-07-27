mod commands;
mod compile;
mod config;
mod new;
mod pkg;

use clap::{Parser, Subcommand};
use miette::Result;

#[derive(Parser)]
#[command(name = "nasaq", version, about = "Nasaq — نَسَق programming language toolchain")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Type-check and parse Nasaq sources
    Check {
        #[arg(default_value = ".")]
        path: String,
    },
    /// Compile Nasaq sources to JavaScript ESM
    Build {
        #[arg(default_value = ".")]
        path: String,
        #[arg(long, default_value = "dist")]
        out: String,
    },
    /// Build and run the entry module with Node.js
    Run {
        #[arg(default_value = ".")]
        path: String,
    },
    /// Run exported `test_*` functions
    Test {
        #[arg(default_value = ".")]
        path: String,
    },
    /// Format Nasaq source files
    Fmt {
        #[arg(default_value = ".")]
        path: String,
    },
    /// Lint Nasaq source files
    Lint {
        #[arg(default_value = ".")]
        path: String,
    },
    /// Build and serve the project for browser development
    Dev {
        #[arg(default_value = ".")]
        path: String,
        #[arg(long, default_value_t = 3000)]
        port: u16,
    },
    /// Render exported component HTML (static SSR)
    Ssr {
        #[arg(default_value = ".")]
        path: String,
        #[arg(long, default_value = "dist")]
        out: String,
    },
    /// Generate npm package.json for dist/
    Publish {
        #[arg(default_value = ".")]
        path: String,
    },
    /// Write browser playground HTML
    Playground {
        #[arg(default_value = ".")]
        path: String,
        #[arg(long, default_value = "dist")]
        out: String,
    },
    /// Compile entry module to WebAssembly (MVP)
    Wasm {
        #[arg(default_value = ".")]
        path: String,
        #[arg(long, default_value = "dist")]
        out: String,
    },
    /// Create a new Nasaq project (like create-react-app)
    New {
        /// Project directory name
        name: String,
        /// Template: app | web | lib
        #[arg(long, default_value = "app")]
        template: String,
    },
    /// Initialize Nasaq in the current directory
    Init {
        #[arg(long, default_value = "app")]
        template: String,
    },
    /// Add a package from the Nasaq registry
    Add {
        name: String,
        #[arg(default_value = ".")]
        path: String,
    },
    /// Install dependencies from nasaq.toml [dependencies]
    Install {
        #[arg(default_value = ".")]
        path: String,
    },
    /// List packages in the Nasaq registry
    Search,
    /// Run compiler benchmarks
    Bench,
    /// Serve the official Nasaq website
    Website {
        #[arg(long, default_value_t = 8080)]
        port: u16,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Check { path } => commands::check(&path),
        Commands::Build { path, out } => commands::build(&path, &out),
        Commands::Run { path } => commands::run(&path),
        Commands::Test { path } => commands::test(&path),
        Commands::Fmt { path } => commands::fmt(&path),
        Commands::Lint { path } => commands::lint(&path),
        Commands::Dev { path, port } => commands::dev(&path, port),
        Commands::Ssr { path, out } => commands::ssr(&path, &out),
        Commands::Publish { path } => commands::publish(&path),
        Commands::Playground { path, out } => commands::playground(&path, &out),
        Commands::Wasm { path, out } => commands::wasm_build(&path, &out),
        Commands::New { name, template } => new::new_project(&name, &template),
        Commands::Init { template } => new::init_project(&template),
        Commands::Add { name, path } => pkg::add_package(&path, &name),
        Commands::Install { path } => pkg::install_deps(&path),
        Commands::Search => pkg::search_packages(),
        Commands::Bench => commands::bench(),
        Commands::Website { port } => commands::website(port),
    }
}
