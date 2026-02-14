//! `ou open` -- Interactively select a worktree and open it in a terminal multiplexer tab.
//!
//! Presents a numbered list of non-bare worktrees, reads the user's selection from stdin,
//! and opens the chosen worktree in a new WezTerm tab (if detected).
//! Falls back to printing the selection if no multiplexer is available.
//!
//! Side effects: opens a new terminal tab via `wezterm cli spawn`.
//! Requires: interactive stdin (not suitable for piped input).
//! Related: `add --auto-open` opens a tab automatically at creation time.

use crate::config::Config;
use crate::error::OuError;
use crate::git::executor::GitExecutor;
use crate::git::runner::GitRunner;
use crate::multiplexer;

/// Execute the `open` command.
///
/// Flow: list worktrees -> filter bare -> present numbered selection via stderr
/// -> read choice from stdin -> detect multiplexer -> open tab with configured title.
/// Falls back to printing the selection if no multiplexer is available.
pub fn run<E: GitExecutor>(git: &GitRunner<E>, config: &Config) -> Result<String, OuError> {
    let worktrees = git.worktree_list()?;

    if worktrees.is_empty() {
        return Err(OuError::Git("no worktrees found".to_string()));
    }

    // Build display list
    let items: Vec<(String, String)> = worktrees
        .iter()
        .filter(|wt| !wt.is_bare)
        .map(|wt| {
            let branch = wt.branch.as_deref().unwrap_or("(detached)");
            let path = wt.path.to_string_lossy().to_string();
            (branch.to_string(), path)
        })
        .collect();

    if items.is_empty() {
        return Err(OuError::Git("no worktrees found".to_string()));
    }

    // Print selection UI to stderr so stdout remains clean for programmatic use
    eprintln!("Select a worktree:");
    for (i, (branch, path)) in items.iter().enumerate() {
        eprintln!("  {}: {} ({})", i + 1, branch, path);
    }
    eprint!("Enter number: ");

    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .map_err(|e| OuError::Git(format!("failed to read input: {e}")))?;

    let idx: usize = input
        .trim()
        .parse()
        .map_err(|_| OuError::Git("invalid selection".to_string()))?;

    if idx == 0 || idx > items.len() {
        return Err(OuError::Git("selection out of range".to_string()));
    }

    let (branch, path) = &items[idx - 1];
    let wt_path = std::path::PathBuf::from(path);

    // Try to open in multiplexer
    if let Some(mux) = multiplexer::detect_multiplexer() {
        let wezterm_config = config.wezterm.as_ref();
        let title = wezterm_config
            .and_then(|c| c.tab_title_template.as_ref())
            .map(|tmpl| tmpl.replace("{name}", branch))
            .unwrap_or_else(|| branch.clone());

        let pane_id = mux.open_tab(&wt_path, Some(&title))?;
        return Ok(format!(
            "Opened '{}' in {} (pane {})",
            branch,
            mux.name(),
            pane_id
        ));
    }

    Ok(format!("Selected: {} ({})", branch, path))
}
