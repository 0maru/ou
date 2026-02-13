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

    // Create initial commit
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
    Command::new(assert_cmd::cargo::cargo_bin!("ou"))
}

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
    assert!(stdout.contains(path.to_str().unwrap()));
}

#[test]
fn test_add_and_remove() {
    let repo = setup_git_repo();
    let path = repo.path();

    // Init ou first
    ou_cmd().args(["init"]).current_dir(path).assert().success();

    // Add worktree
    ou_cmd()
        .args(["add", "feat/test"])
        .current_dir(path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Created worktree 'feat/test'"));

    // Verify worktree exists in list
    ou_cmd()
        .args(["list"])
        .current_dir(path)
        .assert()
        .success()
        .stdout(predicate::str::contains("feat/test"));

    // Remove worktree
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

    // List should show locked
    ou_cmd()
        .args(["list"])
        .current_dir(path)
        .assert()
        .success()
        .stdout(predicate::str::contains("[locked]"));

    // Remove without -ff should fail
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

    // Create .env file in repo root
    std::fs::write(path.join(".env"), "SECRET=test\n").unwrap();

    ou_cmd().args(["init"]).current_dir(path).assert().success();

    ou_cmd()
        .args(["add", "feat/symlink-test"])
        .current_dir(path)
        .assert()
        .success();

    // Check symlink was created
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

    // No merged worktrees to clean
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
