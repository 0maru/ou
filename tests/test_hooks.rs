mod common;

use assert_cmd::prelude::*;
use predicates::prelude::*;

use common::{ou_cmd, setup_git_repo};

#[test]
fn test_post_add_hook_creates_marker_file() {
    let repo = setup_git_repo();
    let path = repo.path();

    ou_cmd().args(["init"]).current_dir(path).assert().success();

    // Write settings with a hook that creates a marker file
    let settings = r#"
default_source = "main"

[hooks]
post_add = ["touch {worktree_path}/hook-marker"]
"#;
    std::fs::write(path.join(".ou/settings.toml"), settings).unwrap();

    ou_cmd()
        .args(["add", "feat/hook-test"])
        .current_dir(path)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Created worktree 'feat/hook-test'",
        ));

    // Verify the marker file was created by the hook
    let git_dir = path.join(".git/ou-worktrees/feat-hook-test");
    assert!(
        git_dir.join("hook-marker").exists(),
        "hook should have created marker file"
    );
}

#[test]
fn test_post_add_hook_template_variables() {
    let repo = setup_git_repo();
    let path = repo.path();

    ou_cmd().args(["init"]).current_dir(path).assert().success();

    // Hook that writes template variables to a file for inspection
    let settings = r#"
default_source = "main"

[hooks]
post_add = ["echo branch={branch_name} wt_name={worktree_name} source={source_branch} > {worktree_path}/vars.txt"]
"#;
    std::fs::write(path.join(".ou/settings.toml"), settings).unwrap();

    ou_cmd()
        .args(["add", "feat/vars-test", "--source", "main"])
        .current_dir(path)
        .assert()
        .success();

    let wt_dir = path.join(".git/ou-worktrees/feat-vars-test");
    let content = std::fs::read_to_string(wt_dir.join("vars.txt")).unwrap();
    assert!(
        content.contains("branch=feat/vars-test"),
        "should contain branch_name, got: {content}"
    );
    assert!(
        content.contains("wt_name=feat-vars-test"),
        "should contain worktree_name, got: {content}"
    );
    assert!(
        content.contains("source=main"),
        "should contain source_branch, got: {content}"
    );
}

#[test]
fn test_post_add_hook_failure_does_not_fail_add() {
    let repo = setup_git_repo();
    let path = repo.path();

    ou_cmd().args(["init"]).current_dir(path).assert().success();

    // Hook with a command that will fail
    let settings = r#"
default_source = "main"

[hooks]
post_add = ["false"]
"#;
    std::fs::write(path.join(".ou/settings.toml"), settings).unwrap();

    ou_cmd()
        .args(["add", "feat/fail-hook"])
        .current_dir(path)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Created worktree 'feat/fail-hook'",
        ))
        .stdout(predicate::str::contains("hook warning"));
}

#[test]
fn test_no_hooks_section_works() {
    let repo = setup_git_repo();
    let path = repo.path();

    ou_cmd().args(["init"]).current_dir(path).assert().success();

    // Settings without hooks section
    let settings = r#"
default_source = "main"
"#;
    std::fs::write(path.join(".ou/settings.toml"), settings).unwrap();

    ou_cmd()
        .args(["add", "feat/no-hooks", "--source", "main"])
        .current_dir(path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Created worktree 'feat/no-hooks'"));
}

#[test]
fn test_post_add_hooks_run_sequentially() {
    let repo = setup_git_repo();
    let path = repo.path();

    ou_cmd().args(["init"]).current_dir(path).assert().success();

    // Two hooks: first creates dir, second creates file inside it
    let settings = r#"
default_source = "main"

[hooks]
post_add = [
    "mkdir -p {worktree_path}/hook-dir",
    "touch {worktree_path}/hook-dir/nested-marker",
]
"#;
    std::fs::write(path.join(".ou/settings.toml"), settings).unwrap();

    ou_cmd()
        .args(["add", "feat/seq-test"])
        .current_dir(path)
        .assert()
        .success();

    let wt_dir = path.join(".git/ou-worktrees/feat-seq-test");
    assert!(
        wt_dir.join("hook-dir/nested-marker").exists(),
        "sequential hooks should create nested structure"
    );
}
