//! `ou init` -- Initialize the `.ou/` configuration directory in a git repository.
//!
//! Creates `.ou/settings.toml` with sensible defaults (detects the default branch
//! name from the remote) and a `.gitignore` to exclude `settings.local.toml`.
//!
//! Side effects: creates `.ou/settings.toml` and `.ou/.gitignore` on disk.
//! Idempotency: returns an error if already initialized.

use std::path::Path;

use crate::config::{self, Config};
use crate::error::OuError;
use crate::fs::FileSystem;
use crate::git::executor::GitExecutor;
use crate::git::runner::GitRunner;

/// Execute the `init` command.
///
/// Locates the repository root, checks that `.ou/settings.toml` does not already
/// exist, detects the default branch name, then writes the config template and
/// `.gitignore`.
pub fn run<E: GitExecutor>(
    git: &GitRunner<E>,
    fs: &dyn FileSystem,
) -> Result<String, OuError> {
    let repo_root = git.get_toplevel()?;
    let settings_dir = repo_root.join(config::SETTINGS_DIR);
    let settings_path = settings_dir.join(config::SETTINGS_FILE);

    if fs.exists(&settings_path) {
        return Err(OuError::AlreadyInitialized(settings_path));
    }

    let repo_name = repo_root
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "repo".to_string());

    let default_branch = git.default_branch()?;

    let template = Config::default_toml()
        .replace("{repo_name}", &repo_name)
        .replace("default_source = \"main\"", &format!("default_source = \"{default_branch}\""));

    fs.mkdir_all(&settings_dir)?;
    fs.write(&settings_path, &template)?;

    create_gitignore(fs, &settings_dir)?;

    Ok(format!("Initialized ou in {}", settings_path.display()))
}

fn create_gitignore(fs: &dyn FileSystem, settings_dir: &Path) -> Result<(), OuError> {
    let gitignore_path = settings_dir.join(".gitignore");
    if !fs.exists(&gitignore_path) {
        fs.write(&gitignore_path, "settings.local.toml\n")?;
    }
    Ok(())
}
