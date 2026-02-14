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
            let content = fs.read_to_string(&settings_path).map_err(|e| {
                OuError::Config(format!("failed to read {}: {e}", settings_path.display()))
            })?;
            toml::from_str::<Config>(&content).map_err(|e| {
                OuError::Config(format!("failed to parse {}: {e}", settings_path.display()))
            })?
        } else {
            Config::default()
        };

        if fs.exists(&local_path) {
            let content = fs.read_to_string(&local_path).map_err(|e| {
                OuError::Config(format!("failed to read {}: {e}", local_path.display()))
            })?;
            let local: Config = toml::from_str(&content).map_err(|e| {
                OuError::Config(format!("failed to parse {}: {e}", local_path.display()))
            })?;
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

    pub fn worktree_base_dir(&self, repo_root: &Path, git_common_dir: &Path) -> PathBuf {
        match &self.worktree_destination_base_dir {
            Some(dir) => {
                let p = PathBuf::from(dir);
                if p.is_absolute() {
                    p
                } else {
                    repo_root.join(p)
                }
            }
            None => git_common_dir.join("ou-worktrees"),
        }
    }

    pub fn default_source_branch(&self) -> &str {
        self.default_source.as_deref().unwrap_or("main")
    }

    pub fn default_toml() -> String {
        r#"default_source = "main"
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fs::mock::MockFileSystem;
    use std::path::PathBuf;

    fn base_config() -> Config {
        Config {
            worktree_destination_base_dir: Some("base-dir".to_string()),
            default_source: Some("develop".to_string()),
            symlinks: vec![".env".to_string()],
            extra_symlinks: vec!["extra1".to_string()],
            init_submodules: false,
            submodule_reference: false,
            wezterm: Some(WeztermConfig {
                auto_open: true,
                tab_title_template: Some("base-tmpl".to_string()),
            }),
        }
    }

    #[test]
    fn test_merge_none_does_not_override() {
        let base = base_config();
        let local = Config::default();
        let merged = base.merge(local);
        assert_eq!(
            merged.worktree_destination_base_dir,
            Some("base-dir".to_string())
        );
        assert_eq!(merged.default_source, Some("develop".to_string()));
    }

    #[test]
    fn test_merge_some_overrides() {
        let base = base_config();
        let local = Config {
            worktree_destination_base_dir: Some("local-dir".to_string()),
            default_source: Some("main".to_string()),
            ..Config::default()
        };
        let merged = base.merge(local);
        assert_eq!(
            merged.worktree_destination_base_dir,
            Some("local-dir".to_string())
        );
        assert_eq!(merged.default_source, Some("main".to_string()));
    }

    #[test]
    fn test_merge_symlinks_replacement() {
        let base = base_config();
        let local = Config {
            symlinks: vec![".envrc".to_string(), ".tool-versions".to_string()],
            ..Config::default()
        };
        let merged = base.merge(local);
        assert_eq!(
            merged.symlinks,
            vec![".envrc".to_string(), ".tool-versions".to_string()]
        );
    }

    #[test]
    fn test_merge_empty_symlinks_no_override() {
        let base = base_config();
        let local = Config {
            symlinks: vec![],
            ..Config::default()
        };
        let merged = base.merge(local);
        assert_eq!(merged.symlinks, vec![".env".to_string()]);
    }

    #[test]
    fn test_merge_extra_symlinks_dedup() {
        let base = base_config();
        let local = Config {
            extra_symlinks: vec!["extra1".to_string(), "extra2".to_string()],
            ..Config::default()
        };
        let merged = base.merge(local);
        assert_eq!(
            merged.extra_symlinks,
            vec!["extra1".to_string(), "extra2".to_string()]
        );
    }

    #[test]
    fn test_merge_booleans_or_only() {
        let base = Config {
            init_submodules: false,
            submodule_reference: false,
            ..Config::default()
        };
        let local = Config {
            init_submodules: true,
            submodule_reference: true,
            ..Config::default()
        };
        let merged = base.merge(local);
        assert!(merged.init_submodules);
        assert!(merged.submodule_reference);

        // Once true, local false does not revert
        let base2 = Config {
            init_submodules: true,
            submodule_reference: true,
            ..Config::default()
        };
        let local2 = Config {
            init_submodules: false,
            submodule_reference: false,
            ..Config::default()
        };
        let merged2 = base2.merge(local2);
        assert!(merged2.init_submodules);
        assert!(merged2.submodule_reference);
    }

    #[test]
    fn test_merge_wezterm_override() {
        let base = base_config();
        let local = Config {
            wezterm: Some(WeztermConfig {
                auto_open: false,
                tab_title_template: Some("local-tmpl".to_string()),
            }),
            ..Config::default()
        };
        let merged = base.merge(local);
        let wez = merged.wezterm.unwrap();
        assert!(!wez.auto_open);
        assert_eq!(wez.tab_title_template, Some("local-tmpl".to_string()));
    }

    #[test]
    fn test_all_symlinks_dedup() {
        let cfg = Config {
            symlinks: vec![".env".to_string(), ".envrc".to_string()],
            extra_symlinks: vec![".env".to_string(), "Makefile".to_string()],
            ..Config::default()
        };
        let all = cfg.all_symlinks();
        assert_eq!(
            all,
            vec![
                ".env".to_string(),
                ".envrc".to_string(),
                "Makefile".to_string()
            ]
        );
    }

    #[test]
    fn test_worktree_base_dir_absolute() {
        let cfg = Config {
            worktree_destination_base_dir: Some("/absolute/path".to_string()),
            ..Config::default()
        };
        let result = cfg.worktree_base_dir(Path::new("/repo/root"), Path::new("/repo/root/.git"));
        assert_eq!(result, PathBuf::from("/absolute/path"));
    }

    #[test]
    fn test_worktree_base_dir_relative() {
        let cfg = Config {
            worktree_destination_base_dir: Some("../worktrees".to_string()),
            ..Config::default()
        };
        let result = cfg.worktree_base_dir(Path::new("/repo/root"), Path::new("/repo/root/.git"));
        assert_eq!(result, PathBuf::from("/repo/root/../worktrees"));
    }

    #[test]
    fn test_worktree_base_dir_none() {
        let cfg = Config::default();
        let result = cfg.worktree_base_dir(
            Path::new("/home/user/myrepo"),
            Path::new("/home/user/myrepo/.git"),
        );
        assert_eq!(result, PathBuf::from("/home/user/myrepo/.git/ou-worktrees"));
    }

    #[test]
    fn test_default_source_branch() {
        let cfg = Config::default();
        assert_eq!(cfg.default_source_branch(), "main");

        let cfg2 = Config {
            default_source: Some("develop".to_string()),
            ..Config::default()
        };
        assert_eq!(cfg2.default_source_branch(), "develop");
    }

    #[test]
    fn test_default_toml_roundtrip() {
        let toml_str = Config::default_toml();
        let parsed: Result<Config, _> = toml::from_str(&toml_str);
        assert!(parsed.is_ok(), "default_toml() should be valid TOML");
    }

    #[test]
    fn test_load_file_exists() {
        let toml_content = r#"
worktree_destination_base_dir = "../wt"
default_source = "develop"
symlinks = [".env"]
"#;
        let fs = MockFileSystem::new()
            .with_dir(PathBuf::from("/repo/.ou"))
            .with_file(PathBuf::from("/repo/.ou/settings.toml"), toml_content);
        let cfg = Config::load(Path::new("/repo"), &fs).unwrap();
        assert_eq!(cfg.worktree_destination_base_dir, Some("../wt".to_string()));
        assert_eq!(cfg.default_source, Some("develop".to_string()));
        assert_eq!(cfg.symlinks, vec![".env".to_string()]);
    }

    #[test]
    fn test_load_file_not_exists() {
        let fs = MockFileSystem::new();
        let cfg = Config::load(Path::new("/repo"), &fs).unwrap();
        assert_eq!(cfg.worktree_destination_base_dir, None);
        assert_eq!(cfg.default_source, None);
        assert!(cfg.symlinks.is_empty());
    }

    #[test]
    fn test_load_with_local_override() {
        let base_toml = r#"
worktree_destination_base_dir = "../base"
default_source = "main"
symlinks = [".env"]
extra_symlinks = ["a"]
"#;
        let local_toml = r#"
worktree_destination_base_dir = "../local"
extra_symlinks = ["b"]
"#;
        let fs = MockFileSystem::new()
            .with_dir(PathBuf::from("/repo/.ou"))
            .with_file(PathBuf::from("/repo/.ou/settings.toml"), base_toml)
            .with_file(PathBuf::from("/repo/.ou/settings.local.toml"), local_toml);
        let cfg = Config::load(Path::new("/repo"), &fs).unwrap();
        assert_eq!(
            cfg.worktree_destination_base_dir,
            Some("../local".to_string())
        );
        assert_eq!(cfg.default_source, Some("main".to_string()));
        assert_eq!(cfg.symlinks, vec![".env".to_string()]);
        assert_eq!(cfg.extra_symlinks, vec!["a".to_string(), "b".to_string()]);
    }

    #[test]
    fn test_load_invalid_toml() {
        let fs = MockFileSystem::new()
            .with_dir(PathBuf::from("/repo/.ou"))
            .with_file(
                PathBuf::from("/repo/.ou/settings.toml"),
                "this is not valid toml {{{}}}",
            );
        let result = Config::load(Path::new("/repo"), &fs);
        assert!(result.is_err());
        match result.unwrap_err() {
            OuError::Config(msg) => assert!(msg.contains("failed to parse")),
            other => panic!("expected OuError::Config, got: {other:?}"),
        }
    }
}
