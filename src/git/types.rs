use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Worktree {
    pub path: PathBuf,
    pub branch: Option<String>,
    pub head: String,
    pub is_bare: bool,
    pub is_locked: bool,
    pub lock_reason: Option<String>,
    pub is_prunable: bool,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Branch {
    pub name: String,
    pub upstream: Option<String>,
    pub is_head: bool,
    pub gone: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MergeStatus {
    Merged,
    NotMerged,
    Unknown,
}

#[derive(Debug)]
pub struct CommandOutput {
    pub stdout: String,
    pub stderr: String,
    pub status: i32,
}

impl CommandOutput {
    pub fn success(&self) -> bool {
        self.status == 0
    }
}
