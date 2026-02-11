use crate::cli::SyncArgs;
use crate::config::Config;
use crate::error::OuError;
use crate::fs::FileSystem;
use crate::git::executor::GitExecutor;
use crate::git::runner::GitRunner;
use crate::symlink;

pub fn run<E: GitExecutor>(
    git: &GitRunner<E>,
    fs: &dyn FileSystem,
    config: &Config,
    args: &SyncArgs,
) -> Result<String, OuError> {
    let repo_root = git.get_toplevel()?;
    let worktrees = git.worktree_list()?;
    let symlink_patterns = config.all_symlinks();

    let source_dir = if let Some(ref source) = args.source {
        let wt = worktrees
            .iter()
            .find(|wt| wt.branch.as_deref() == Some(source))
            .ok_or_else(|| OuError::WorktreeNotFound(source.clone()))?;
        wt.path.clone()
    } else {
        repo_root.clone()
    };

    let targets: Vec<_> = if args.all {
        worktrees
            .iter()
            .filter(|wt| !wt.is_bare && wt.path != source_dir)
            .collect()
    } else {
        // Sync only current worktree if not --all
        // In practice, the current directory's worktree
        let cwd = std::env::current_dir()?;
        worktrees
            .iter()
            .filter(|wt| !wt.is_bare && wt.path == cwd)
            .collect()
    };

    if targets.is_empty() {
        return Ok("No worktrees to sync.".to_string());
    }

    let mut synced = Vec::new();
    for wt in targets {
        let created = symlink::create_symlinks(fs, &source_dir, &wt.path, &symlink_patterns)?;
        let branch = wt.branch.as_deref().unwrap_or("(detached)");
        if !created.is_empty() {
            eprintln!("Synced {branch}: {}", created.join(", "));
        }

        if config.init_submodules {
            if let Err(e) = git.init_submodules(&wt.path) {
                eprintln!("Warning: submodule init failed for {branch}: {e}");
            }
        }

        synced.push(branch.to_string());
    }

    Ok(format!("Synced: {}", synced.join(", ")))
}
