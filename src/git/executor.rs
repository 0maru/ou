use std::process::Command;

use crate::error::OuError;
use crate::git::types::CommandOutput;

pub trait GitExecutor: Send + Sync {
    fn run(&self, args: &[&str]) -> Result<CommandOutput, OuError>;
}

pub struct OsGitExecutor;

impl GitExecutor for OsGitExecutor {
    fn run(&self, args: &[&str]) -> Result<CommandOutput, OuError> {
        let output = Command::new("git")
            .args(args)
            .output()
            .map_err(|e| OuError::Git(format!("failed to execute git: {e}")))?;

        Ok(CommandOutput {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            status: output.status.code().unwrap_or(-1),
        })
    }
}
