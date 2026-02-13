//! `ou clean` -- Automatically remove worktrees whose branches are merged or have a gone upstream.
//!
//! Scans all worktrees and identifies cleanup candidates based on two criteria:
//! - The branch is fully merged into the default source branch (e.g., `main`)
//! - The branch's upstream tracking ref is gone (deleted on remote)
//!
//! In `--check` mode, performs a dry run listing what would be removed.
//! In normal mode, removes each candidate's worktree and branch.
//!
//! Side effects: removes worktree directories and deletes git branches (unless --check).
//! Related: `remove` is the manual equivalent; `clean` automates candidate selection.

use crate::cli::CleanArgs;
use crate::config::Config;
use crate::error::OuError;
use crate::git::executor::GitExecutor;
use crate::git::runner::GitRunner;
use crate::git::types::MergeStatus;

/// Execute the `clean` command.
///
/// Flow: list worktrees and branches -> for each non-bare, non-default worktree,
/// check merge status and upstream gone status -> collect candidates -> either
/// report (--check) or remove each candidate's worktree and branch.
pub fn run<E: GitExecutor>(
    git: &GitRunner<E>,
    config: &Config,
    args: &CleanArgs,
) -> Result<String, OuError> {
    let worktrees = git.worktree_list()?;
    let branches = git.branch_list()?;
    let default_branch = config.default_source_branch();

    let mut candidates = Vec::new();

    for wt in &worktrees {
        if wt.is_bare {
            continue;
        }

        let Some(ref branch_name) = wt.branch else {
            continue;
        };

        if branch_name == default_branch {
            continue;
        }

        // Classify cleanup candidate based on two independent signals:
        // 1. Is the branch fully merged into the default branch?
        // 2. Has the upstream tracking branch been deleted on the remote?
        // A worktree is a candidate if either condition is true.
        let merged = git
            .is_branch_merged(branch_name, default_branch)
            .unwrap_or(MergeStatus::Unknown);

        // Check if upstream is gone
        let gone = branches
            .iter()
            .find(|b| b.name == *branch_name)
            .is_some_and(|b| b.gone);

        let reason = match (&merged, gone) {
            (MergeStatus::Merged, true) => "merged + upstream gone",
            (MergeStatus::Merged, false) => "merged",
            (_, true) => "upstream gone",
            _ => continue,
        };

        candidates.push((branch_name.clone(), wt.path.clone(), reason.to_string()));
    }

    if candidates.is_empty() {
        return Ok("No worktrees to clean.".to_string());
    }

    if args.check {
        let mut msg = String::from("Would remove:\n");
        for (branch, path, reason) in &candidates {
            msg.push_str(&format!("  {branch} ({reason}) at {}\n", path.display()));
        }
        return Ok(msg);
    }

    let mut removed = Vec::new();
    for (branch, path, reason) in &candidates {
        eprintln!("Removing {branch} ({reason})...");
        if let Err(e) = git.worktree_remove(path, false) {
            eprintln!("  Warning: failed to remove worktree: {e}");
            continue;
        }
        if let Err(e) = git.branch_delete(branch, false) {
            eprintln!("  Warning: failed to delete branch: {e}");
        }
        removed.push(branch.clone());
    }

    if removed.is_empty() {
        Ok("No worktrees were cleaned.".to_string())
    } else {
        Ok(format!("Cleaned: {}", removed.join(", ")))
    }
}
