mod common;

use assert_cmd::prelude::*;
use predicates::prelude::*;

use common::{ou_cmd, setup_git_repo};

#[test]
fn test_help() {
    ou_cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Git worktree management CLI"));
}

#[test]
fn test_init() {
    let repo = setup_git_repo();
    let path = repo.path();

    ou_cmd()
        .args(["init"])
        .current_dir(path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Initialized ou"));

    assert!(path.join(".ou/settings.toml").exists());
    assert!(path.join(".ou/.gitignore").exists());

    let content = std::fs::read_to_string(path.join(".ou/settings.toml")).unwrap();
    assert!(content.contains("default_source = \"main\""));
}

#[test]
fn test_init_already_initialized() {
    let repo = setup_git_repo();
    let path = repo.path();

    ou_cmd().args(["init"]).current_dir(path).assert().success();

    ou_cmd()
        .args(["init"])
        .current_dir(path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("already initialized"));
}

#[test]
fn test_list() {
    let repo = setup_git_repo();
    let path = repo.path();

    ou_cmd()
        .args(["list"])
        .current_dir(path)
        .assert()
        .success()
        .stdout(predicate::str::contains("main"));
}

#[test]
fn test_list_quiet() {
    let repo = setup_git_repo();
    let path = repo.path();

    let output = ou_cmd()
        .args(["list", "--quiet"])
        .current_dir(path)
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let canonical = path.canonicalize().unwrap();
    assert!(
        stdout.contains(canonical.to_str().unwrap()),
        "stdout should contain canonical path {canonical:?}, got: {stdout}"
    );
}

#[test]
fn test_add_and_remove() {
    let repo = setup_git_repo();
    let path = repo.path();

    ou_cmd().args(["init"]).current_dir(path).assert().success();

    ou_cmd()
        .args(["add", "feat/test"])
        .current_dir(path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Created worktree 'feat/test'"));

    ou_cmd()
        .args(["list"])
        .current_dir(path)
        .assert()
        .success()
        .stdout(predicate::str::contains("feat/test"));

    ou_cmd()
        .args(["remove", "feat/test"])
        .current_dir(path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Removed: feat/test"));
}

#[test]
fn test_add_with_lock() {
    let repo = setup_git_repo();
    let path = repo.path();

    common::require_git!(2, 15);

    ou_cmd().args(["init"]).current_dir(path).assert().success();

    ou_cmd()
        .args([
            "add",
            "feat/locked",
            "--lock",
            "--reason",
            "work in progress",
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

    ou_cmd()
        .args(["remove", "feat/locked"])
        .current_dir(path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("locked"));
}

#[test]
fn test_add_with_symlinks() {
    let repo = setup_git_repo();
    let path = repo.path();

    std::fs::write(path.join(".env"), "SECRET=test\n").unwrap();

    ou_cmd().args(["init"]).current_dir(path).assert().success();

    ou_cmd()
        .args(["add", "feat/symlink-test"])
        .current_dir(path)
        .assert()
        .success();

    let repo_name = path.file_name().unwrap().to_string_lossy();
    let wt_dir = path
        .parent()
        .unwrap()
        .join(format!("{repo_name}-worktrees"))
        .join("feat-symlink-test");

    let env_link = wt_dir.join(".env");
    assert!(
        env_link
            .symlink_metadata()
            .unwrap()
            .file_type()
            .is_symlink(),
        ".env should be a symlink in the worktree"
    );
}

#[test]
fn test_clean_check() {
    let repo = setup_git_repo();
    let path = repo.path();

    ou_cmd().args(["init"]).current_dir(path).assert().success();

    ou_cmd()
        .args(["clean", "--check"])
        .current_dir(path)
        .assert()
        .success()
        .stdout(predicate::str::contains("No worktrees to clean"));
}

#[test]
fn test_remove_nonexistent() {
    let repo = setup_git_repo();
    let path = repo.path();

    ou_cmd()
        .args(["remove", "nonexistent"])
        .current_dir(path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}
