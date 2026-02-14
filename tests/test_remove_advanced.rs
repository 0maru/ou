mod common;

use assert_cmd::prelude::*;
use predicates::prelude::*;

use common::{ou_cmd, setup_git_repo};

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

    ou_cmd().args(["init"]).current_dir(path).assert().success();

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

    ou_cmd().args(["init"]).current_dir(path).assert().success();

    ou_cmd()
        .args(["add", "feat/dirty"])
        .current_dir(path)
        .assert()
        .success();

    let wt_dir = path
        .join(".git")
        .join("ou-worktrees")
        .join("feat-dirty");
    std::fs::write(wt_dir.join("uncommitted.txt"), "dirty content\n").unwrap();

    ou_cmd()
        .args(["remove", "feat/dirty"])
        .current_dir(path)
        .assert()
        .failure();

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

    common::require_git!(2, 15);

    ou_cmd().args(["init"]).current_dir(path).assert().success();

    ou_cmd()
        .args([
            "add",
            "feat/locked-rm",
            "--lock",
            "--reason",
            "do not delete",
        ])
        .current_dir(path)
        .assert()
        .success();

    ou_cmd()
        .args(["remove", "feat/locked-rm"])
        .current_dir(path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("locked"));

    ou_cmd()
        .args(["remove", "feat/locked-rm", "-f"])
        .current_dir(path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("locked"));

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

    ou_cmd().args(["init"]).current_dir(path).assert().success();

    ou_cmd()
        .args(["add", "feat/exists"])
        .current_dir(path)
        .assert()
        .success();

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
