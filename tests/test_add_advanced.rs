mod common;

use std::process::Command;

use assert_cmd::prelude::*;
use predicates::prelude::*;

use common::{ou_cmd, setup_git_repo};

#[test]
fn test_add_with_source() {
    let repo = setup_git_repo();
    let path = repo.path();

    ou_cmd().args(["init"]).current_dir(path).assert().success();

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

    ou_cmd().args(["init"]).current_dir(path).assert().success();

    ou_cmd()
        .args(["add", "feat/dup"])
        .current_dir(path)
        .assert()
        .success();

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

    ou_cmd().args(["init"]).current_dir(path).assert().success();

    std::fs::write(path.join("README.md"), "# test\ndirty\n").unwrap();

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

    ou_cmd().args(["init"]).current_dir(path).assert().success();

    let output = ou_cmd()
        .args(["add", "feat/slash-test"])
        .current_dir(path)
        .output()
        .unwrap();

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("feat-slash-test"),
        "worktree path should contain 'feat-slash-test', got: {stdout}"
    );
}

#[test]
fn test_add_lock_with_reason() {
    let repo = setup_git_repo();
    let path = repo.path();

    common::require_git!(2, 15);

    ou_cmd().args(["init"]).current_dir(path).assert().success();

    ou_cmd()
        .args([
            "add",
            "feat/locked-reason",
            "--lock",
            "--reason",
            "testing lock",
        ])
        .current_dir(path)
        .assert()
        .success()
        .stdout(predicate::str::contains("[locked]"));

    ou_cmd()
        .args(["list"])
        .current_dir(path)
        .assert()
        .success()
        .stdout(predicate::str::contains("[locked]"));
}
