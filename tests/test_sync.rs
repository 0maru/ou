mod common;

use assert_cmd::prelude::*;
use predicates::prelude::*;

use common::{ou_cmd, setup_git_repo};

#[test]
fn test_sync_all_syncs_worktrees() {
    let repo = setup_git_repo();
    let path = repo.path();

    std::fs::write(path.join(".env"), "SECRET=test\n").unwrap();

    ou_cmd().args(["init"]).current_dir(path).assert().success();

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

    ou_cmd()
        .args(["sync", "--all"])
        .current_dir(path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Synced:"));

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

    ou_cmd().args(["init"]).current_dir(path).assert().success();

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

    std::fs::write(path.join(".env"), "SECRET=test\n").unwrap();

    ou_cmd().args(["init"]).current_dir(path).assert().success();

    ou_cmd()
        .args(["add", "feat/resync"])
        .current_dir(path)
        .assert()
        .success();

    let wt_dir = path
        .join(".git")
        .join("ou-worktrees")
        .join("feat-resync");
    let env_link = wt_dir.join(".env");

    assert!(
        env_link
            .symlink_metadata()
            .unwrap()
            .file_type()
            .is_symlink(),
        ".env symlink should exist after add"
    );

    std::fs::remove_file(&env_link).unwrap();
    assert!(!env_link.exists(), ".env should be deleted");

    ou_cmd()
        .args(["sync", "--all"])
        .current_dir(path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Synced:"));

    assert!(
        env_link
            .symlink_metadata()
            .unwrap()
            .file_type()
            .is_symlink(),
        ".env symlink should be recreated after sync"
    );
}

#[test]
fn test_sync_nonexistent_source() {
    let repo = setup_git_repo();
    let path = repo.path();

    ou_cmd().args(["init"]).current_dir(path).assert().success();

    ou_cmd()
        .args(["sync", "--source", "nonexistent"])
        .current_dir(path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}
