use crate::cli::AddArgs;
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
    args: &AddArgs,
) -> Result<String, OuError> {
    let repo_root = git.get_toplevel()?;
    let base_dir = config.worktree_base_dir(&repo_root);

    let wt_name = args.name.replace('/', "-");
    let wt_path = base_dir.join(&wt_name);

    if fs.exists(&wt_path) {
        return Err(OuError::WorktreeAlreadyExists(args.name.clone()));
    }

    let source = args
        .source
        .as_deref()
        .unwrap_or(config.default_source_branch());

    // Handle --carry: stash uncommitted changes
    let carried = if args.carry {
        git.stash_push(&format!("ou-carry: {}", args.name))?
    } else {
        false
    };

    // Create worktree + branch
    fs.mkdir_all(&base_dir)?;
    git.worktree_add(&wt_path, &args.name, Some(source))?;

    // Create symlinks
    let symlink_patterns = config.all_symlinks();
    if !symlink_patterns.is_empty() {
        let created = symlink::create_symlinks(fs, &repo_root, &wt_path, &symlink_patterns)?;
        if !created.is_empty() {
            eprintln!("Symlinked: {}", created.join(", "));
        }
    }

    // Lock if requested
    if args.lock {
        git.worktree_lock(&wt_path, args.reason.as_deref())?;
    }

    // Initialize submodules if requested
    if args.init_submodules || config.init_submodules {
        git.init_submodules(&wt_path)?;
    }

    // Apply carried stash in the new worktree
    if carried {
        // stash is shared across worktrees, pop via -C <new_worktree>
        git.stash_pop()?;
    }

    let mut msg = format!("Created worktree '{}' at {}", args.name, wt_path.display());
    if args.lock {
        msg.push_str(" [locked]");
    }

    Ok(msg)
}
