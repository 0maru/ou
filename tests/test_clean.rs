mod common;

use std::process::Command;

use assert_cmd::prelude::*;
use predicates::prelude::*;

use common::{ou_cmd, setup_git_repo};

fn worktree_dir(repo_path: &std::path::Path, branch: &str) -> std::path::PathBuf {
    let repo_name = repo_path.file_name().unwrap().to_string_lossy();
    let wt_base = repo_path
        .parent()
        .unwrap()
        .join(format!("{repo_name}-worktrees"));
    wt_base.join(branch.replace('/', "-"))
}

fn commit_in_worktree(wt_path: &std::path::Path, filename: &str, message: &str) {
    std::fs::write(wt_path.join(filename), "content\n").unwrap();
    Command::new("git")
        .args(["add", "."])
        .current_dir(wt_path)
        .output()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", message])
        .current_dir(wt_path)
        .output()
        .unwrap();
}

fn merge_branch(repo_path: &std::path::Path, branch: &str) {
    Command::new("git")
        .args(["merge", branch])
        .current_dir(repo_path)
        .output()
        .unwrap();
}

#[test]
fn test_clean_removes_merged_branch() {
    let repo = setup_git_repo();
    let path = repo.path();

    ou_cmd().args(["init"]).current_dir(path).assert().success();

    ou_cmd()
        .args(["add", "feat/to-merge"])
        .current_dir(path)
        .assert()
        .success();

    let wt_dir = worktree_dir(path, "feat/to-merge");

    commit_in_worktree(&wt_dir, "feature.txt", "add feature");

    merge_branch(path, "feat/to-merge");

    ou_cmd()
        .args(["clean"])
        .current_dir(path)
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Cleaned:").and(predicate::str::contains("feat/to-merge")),
        );

    assert!(
        !wt_dir.exists(),
        "worktree directory should be removed after clean"
    );
}

#[test]
fn test_clean_check_shows_candidates() {
    let repo = setup_git_repo();
    let path = repo.path();

    ou_cmd().args(["init"]).current_dir(path).assert().success();

    ou_cmd()
        .args(["add", "feat/check-merge"])
        .current_dir(path)
        .assert()
        .success();

    let wt_dir = worktree_dir(path, "feat/check-merge");

    commit_in_worktree(&wt_dir, "check.txt", "add check feature");
    merge_branch(path, "feat/check-merge");

    ou_cmd()
        .args(["clean", "--check"])
        .current_dir(path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Would remove:"));

    assert!(
        wt_dir.exists(),
        "worktree directory should still exist after --check"
    );
}

#[test]
fn test_clean_no_candidates() {
    let repo = setup_git_repo();
    let path = repo.path();

    ou_cmd().args(["init"]).current_dir(path).assert().success();

    ou_cmd()
        .args(["add", "feat/not-merged"])
        .current_dir(path)
        .assert()
        .success();

    let wt_dir = worktree_dir(path, "feat/not-merged");
    commit_in_worktree(&wt_dir, "unmerged.txt", "unmerged work");

    ou_cmd()
        .args(["clean"])
        .current_dir(path)
        .assert()
        .success()
        .stdout(predicate::str::contains("No worktrees to clean"));
}

#[test]
fn test_clean_multiple_candidates() {
    let repo = setup_git_repo();
    let path = repo.path();

    ou_cmd().args(["init"]).current_dir(path).assert().success();

    ou_cmd()
        .args(["add", "feat/multi-a"])
        .current_dir(path)
        .assert()
        .success();

    ou_cmd()
        .args(["add", "feat/multi-b"])
        .current_dir(path)
        .assert()
        .success();

    let wt_a = worktree_dir(path, "feat/multi-a");
    let wt_b = worktree_dir(path, "feat/multi-b");

    commit_in_worktree(&wt_a, "feature_a.txt", "add feature a");
    commit_in_worktree(&wt_b, "feature_b.txt", "add feature b");

    merge_branch(path, "feat/multi-a");
    merge_branch(path, "feat/multi-b");

    ou_cmd()
        .args(["clean"])
        .current_dir(path)
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Cleaned:")
                .and(predicate::str::contains("feat/multi-a"))
                .and(predicate::str::contains("feat/multi-b")),
        );

    assert!(
        !wt_a.exists(),
        "worktree directory for feat/multi-a should be removed"
    );
    assert!(
        !wt_b.exists(),
        "worktree directory for feat/multi-b should be removed"
    );
}
