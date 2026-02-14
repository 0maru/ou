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
fn test_sync_all_syncs_worktrees() {
    let repo = setup_git_repo();
    let path = repo.path();

    // Create .env file in repo root
    std::fs::write(path.join(".env"), "SECRET=test\n").unwrap();

    // Init ou
    ou_cmd()
        .args(["init"])
        .current_dir(path)
        .assert()
        .success();

    // Add two worktrees
    ou_cmd()
        .args(["add", "feat/sync-a"])
        .current_dir(path)
        .assert()
        .success();

    ou_cmd()
        .args(["add", "feat/sync-b"])
        .current_dir(path)
        .assert()
        .success();

    // Run sync --all from repo root
    ou_cmd()
        .args(["sync", "--all"])
        .current_dir(path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Synced:"));

    // Verify .env symlinks exist in both worktrees
    let wt_base = path.join(".git").join("ou-worktrees");

    let env_a = wt_base.join("feat-sync-a").join(".env");
    let env_b = wt_base.join("feat-sync-b").join(".env");

    assert!(
        env_a.symlink_metadata().unwrap().file_type().is_symlink(),
        ".env should be a symlink in feat/sync-a worktree"
    );
    assert!(
        env_b.symlink_metadata().unwrap().file_type().is_symlink(),
        ".env should be a symlink in feat/sync-b worktree"
    );
}

#[test]
fn test_sync_no_targets() {
    let repo = setup_git_repo();
    let path = repo.path();

    // Init ou
    ou_cmd()
        .args(["init"])
        .current_dir(path)
        .assert()
        .success();

    // sync --all with only main worktree (no additional worktrees)
    ou_cmd()
        .args(["sync", "--all"])
        .current_dir(path)
        .assert()
        .success()
        .stdout(predicate::str::contains("No worktrees to sync."));
}

#[test]
fn test_sync_recreates_deleted_symlinks() {
    let repo = setup_git_repo();
    let path = repo.path();

    // Create .env file
    std::fs::write(path.join(".env"), "SECRET=test\n").unwrap();

    // Init ou and add worktree
    ou_cmd()
        .args(["init"])
        .current_dir(path)
        .assert()
        .success();

    ou_cmd()
        .args(["add", "feat/resync"])
        .current_dir(path)
        .assert()
        .success();

    // Locate the worktree directory
    let wt_dir = path
        .join(".git")
        .join("ou-worktrees")
        .join("feat-resync");
    let env_link = wt_dir.join(".env");

    // Verify symlink exists after add
    assert!(
        env_link.symlink_metadata().unwrap().file_type().is_symlink(),
        ".env symlink should exist after add"
    );

    // Delete the symlink manually
    std::fs::remove_file(&env_link).unwrap();
    assert!(!env_link.exists(), ".env should be deleted");

    // Run sync --all to recreate symlinks
    ou_cmd()
        .args(["sync", "--all"])
        .current_dir(path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Synced:"));

    // Verify symlink was recreated
    assert!(
        env_link.symlink_metadata().unwrap().file_type().is_symlink(),
        ".env symlink should be recreated after sync"
    );
}

#[test]
fn test_sync_nonexistent_source() {
    let repo = setup_git_repo();
    let path = repo.path();

    // Init ou
    ou_cmd()
        .args(["init"])
        .current_dir(path)
        .assert()
        .success();

    // sync with nonexistent source
    ou_cmd()
        .args(["sync", "--source", "nonexistent"])
        .current_dir(path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}
