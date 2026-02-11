//! `ou list` -- Display all git worktrees with branch, commit hash, path, and status flags.
//!
//! In normal mode, outputs a colored, aligned table (branch=green, hash=yellow,
//! path=dim, flags=red). In `--quiet` mode, outputs only worktree paths for piping.
//!
//! Side effects: none (read-only).
//! Returns: `FormatResult::Table` (normal) or `FormatResult::Plain` (quiet).

use console::Style;

use crate::cli::ListArgs;
use crate::error::OuError;
use crate::git::executor::GitExecutor;
use crate::git::runner::GitRunner;
use crate::result::FormatResult;

/// Execute the `list` command.
///
/// Queries `git worktree list --porcelain`, parses the output into `Worktree` structs,
/// and formats them as either a colored table or plain path list depending on `--quiet`.
pub fn run<E: GitExecutor>(
    git: &GitRunner<E>,
    args: &ListArgs,
) -> Result<FormatResult, OuError> {
    let worktrees = git.worktree_list()?;

    if args.quiet {
        let paths: Vec<String> = worktrees
            .iter()
            .map(|wt| wt.path.to_string_lossy().to_string())
            .collect();
        return Ok(FormatResult::Plain(paths.join("\n")));
    }

    let branch_style = Style::new().green().bold();
    let hash_style = Style::new().yellow();
    let path_style = Style::new().dim();
    let flag_style = Style::new().red();

    let mut rows = Vec::new();
    for wt in &worktrees {
        let branch = wt.branch.as_deref().unwrap_or("(detached)");
        // Truncate commit hash to 7 characters (standard git short hash)
        let short_head = if wt.head.len() >= 7 {
            &wt.head[..7]
        } else {
            &wt.head
        };
        let mut flags = Vec::new();
        if wt.is_bare {
            flags.push("[bare]");
        }
        if wt.is_locked {
            flags.push("[locked]");
        }
        if wt.is_prunable {
            flags.push("[prunable]");
        }
        let flag_str = flags.join(" ");

        rows.push(vec![
            branch_style.apply_to(branch).to_string(),
            hash_style.apply_to(short_head).to_string(),
            path_style.apply_to(wt.path.to_string_lossy()).to_string(),
            flag_style.apply_to(&flag_str).to_string(),
        ]);
    }

    Ok(FormatResult::Table(rows))
}
