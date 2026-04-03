use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use clap::{Parser, Subcommand};

use rafaelcmm_ai_dotfiles::{run, Command};

#[derive(Debug, Parser)]
#[command(name = "rafaelcmm-ai-dotfiles")]
#[command(version)]
#[command(about = "Install, update and debloat AI tool configuration files")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(long, hide = true)]
    home: Option<PathBuf>,

    #[arg(long, hide = true, default_value_t = false)]
    allow_outside_home: bool,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Install,
    Update,
    Debloat,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let default_home = default_home_dir()?;
    let home = cli.home.unwrap_or_else(|| default_home.clone());

    if !cli.allow_outside_home && !path_is_within(&home, &default_home)? {
        anyhow::bail!(
            "refusing to operate outside HOME: {} is not inside {} (use --allow-outside-home to override)",
            home.display(),
            default_home.display()
        );
    }

    let command = match cli.command {
        Commands::Install => Command::Install,
        Commands::Update => Command::Update,
        Commands::Debloat => Command::Debloat,
    };

    let message = run(command, &home)?;
    println!("{message}");
    Ok(())
}

fn default_home_dir() -> Result<PathBuf> {
    if let Some(home) = std::env::var_os("HOME") {
        return Ok(PathBuf::from(home));
    }

    anyhow::bail!("HOME environment variable is not set")
}

fn path_is_within(candidate: &Path, home: &Path) -> Result<bool> {
    let normalized_candidate = normalize_to_absolute(candidate)?;
    let normalized_home = normalize_to_absolute(home)?;
    Ok(normalized_candidate.starts_with(&normalized_home))
}

fn normalize_to_absolute(path: &Path) -> Result<PathBuf> {
    if path.exists() {
        return Ok(fs::canonicalize(path)?);
    }

    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };

    Ok(absolute)
}
