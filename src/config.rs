use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::OuError;
use crate::fs::FileSystem;

pub const SETTINGS_DIR: &str = ".ou";
pub const SETTINGS_FILE: &str = "settings.toml";
pub const SETTINGS_LOCAL_FILE: &str = "settings.local.toml";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub worktree_destination_base_dir: Option<String>,

    #[serde(default)]
    pub default_source: Option<String>,

    #[serde(default)]
    pub symlinks: Vec<String>,

    #[serde(default)]
    pub extra_symlinks: Vec<String>,

    #[serde(default)]
    pub init_submodules: bool,

    #[serde(default)]
    pub submodule_reference: bool,

    #[serde(default)]
    pub wezterm: Option<WeztermConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WeztermConfig {
    #[serde(default)]
    pub auto_open: bool,

    #[serde(default)]
    pub tab_title_template: Option<String>,
}

impl Config {
    pub fn load(repo_root: &Path, fs: &dyn FileSystem) -> Result<Self, OuError> {
        let settings_dir = repo_root.join(SETTINGS_DIR);
        let settings_path = settings_dir.join(SETTINGS_FILE);
        let local_path = settings_dir.join(SETTINGS_LOCAL_FILE);

        let base = if fs.exists(&settings_path) {
            let content = fs
                .read_to_string(&settings_path)
                .map_err(|e| OuError::Config(format!("failed to read {}: {e}", settings_path.display())))?;
            toml::from_str::<Config>(&content)
                .map_err(|e| OuError::Config(format!("failed to parse {}: {e}", settings_path.display())))?
        } else {
            Config::default()
        };

        if fs.exists(&local_path) {
            let content = fs
                .read_to_string(&local_path)
                .map_err(|e| OuError::Config(format!("failed to read {}: {e}", local_path.display())))?;
            let local: Config = toml::from_str(&content)
                .map_err(|e| OuError::Config(format!("failed to parse {}: {e}", local_path.display())))?;
            Ok(base.merge(local))
        } else {
            Ok(base)
        }
    }

    fn merge(mut self, local: Config) -> Config {
        if local.worktree_destination_base_dir.is_some() {
            self.worktree_destination_base_dir = local.worktree_destination_base_dir;
        }
        if local.default_source.is_some() {
            self.default_source = local.default_source;
        }
        if !local.symlinks.is_empty() {
            self.symlinks = local.symlinks;
        }
        // extra_symlinks: merge both, deduplicate
        for s in local.extra_symlinks {
            if !self.extra_symlinks.contains(&s) {
                self.extra_symlinks.push(s);
            }
        }
        if local.init_submodules {
            self.init_submodules = true;
        }
        if local.submodule_reference {
            self.submodule_reference = true;
        }
        if local.wezterm.is_some() {
            self.wezterm = local.wezterm;
        }
        self
    }

    pub fn all_symlinks(&self) -> Vec<String> {
        let mut all = self.symlinks.clone();
        for s in &self.extra_symlinks {
            if !all.contains(s) {
                all.push(s.clone());
            }
        }
        all
    }

    pub fn worktree_base_dir(&self, repo_root: &Path) -> PathBuf {
        match &self.worktree_destination_base_dir {
            Some(dir) => {
                let p = PathBuf::from(dir);
                if p.is_absolute() {
                    p
                } else {
                    repo_root.join(p)
                }
            }
            None => {
                let repo_name = repo_root
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "repo".to_string());
                repo_root
                    .parent()
                    .unwrap_or(repo_root)
                    .join(format!("{repo_name}-worktrees"))
            }
        }
    }

    pub fn default_source_branch(&self) -> &str {
        self.default_source.as_deref().unwrap_or("main")
    }

    pub fn default_toml() -> String {
        r#"worktree_destination_base_dir = "../{repo_name}-worktrees"
default_source = "main"
symlinks = [".env", ".envrc", ".tool-versions"]
extra_symlinks = []
init_submodules = false
submodule_reference = false

[wezterm]
auto_open = false
tab_title_template = "{name}"
"#
        .to_string()
    }
}
