mod cli;
mod commands;
mod config;
mod error;
mod fs;
mod git;
mod hooks;
mod multiplexer;
mod result;
mod symlink;
mod tui;

use anyhow::{Context, Result};
use clap::Parser;

use crate::cli::{Cli, Commands};
use crate::config::Config;
use crate::fs::OsFileSystem;
use crate::git::executor::OsGitExecutor;
use crate::git::runner::GitRunner;

const MIN_GIT_VERSION: (u32, u32, u32) = (2, 17, 0);

fn main() -> Result<()> {
    let cli = Cli::parse();
    let fs = OsFileSystem;
    let cwd = std::env::current_dir().context("failed to get current directory")?;
    let git = GitRunner::new(OsGitExecutor, cwd.clone());

    let (major, minor, patch) = git.git_version().context("failed to detect git version")?;
    if (major, minor, patch) < MIN_GIT_VERSION {
        return Err(crate::error::OuError::GitVersionTooOld {
            required: format!(
                "{}.{}.{}",
                MIN_GIT_VERSION.0, MIN_GIT_VERSION.1, MIN_GIT_VERSION.2
            ),
            found: format!("{major}.{minor}.{patch}"),
        }
        .into());
    }

    match cli.command {
        Commands::Init => {
            let msg = commands::init::run(&git, &fs)?;
            println!("{msg}");
        }
        Commands::Add(args) => {
            let repo_root = git.get_toplevel()?;
            let config = Config::load(&repo_root, &fs)?;
            let msg = commands::add::run(&git, &fs, &config, &args)?;
            println!("{msg}");
        }
        Commands::List(args) => {
            let result = commands::list::run(&git, &args)?;
            print!("{result}");
        }
        Commands::Remove(args) => {
            let msg = commands::remove::run(&git, &args)?;
            println!("{msg}");
        }
        Commands::Clean(args) => {
            let repo_root = git.get_toplevel()?;
            let config = Config::load(&repo_root, &fs)?;
            let msg = commands::clean::run(&git, &config, &args)?;
            println!("{msg}");
        }
        Commands::Sync(args) => {
            let repo_root = git.get_toplevel()?;
            let config = Config::load(&repo_root, &fs)?;
            let msg = commands::sync::run(&git, &fs, &config, &args)?;
            println!("{msg}");
        }
        Commands::Open => {
            let repo_root = git.get_toplevel()?;
            let config = Config::load(&repo_root, &fs)?;
            let msg = commands::open::run(&git, &config)?;
            println!("{msg}");
        }
        Commands::Dashboard => {
            tui::run_dashboard(&git)?;
        }
    }

    Ok(())
}
