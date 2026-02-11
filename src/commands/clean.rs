use crate::cli::CleanArgs;
use crate::config::Config;
use crate::error::OuError;
use crate::git::executor::GitExecutor;
use crate::git::runner::GitRunner;
use crate::git::types::MergeStatus;

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

        // Check if branch is merged
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
