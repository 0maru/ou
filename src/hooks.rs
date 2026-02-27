use std::collections::HashMap;
use std::process::Command;

#[derive(Debug)]
pub struct HookContext {
    vars: HashMap<String, String>,
}

impl HookContext {
    pub fn new() -> Self {
        Self {
            vars: HashMap::new(),
        }
    }

    pub fn set(mut self, key: &str, value: &str) -> Self {
        self.vars.insert(key.to_string(), value.to_string());
        self
    }

    pub fn render(&self, template: &str) -> String {
        let mut result = template.to_string();
        for (key, value) in &self.vars {
            result = result.replace(&format!("{{{key}}}"), value);
        }
        result
    }
}

/// Run hook commands sequentially. Returns a list of warning messages for failed commands.
pub fn run_hooks(commands: &[String], ctx: &HookContext) -> Vec<String> {
    let mut warnings = Vec::new();
    let total = commands.len();
    for (i, cmd) in commands.iter().enumerate() {
        let rendered = ctx.render(cmd);
        if total == 1 {
            eprintln!("Running hook: {rendered}");
        } else {
            eprintln!("Running hook [{}/{}]: {rendered}", i + 1, total);
        }
        match Command::new("sh").arg("-c").arg(&rendered).status() {
            Ok(status) if status.success() => {}
            Ok(status) => {
                let msg = format!(
                    "hook command exited with {}: {rendered}",
                    status.code().unwrap_or(-1)
                );
                eprintln!("Warning: {msg}");
                warnings.push(msg);
            }
            Err(e) => {
                let msg = format!("hook command failed to execute: {rendered}: {e}");
                eprintln!("Warning: {msg}");
                warnings.push(msg);
            }
        }
    }
    warnings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_basic() {
        let ctx = HookContext::new()
            .set("worktree_path", "/repo/.git/ou-worktrees/feat-login")
            .set("branch_name", "feat/login");
        assert_eq!(
            ctx.render("echo {worktree_path}"),
            "echo /repo/.git/ou-worktrees/feat-login"
        );
        assert_eq!(
            ctx.render("git checkout {branch_name}"),
            "git checkout feat/login"
        );
    }

    #[test]
    fn test_render_multiple_vars_in_one_template() {
        let ctx = HookContext::new()
            .set("pane_id", "42")
            .set("worktree_path", "/wt");
        assert_eq!(
            ctx.render("wezterm cli split-pane --pane-id {pane_id} --cwd {worktree_path}"),
            "wezterm cli split-pane --pane-id 42 --cwd /wt"
        );
    }

    #[test]
    fn test_render_no_vars() {
        let ctx = HookContext::new();
        assert_eq!(ctx.render("echo hello"), "echo hello");
    }

    #[test]
    fn test_render_unknown_var_left_as_is() {
        let ctx = HookContext::new().set("a", "1");
        assert_eq!(ctx.render("{a} {b}"), "1 {b}");
    }

    #[test]
    fn test_run_hooks_success() {
        let ctx = HookContext::new().set("msg", "hello");
        let warnings = run_hooks(&["echo {msg}".to_string()], &ctx);
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_run_hooks_failure_returns_warning() {
        let ctx = HookContext::new();
        let warnings = run_hooks(&["false".to_string()], &ctx);
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("exited with"));
    }

    #[test]
    fn test_run_hooks_partial_failure() {
        let ctx = HookContext::new();
        let warnings = run_hooks(
            &["true".to_string(), "false".to_string(), "true".to_string()],
            &ctx,
        );
        assert_eq!(warnings.len(), 1);
    }

    #[test]
    fn test_run_hooks_empty() {
        let ctx = HookContext::new();
        let warnings = run_hooks(&[], &ctx);
        assert!(warnings.is_empty());
    }
}
