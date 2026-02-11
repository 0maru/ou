use crate::cli::ListArgs;
use crate::error::OuError;
use crate::git::executor::GitExecutor;
use crate::git::runner::GitRunner;
use crate::result::FormatResult;

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

    let mut rows = Vec::new();
    for wt in &worktrees {
        let branch = wt.branch.as_deref().unwrap_or("(detached)");
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
            branch.to_string(),
            short_head.to_string(),
            wt.path.to_string_lossy().to_string(),
            flag_str,
        ]);
    }

    Ok(FormatResult::Table(rows))
}
