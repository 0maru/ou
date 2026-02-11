use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "ou", about = "Git worktree management CLI", version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize .ou/settings.toml
    Init,

    /// Create a worktree + branch + symlinks
    Add(AddArgs),

    /// List worktrees
    List(ListArgs),

    /// Remove worktrees and branches
    Remove(RemoveArgs),

    /// Clean merged/gone worktrees
    Clean(CleanArgs),

    /// Sync symlinks and submodules
    Sync(SyncArgs),

    /// Fuzzy select and open in terminal
    Open,

    /// TUI dashboard
    Dashboard,
}

#[derive(clap::Args)]
pub struct AddArgs {
    /// Branch name for the new worktree
    pub name: String,

    /// Base branch (default: config value or "main")
    #[arg(long)]
    pub source: Option<String>,

    /// Move uncommitted changes via stash
    #[arg(long)]
    pub carry: bool,

    /// Copy uncommitted changes to both worktrees
    #[arg(long)]
    pub sync: bool,

    /// Limit carry/sync to specific files
    #[arg(long)]
    pub file: Vec<String>,

    /// Lock the new worktree
    #[arg(long)]
    pub lock: bool,

    /// Reason for locking
    #[arg(long)]
    pub reason: Option<String>,

    /// Initialize submodules
    #[arg(long)]
    pub init_submodules: bool,

    /// Use reference for submodules
    #[arg(long)]
    pub submodule_reference: bool,
}

#[derive(clap::Args)]
pub struct ListArgs {
    /// Output only paths (for piping to fzf etc.)
    #[arg(short, long)]
    pub quiet: bool,
}

#[derive(clap::Args)]
pub struct RemoveArgs {
    /// Branch names to remove
    pub branches: Vec<String>,

    /// Force removal even with uncommitted changes
    #[arg(short = 'f', long, action = clap::ArgAction::Count)]
    pub force: u8,
}

#[derive(clap::Args)]
pub struct CleanArgs {
    /// Dry run: show what would be deleted
    #[arg(long)]
    pub check: bool,
}

#[derive(clap::Args)]
pub struct SyncArgs {
    /// Sync to all worktrees
    #[arg(long)]
    pub all: bool,

    /// Source worktree for sync
    #[arg(long)]
    pub source: Option<String>,
}
