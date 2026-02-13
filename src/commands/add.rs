//! `ou add <name>` -- Create a new git worktree with branch, symlinks, and optional integrations.
//!
//! Orchestrates: branch creation, `git worktree add`, symlink creation from repo root,
//! and optionally: lock the worktree, init submodules, carry uncommitted changes via
//! stash, and auto-open in WezTerm.
//!
//! Side effects: creates a worktree directory, a git branch, symlinks on disk, and
//! optionally modifies stash state and opens a terminal tab.
//! Related: `sync` re-applies symlinks/submodules; `remove` is the inverse operation.

use crate::cli::AddArgs;
use crate::config::Config;
use crate::error::OuError;
use crate::fs::FileSystem;
use crate::git::executor::GitExecutor;
use crate::git::runner::GitRunner;
use crate::multiplexer;
use crate::symlink;

/// Execute the `add` command.
///
/// Flow: sanitize name -> check existence -> optionally stash (--carry) -> create worktree
/// -> create symlinks -> optionally lock -> optionally init submodules -> pop stash
/// -> optionally auto-open in WezTerm.
pub fn run<E: GitExecutor>(
    git: &GitRunner<E>,
    fs: &dyn FileSystem,
    config: &Config,
    args: &AddArgs,
) -> Result<String, OuError> {
    let repo_root = git.get_toplevel()?;
    let base_dir = config.worktree_base_dir(&repo_root);

    // Sanitize branch name for use as directory name: "feat/login" -> "feat-login"
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
        git.stash_pop()?;
    }

    let mut msg = format!("Created worktree '{}' at {}", args.name, wt_path.display());
    if args.lock {
        msg.push_str(" [locked]");
    }

    // Auto-open in WezTerm if configured: spawns a new tab at the worktree path
    // with a title derived from the config template.
    let auto_open = config.wezterm.as_ref().is_some_and(|c| c.auto_open);

    if auto_open && let Some(mux) = multiplexer::detect_multiplexer() {
        let title = config
            .wezterm
            .as_ref()
            .and_then(|c| c.tab_title_template.as_ref())
            .map(|tmpl| tmpl.replace("{name}", &args.name))
            .unwrap_or_else(|| args.name.clone());

        match mux.open_tab(&wt_path, Some(&title)) {
            Ok(pane_id) => {
                msg.push_str(&format!(" (opened in {} pane {})", mux.name(), pane_id));
            }
            Err(e) => {
                eprintln!("Warning: failed to open tab: {e}");
            }
        }
    }

    Ok(msg)
}
