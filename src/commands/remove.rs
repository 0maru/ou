//! `ou remove <branch>...` -- Remove one or more worktrees and their associated branches.
//!
//! Supports batch removal with partial-failure semantics: successfully removed worktrees
//! are reported, and errors for individual branches are collected separately.
//!
//! Force levels:
//! - No flag: refuses if uncommitted changes or locked
//! - `-f`: allows removal with uncommitted changes (passes --force to git)
//! - `-ff`: additionally unlocks locked worktrees before removal
//!
//! Side effects: removes worktree directories from disk, deletes git branches.
//! Related: `clean` automates candidate selection based on merge/gone status.

use crate::cli::RemoveArgs;
use crate::error::OuError;
use crate::git::executor::GitExecutor;
use crate::git::runner::GitRunner;

/// Execute the `remove` command.
///
/// Iterates over the requested branch names, resolves each to a worktree, validates
/// lock/bare status against the force level, removes the worktree, then attempts to
/// delete the branch. Returns a combined success/error report.
pub fn run<E: GitExecutor>(git: &GitRunner<E>, args: &RemoveArgs) -> Result<String, OuError> {
    if args.branches.is_empty() {
        return Err(OuError::Git("no branches specified".to_string()));
    }

    let worktrees = git.worktree_list()?;
    let mut removed = Vec::new();
    let mut errors = Vec::new();

    for branch_name in &args.branches {
        let wt = worktrees
            .iter()
            .find(|wt| wt.branch.as_deref() == Some(branch_name));

        let Some(wt) = wt else {
            errors.push(format!("worktree for branch '{branch_name}' not found"));
            continue;
        };

        if wt.is_bare {
            errors.push(format!("cannot remove bare worktree '{branch_name}'"));
            continue;
        }

        // Force level check: -f (force=1) is not enough to remove a locked worktree.
        // Only -ff (force>=2) will unlock and proceed.
        if wt.is_locked && args.force < 2 {
            let reason = wt.lock_reason.as_deref().unwrap_or("no reason given");
            errors.push(format!(
                "worktree '{branch_name}' is locked: {reason} (use -ff to force)"
            ));
            continue;
        }

        // Unlock if needed
        if wt.is_locked {
            git.worktree_unlock(&wt.path)?;
        }

        // Remove worktree
        let force = args.force >= 1;
        match git.worktree_remove(&wt.path, force) {
            Ok(()) => {}
            Err(e) => {
                errors.push(format!("failed to remove worktree '{branch_name}': {e}"));
                continue;
            }
        }

        // Branch deletion is best-effort: if the worktree was removed successfully
        // but branch deletion fails, warn but don't treat as a fatal error.
        match git.branch_delete(branch_name, force) {
            Ok(()) => {}
            Err(e) => {
                eprintln!("Warning: worktree removed but branch deletion failed: {e}");
            }
        }

        removed.push(branch_name.clone());
    }

    let mut msg = String::new();
    if !removed.is_empty() {
        msg.push_str(&format!("Removed: {}", removed.join(", ")));
    }
    if !errors.is_empty() {
        if !msg.is_empty() {
            msg.push('\n');
        }
        msg.push_str(&format!("Errors:\n  {}", errors.join("\n  ")));
    }

    if removed.is_empty() && !errors.is_empty() {
        Err(OuError::Git(msg))
    } else {
        Ok(msg)
    }
}
