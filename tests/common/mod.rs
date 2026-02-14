use std::process::Command;

use tempfile::TempDir;

pub fn setup_git_repo() -> TempDir {
    let dir = TempDir::new().unwrap();
    let path = dir.path();

    // Try --initial-branch=main first (git 2.28+), fall back to init + branch -M
    let init_output = Command::new("git")
        .args(["init", "--initial-branch=main"])
        .current_dir(path)
        .output()
        .unwrap();

    if !init_output.status.success() {
        Command::new("git")
            .args(["init"])
            .current_dir(path)
            .output()
            .unwrap();
        Command::new("git")
            .args(["branch", "-M", "main"])
            .current_dir(path)
            .output()
            .unwrap();
    }

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

pub fn ou_cmd() -> Command {
    Command::new(assert_cmd::cargo::cargo_bin!("ou"))
}

#[allow(dead_code)]
pub fn git_version() -> (u32, u32, u32) {
    let output = Command::new("git")
        .args(["--version"])
        .output()
        .expect("failed to run git --version");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let version_str = stdout
        .trim()
        .strip_prefix("git version ")
        .expect("unexpected git version format");
    let version_part = version_str
        .split(|c: char| !c.is_ascii_digit() && c != '.')
        .next()
        .unwrap_or("");
    let parts: Vec<u32> = version_part
        .split('.')
        .filter_map(|s| s.parse().ok())
        .collect();
    (
        parts.first().copied().unwrap_or(0),
        parts.get(1).copied().unwrap_or(0),
        parts.get(2).copied().unwrap_or(0),
    )
}

/// Skip test if git version is below the required minimum.
/// Usage: `require_git!(2, 15);`
#[allow(unused_macros)]
macro_rules! require_git {
    ($major:expr, $minor:expr) => {
        let (maj, min, _) = common::git_version();
        if (maj, min) < ($major, $minor) {
            eprintln!(
                "skipping test: git {}.{} required, found {}.{}",
                $major, $minor, maj, min
            );
            return;
        }
    };
}

#[allow(unused_imports)]
pub(crate) use require_git;
