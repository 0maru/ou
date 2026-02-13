use std::process::Command;

use assert_cmd::prelude::*;
use predicates::prelude::*;
use tempfile::TempDir;

fn setup_git_repo() -> TempDir {
    let dir = TempDir::new().unwrap();
    let path = dir.path();

    Command::new("git")
        .args(["init", "--initial-branch=main"])
        .current_dir(path)
        .output()
        .unwrap();

    Command::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(path)
        .output()
        .unwrap();

    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(path)
        .output()
        .unwrap();

    std::fs::write(path.join("README.md"), "# test\n").unwrap();
    Command::new("git")
        .args(["add", "."])
        .current_dir(path)
        .output()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "initial"])
        .current_dir(path)
        .output()
        .unwrap();

    dir
}

fn ou_cmd() -> Command {
    Command::cargo_bin("ou").unwrap()
}

/// Get the worktree directory path for a given branch name
fn worktree_dir(repo_path: &std::path::Path, branch: &str) -> std::path::PathBuf {
    let repo_name = repo_path.file_name().unwrap().to_string_lossy();
    let wt_base = repo_path
        .parent()
        .unwrap()
        .join(format!("{repo_name}-worktrees"));
    // ou converts "/" to "-" in worktree directory names
    wt_base.join(branch.replace('/', "-"))
}

/// Make a commit in a worktree directory
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

/// Merge a branch into main at the repo root
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

    ou_cmd()
        .args(["init"])
        .current_dir(path)
        .assert()
        .success();

    ou_cmd()
        .args(["add", "feat/to-merge"])
        .current_dir(path)
        .assert()
        .success();

    let wt_dir = worktree_dir(path, "feat/to-merge");

    // Make a commit in the worktree
    commit_in_worktree(&wt_dir, "feature.txt", "add feature");

    // Merge into main
    merge_branch(path, "feat/to-merge");

    // Run clean
    ou_cmd()
        .args(["clean"])
        .current_dir(path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Cleaned:").and(predicate::str::contains("feat/to-merge")));

    // Verify worktree directory no longer exists
    assert!(
        !wt_dir.exists(),
        "worktree directory should be removed after clean"
    );
}

#[test]
fn test_clean_check_shows_candidates() {
    let repo = setup_git_repo();
    let path = repo.path();

    ou_cmd()
        .args(["init"])
        .current_dir(path)
        .assert()
        .success();

    ou_cmd()
        .args(["add", "feat/check-merge"])
        .current_dir(path)
        .assert()
        .success();

    let wt_dir = worktree_dir(path, "feat/check-merge");

    // Make a commit and merge
    commit_in_worktree(&wt_dir, "check.txt", "add check feature");
    merge_branch(path, "feat/check-merge");

    // Run clean --check (dry run)
    ou_cmd()
        .args(["clean", "--check"])
        .current_dir(path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Would remove:"));

    // Verify worktree directory still exists (dry run)
    assert!(
        wt_dir.exists(),
        "worktree directory should still exist after --check"
    );
}

#[test]
fn test_clean_no_candidates() {
    let repo = setup_git_repo();
    let path = repo.path();

    ou_cmd()
        .args(["init"])
        .current_dir(path)
        .assert()
        .success();

    // Add a worktree and make a commit (so it's not a subset of main)
    ou_cmd()
        .args(["add", "feat/not-merged"])
        .current_dir(path)
        .assert()
        .success();

    let wt_dir = worktree_dir(path, "feat/not-merged");
    commit_in_worktree(&wt_dir, "unmerged.txt", "unmerged work");

    // Run clean without merging the branch into main
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

    ou_cmd()
        .args(["init"])
        .current_dir(path)
        .assert()
        .success();

    // Add two worktrees
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

    // Make commits in both worktrees
    commit_in_worktree(&wt_a, "feature_a.txt", "add feature a");
    commit_in_worktree(&wt_b, "feature_b.txt", "add feature b");

    // Merge both into main
    merge_branch(path, "feat/multi-a");
    merge_branch(path, "feat/multi-b");

    // Run clean
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

    // Verify both worktree directories are removed
    assert!(
        !wt_a.exists(),
        "worktree directory for feat/multi-a should be removed"
    );
    assert!(
        !wt_b.exists(),
        "worktree directory for feat/multi-b should be removed"
    );
}
