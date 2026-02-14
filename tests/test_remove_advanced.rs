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

#[test]
fn test_remove_no_args() {
    let repo = setup_git_repo();
    let path = repo.path();

    ou_cmd()
        .args(["remove"])
        .current_dir(path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("no branches specified"));
}

#[test]
fn test_remove_multiple_branches() {
    let repo = setup_git_repo();
    let path = repo.path();

    ou_cmd()
        .args(["init"])
        .current_dir(path)
        .assert()
        .success();

    ou_cmd()
        .args(["add", "feat/rm-a"])
        .current_dir(path)
        .assert()
        .success();

    ou_cmd()
        .args(["add", "feat/rm-b"])
        .current_dir(path)
        .assert()
        .success();

    ou_cmd()
        .args(["remove", "feat/rm-a", "feat/rm-b"])
        .current_dir(path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Removed: feat/rm-a, feat/rm-b"));
}

#[test]
fn test_remove_with_force() {
    let repo = setup_git_repo();
    let path = repo.path();

    ou_cmd()
        .args(["init"])
        .current_dir(path)
        .assert()
        .success();

    ou_cmd()
        .args(["add", "feat/dirty"])
        .current_dir(path)
        .assert()
        .success();

    // Find the worktree directory and create an uncommitted file
    let wt_dir = path
        .join(".git")
        .join("ou-worktrees")
        .join("feat-dirty");
    std::fs::write(wt_dir.join("uncommitted.txt"), "dirty content\n").unwrap();

    // Remove without force should fail
    ou_cmd()
        .args(["remove", "feat/dirty"])
        .current_dir(path)
        .assert()
        .failure();

    // Remove with force should succeed
    ou_cmd()
        .args(["remove", "feat/dirty", "-f"])
        .current_dir(path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Removed: feat/dirty"));
}

#[test]
fn test_remove_locked_needs_double_force() {
    let repo = setup_git_repo();
    let path = repo.path();

    ou_cmd()
        .args(["init"])
        .current_dir(path)
        .assert()
        .success();

    ou_cmd()
        .args(["add", "feat/locked-rm", "--lock", "--reason", "do not delete"])
        .current_dir(path)
        .assert()
        .success();

    // Remove without force should fail (locked)
    ou_cmd()
        .args(["remove", "feat/locked-rm"])
        .current_dir(path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("locked"));

    // Remove with single -f should still fail (locked requires -ff)
    ou_cmd()
        .args(["remove", "feat/locked-rm", "-f"])
        .current_dir(path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("locked"));

    // Remove with -ff should succeed
    ou_cmd()
        .args(["remove", "feat/locked-rm", "-ff"])
        .current_dir(path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Removed: feat/locked-rm"));
}

#[test]
fn test_remove_mixed_success_error() {
    let repo = setup_git_repo();
    let path = repo.path();

    ou_cmd()
        .args(["init"])
        .current_dir(path)
        .assert()
        .success();

    ou_cmd()
        .args(["add", "feat/exists"])
        .current_dir(path)
        .assert()
        .success();

    // Remove existing + nonexistent
    let output = ou_cmd()
        .args(["remove", "feat/exists", "nonexistent"])
        .current_dir(path)
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Removed: feat/exists"),
        "should contain removed message, got: {stdout}"
    );
    assert!(
        stdout.contains("not found"),
        "should contain not found error, got: {stdout}"
    );
}
