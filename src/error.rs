use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
#[allow(dead_code)]
pub enum OuError {
    #[error("not a git repository (or any of the parent directories)")]
    NotGitRepository,

    #[error("already initialized: {0}")]
    AlreadyInitialized(PathBuf),

    #[error("worktree '{0}' already exists")]
    WorktreeAlreadyExists(String),

    #[error("worktree '{0}' not found")]
    WorktreeNotFound(String),

    #[error("branch '{0}' not found")]
    BranchNotFound(String),

    #[error("worktree '{0}' has uncommitted changes (use -f to force)")]
    UncommittedChanges(String),

    #[error("worktree '{0}' is locked: {1} (use -ff to force)")]
    WorktreeLocked(String, String),

    #[error("config error: {0}")]
    Config(String),

    #[error("git version {found} is too old (requires {required}+)")]
    GitVersionTooOld { required: String, found: String },

    #[error("git error: {0}")]
    Git(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("multiplexer error: {0}")]
    Multiplexer(String),

    #[error("symlink error: {0}")]
    Symlink(String),
}
