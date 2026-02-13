use std::path::{Path, PathBuf};

use crate::error::OuError;
use crate::git::executor::GitExecutor;
use crate::git::types::{Branch, CommandOutput, MergeStatus, Worktree};

pub struct GitRunner<E: GitExecutor> {
    executor: E,
    repo_dir: PathBuf,
}

impl<E: GitExecutor> GitRunner<E> {
    pub fn new(executor: E, repo_dir: PathBuf) -> Self {
        Self { executor, repo_dir }
    }

    fn run(&self, args: &[&str]) -> Result<CommandOutput, OuError> {
        let dir_str = self.repo_dir.to_string_lossy().to_string();
        let mut full_args = vec!["-C", &dir_str];
        full_args.extend(args);
        self.executor.run(&full_args)
    }

    fn run_ok(&self, args: &[&str]) -> Result<String, OuError> {
        let output = self.run(args)?;
        if output.success() {
            Ok(output.stdout)
        } else {
            Err(OuError::Git(output.stderr.trim().to_string()))
        }
    }

    #[allow(dead_code)]
    pub fn is_git_repo(&self) -> bool {
        self.run(&["rev-parse", "--git-dir"])
            .is_ok_and(|o| o.success())
    }

    pub fn get_toplevel(&self) -> Result<PathBuf, OuError> {
        let out = self.run_ok(&["rev-parse", "--show-toplevel"])?;
        Ok(PathBuf::from(out.trim()))
    }

    #[allow(dead_code)]
    pub fn get_common_dir(&self) -> Result<PathBuf, OuError> {
        let out = self.run_ok(&["rev-parse", "--git-common-dir"])?;
        let p = PathBuf::from(out.trim());
        if p.is_absolute() {
            Ok(p)
        } else {
            Ok(self.repo_dir.join(p))
        }
    }

    #[allow(dead_code)]
    pub fn get_current_branch(&self) -> Result<Option<String>, OuError> {
        let output = self.run(&["symbolic-ref", "--short", "HEAD"])?;
        if output.success() {
            Ok(Some(output.stdout.trim().to_string()))
        } else {
            Ok(None)
        }
    }

    pub fn worktree_list(&self) -> Result<Vec<Worktree>, OuError> {
        let out = self.run_ok(&["worktree", "list", "--porcelain"])?;
        parse_worktree_list(&out)
    }

    pub fn worktree_add(
        &self,
        path: &Path,
        branch: &str,
        source: Option<&str>,
    ) -> Result<(), OuError> {
        let path_str = path.to_string_lossy().to_string();
        let mut args = vec!["worktree", "add", "-b", branch, &path_str];
        if let Some(src) = source {
            args.push(src);
        }
        self.run_ok(&args)?;
        Ok(())
    }

    pub fn worktree_remove(&self, path: &Path, force: bool) -> Result<(), OuError> {
        let path_str = path.to_string_lossy().to_string();
        let mut args = vec!["worktree", "remove", &path_str];
        if force {
            args.push("--force");
        }
        self.run_ok(&args)?;
        Ok(())
    }

    pub fn worktree_lock(&self, path: &Path, reason: Option<&str>) -> Result<(), OuError> {
        let path_str = path.to_string_lossy().to_string();
        let mut args = vec!["worktree", "lock", &path_str];
        if let Some(r) = reason {
            args.push("--reason");
            args.push(r);
        }
        self.run_ok(&args)?;
        Ok(())
    }

    pub fn worktree_unlock(&self, path: &Path) -> Result<(), OuError> {
        let path_str = path.to_string_lossy().to_string();
        self.run_ok(&["worktree", "unlock", &path_str])?;
        Ok(())
    }

    pub fn branch_list(&self) -> Result<Vec<Branch>, OuError> {
        let out = self.run_ok(&[
            "for-each-ref",
            "--format=%(refname:short)\t%(upstream:short)\t%(HEAD)\t%(upstream:track)",
            "refs/heads/",
        ])?;
        parse_branch_list(&out)
    }

    pub fn branch_delete(&self, name: &str, force: bool) -> Result<(), OuError> {
        let flag = if force { "-D" } else { "-d" };
        self.run_ok(&["branch", flag, name])?;
        Ok(())
    }

    pub fn is_branch_merged(&self, branch: &str, target: &str) -> Result<MergeStatus, OuError> {
        let output = self.run(&["merge-base", "--is-ancestor", branch, target])?;
        if output.success() {
            Ok(MergeStatus::Merged)
        } else {
            Ok(MergeStatus::NotMerged)
        }
    }

    pub fn stash_push(&self, message: &str) -> Result<bool, OuError> {
        let output = self.run_ok(&["stash", "push", "-m", message])?;
        Ok(!output.contains("No local changes"))
    }

