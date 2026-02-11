use crate::cli::RemoveArgs;
use crate::error::OuError;
use crate::git::executor::GitExecutor;
use crate::git::runner::GitRunner;

pub fn run<E: GitExecutor>(
    git: &GitRunner<E>,
    args: &RemoveArgs,
) -> Result<String, OuError> {
    if args.branches.is_empty() {
        return Err(OuError::Git("no branches specified".to_string()));
    }

    let worktrees = git.worktree_list()?;
    let mut removed = Vec::new();
    let mut errors = Vec::new();

    for branch_name in &args.branches {
        let wt = worktrees.iter().find(|wt| {
            wt.branch.as_deref() == Some(branch_name)
        });

        let Some(wt) = wt else {
            errors.push(format!("worktree for branch '{branch_name}' not found"));
            continue;
        };

        if wt.is_bare {
            errors.push(format!("cannot remove bare worktree '{branch_name}'"));
            continue;
        }

        // Check lock status
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

        // Delete branch
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
