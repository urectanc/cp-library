use std::error::Error;
use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    List {
        #[arg(long, default_value = DEFAULT_MANIFEST_DIR)]
        manifest_dir: PathBuf,
    },
    Bundle {
        #[arg(long, default_value = DEFAULT_MANIFEST_DIR)]
        manifest_dir: PathBuf,
        #[arg(long, default_value = DEFAULT_NAMESPACE)]
        namespace: String,
    },
    Expand {
        #[arg(long, default_value = DEFAULT_MANIFEST_DIR)]
        manifest_dir: PathBuf,
        #[arg(long, default_value = DEFAULT_NAMESPACE)]
        namespace: String,
        #[arg(required = true)]
        crates: Vec<String>,
    },
}

const DEFAULT_MANIFEST_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/..");
const DEFAULT_NAMESPACE: &str = "urectanc";

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::List { manifest_dir } => list(manifest_dir),
        Commands::Bundle {
            manifest_dir,
            namespace,
        } => bundle(manifest_dir, namespace),
        Commands::Expand {
            manifest_dir,
            namespace,
            crates,
        } => expand(manifest_dir, namespace, crates),
    }
}

fn list(manifest_dir: PathBuf) -> Result<(), Box<dyn Error>> {
    let workspace = bundle::Workspace::load(manifest_dir)?;
    println!("{}", workspace.list().join("\n"));
    Ok(())
}

fn bundle(manifest_dir: PathBuf, namespace: String) -> Result<(), Box<dyn Error>> {
    let workspace = bundle::Workspace::load(manifest_dir)?;
    let source = std::io::read_to_string(std::io::stdin().lock())?;
    let bundled = workspace.bundle(&source, &namespace)?;
    if bundled.is_empty() {
        print!("{source}");
    } else {
        print!("{source}\n\n{bundled}");
    }
    Ok(())
}

fn expand(
    manifest_dir: PathBuf,
    namespace: String,
    crates: Vec<String>,
) -> Result<(), Box<dyn Error>> {
    let workspace = bundle::Workspace::load(manifest_dir)?;
    let expanded = workspace.expand(&crates, &namespace)?;
    print!("{expanded}");
    Ok(())
}
