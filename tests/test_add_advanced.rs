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
fn test_add_with_source() {
    let repo = setup_git_repo();
    let path = repo.path();

    // Init ou
    ou_cmd()
        .args(["init"])
        .current_dir(path)
        .assert()
        .success();

    // Create develop branch
    Command::new("git")
        .args(["checkout", "-b", "develop"])
        .current_dir(path)
        .output()
        .unwrap();
    Command::new("git")
        .args(["checkout", "main"])
        .current_dir(path)
        .output()
        .unwrap();

    // Add worktree from develop
    ou_cmd()
        .args(["add", "feat/from-dev", "--source", "develop"])
        .current_dir(path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Created worktree 'feat/from-dev'"));
}

#[test]
fn test_add_duplicate_name() {
    let repo = setup_git_repo();
    let path = repo.path();

    ou_cmd()
        .args(["init"])
        .current_dir(path)
        .assert()
        .success();

    // First add
    ou_cmd()
        .args(["add", "feat/dup"])
        .current_dir(path)
        .assert()
        .success();

    // Second add should fail
    ou_cmd()
        .args(["add", "feat/dup"])
        .current_dir(path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));
}

#[test]
fn test_add_with_carry() {
    let repo = setup_git_repo();
    let path = repo.path();

    ou_cmd()
        .args(["init"])
        .current_dir(path)
        .assert()
        .success();

    // Create uncommitted change
    std::fs::write(path.join("README.md"), "# test\ndirty\n").unwrap();

    // Add with carry should succeed even with uncommitted changes
    ou_cmd()
        .args(["add", "feat/carried", "--carry"])
        .current_dir(path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Created worktree 'feat/carried'"));
}

#[test]
fn test_add_slash_to_dash() {
    let repo = setup_git_repo();
    let path = repo.path();

    ou_cmd()
        .args(["init"])
        .current_dir(path)
        .assert()
        .success();

    // Add worktree with slash in name
    let output = ou_cmd()
        .args(["add", "feat/slash-test"])
        .current_dir(path)
        .output()
        .unwrap();

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    // The worktree directory name should use dash instead of slash
    assert!(
        stdout.contains("feat-slash-test"),
        "worktree path should contain 'feat-slash-test', got: {stdout}"
    );
}

#[test]
fn test_add_lock_with_reason() {
    let repo = setup_git_repo();
    let path = repo.path();

    ou_cmd()
        .args(["init"])
        .current_dir(path)
        .assert()
        .success();

    // Add worktree with lock and reason
    ou_cmd()
        .args(["add", "feat/locked-reason", "--lock", "--reason", "testing lock"])
        .current_dir(path)
        .assert()
        .success()
        .stdout(predicate::str::contains("[locked]"));

    // Verify locked status in list
    ou_cmd()
        .args(["list"])
        .current_dir(path)
        .assert()
        .success()
        .stdout(predicate::str::contains("[locked]"));
}