    pub fn stash_pop(&self) -> Result<(), OuError> {
        self.run_ok(&["stash", "pop"])?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn has_uncommitted_changes(&self) -> Result<bool, OuError> {
        let output = self.run_ok(&["status", "--porcelain"])?;
        Ok(!output.trim().is_empty())
    }

    pub fn init_submodules(&self, path: &Path) -> Result<(), OuError> {
        let path_str = path.to_string_lossy().to_string();
        self.run_ok(&[
            "-C",
            &path_str,
            "submodule",
            "update",
            "--init",
            "--recursive",
        ])?;
        Ok(())
    }

    pub fn default_branch(&self) -> Result<String, OuError> {
        let output = self.run(&["symbolic-ref", "refs/remotes/origin/HEAD"]);
        if let Ok(out) = output {
            if out.success() {
                let full = out.stdout.trim();
                if let Some(name) = full.strip_prefix("refs/remotes/origin/") {
                    return Ok(name.to_string());
                }
            }
        }

        for candidate in &["main", "master"] {
            let check = self.run(&["rev-parse", "--verify", candidate]);
            if check.is_ok_and(|o| o.success()) {
                return Ok(candidate.to_string());
            }
        }

        Ok("main".to_string())
    }
}

fn parse_worktree_list(output: &str) -> Result<Vec<Worktree>, OuError> {
    let mut worktrees = Vec::new();
    let mut current_path: Option<PathBuf> = None;
    let mut current_branch: Option<String> = None;
    let mut current_head = String::new();
    let mut is_bare = false;
    let mut is_locked = false;
    let mut lock_reason: Option<String> = None;
    let mut is_prunable = false;

    for line in output.lines() {
        if line.is_empty() {
            if let Some(path) = current_path.take() {
                worktrees.push(Worktree {
                    path,
                    branch: current_branch.take(),
                    head: std::mem::take(&mut current_head),
                    is_bare,
                    is_locked,
                    lock_reason: lock_reason.take(),
                    is_prunable,
                });
                is_bare = false;
                is_locked = false;
                is_prunable = false;
            }
            continue;
        }

        if let Some(path) = line.strip_prefix("worktree ") {
            current_path = Some(PathBuf::from(path));
        } else if let Some(head) = line.strip_prefix("HEAD ") {
            current_head = head.to_string();
        } else if let Some(branch) = line.strip_prefix("branch ") {
            let name = branch.strip_prefix("refs/heads/").unwrap_or(branch);
            current_branch = Some(name.to_string());
        } else if line == "bare" {
            is_bare = true;
        } else if line == "locked" {
            is_locked = true;
        } else if let Some(reason) = line.strip_prefix("locked ") {
            is_locked = true;
            lock_reason = Some(reason.to_string());
        } else if line == "prunable" {
            is_prunable = true;
        }
    }

    if let Some(path) = current_path.take() {
        worktrees.push(Worktree {
            path,
            branch: current_branch.take(),
            head: current_head,
            is_bare,
            is_locked,
            lock_reason: lock_reason.take(),
            is_prunable,
        });
    }

    Ok(worktrees)
}

fn parse_branch_list(output: &str) -> Result<Vec<Branch>, OuError> {
    let mut branches = Vec::new();
    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.is_empty() {
            continue;
        }
        let name = parts[0].to_string();
        let upstream = parts.get(1).and_then(|s| {
            if s.is_empty() {
                None
            } else {
                Some(s.to_string())
            }
        });
        let is_head = parts.get(2).is_some_and(|s| s.trim() == "*");
        let gone = parts.get(3).is_some_and(|s| s.contains("[gone]"));

        branches.push(Branch {
            name,
            upstream,
            is_head,
            gone,
        });
    }
    Ok(branches)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_worktree_list() {
        let input = "\
worktree /home/user/project
HEAD abc1234567890
branch refs/heads/main

worktree /home/user/project-feat
HEAD def4567890123
branch refs/heads/feat/test
locked reason for lock

";
        let wts = parse_worktree_list(input).unwrap();
        assert_eq!(wts.len(), 2);
        assert_eq!(wts[0].path, PathBuf::from("/home/user/project"));
        assert_eq!(wts[0].branch.as_deref(), Some("main"));
        assert!(!wts[0].is_locked);

        assert_eq!(wts[1].branch.as_deref(), Some("feat/test"));
        assert!(wts[1].is_locked);
        assert_eq!(wts[1].lock_reason.as_deref(), Some("reason for lock"));
    }

    #[test]
    fn test_parse_branch_list() {
        let input = "main\torigin/main\t*\t\nfeat/test\torigin/feat/test\t \t[gone]\n";
        let branches = parse_branch_list(input).unwrap();
        assert_eq!(branches.len(), 2);
        assert_eq!(branches[0].name, "main");
        assert!(branches[0].is_head);
        assert!(!branches[0].gone);

        assert_eq!(branches[1].name, "feat/test");
        assert!(!branches[1].is_head);
        assert!(branches[1].gone);
    }
}
