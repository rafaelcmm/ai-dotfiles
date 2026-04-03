//! Binary entrypoint for `rafaelcmm-ai-dotfiles`.
//!
//! This executable validates target paths, optionally performs self-update,
//! then delegates dotfile synchronization to the library command runner.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use clap::{Parser, Subcommand};

use rafaelcmm_ai_dotfiles::{run, Command};

mod self_update;

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
    Update {
        #[arg(long, default_value_t = false)]
        no_self_update: bool,

        #[arg(long, default_value_t = false)]
        yes: bool,
    },
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
        Commands::Update {
            no_self_update,
            yes,
        } => {
            if !no_self_update
                && self_update::maybe_self_update_and_reexec(&home, cli.allow_outside_home, yes)?
            {
                return Ok(());
            }
            Command::Update
        }
        Commands::Debloat => Command::Debloat,
    };

    let message = run(command, &home)?;
    println!("{message}");
    Ok(())
}

/// Resolves the user's home directory from `HOME`.
///
/// # Errors
///
/// Returns an error when `HOME` is not set in the process environment.
fn default_home_dir() -> Result<PathBuf> {
    if let Some(home) = std::env::var_os("HOME") {
        return Ok(PathBuf::from(home));
    }

    anyhow::bail!("HOME environment variable is not set")
}

/// Returns whether `candidate` is inside `home` after normalization.
///
/// Existing paths are canonicalized to avoid false positives/negatives when
/// symbolic links are involved.
///
/// # Errors
///
/// Returns an error if the current directory cannot be read or if
/// canonicalization fails for existing paths.
fn path_is_within(candidate: &Path, home: &Path) -> Result<bool> {
    let normalized_candidate = normalize_to_absolute(candidate)?;
    let normalized_home = normalize_to_absolute(home)?;
    Ok(normalized_candidate.starts_with(&normalized_home))
}

/// Normalizes a path to an absolute path.
///
/// Existing paths are canonicalized. Non-existent paths are joined against the
/// current directory when relative.
///
/// # Errors
///
/// Returns an error if canonicalization fails for existing paths or if current
/// directory discovery fails for relative paths.
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
