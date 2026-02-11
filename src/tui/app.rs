use crate::git::executor::GitExecutor;
use crate::git::runner::GitRunner;
use crate::git::types::Worktree;

pub struct App {
    pub worktrees: Vec<Worktree>,
    pub selected: usize,
    #[allow(dead_code)]
    pub should_quit: bool,
    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            worktrees: Vec::new(),
            selected: 0,
            should_quit: false,
            status_message: None,
        }
    }

    pub fn refresh<E: GitExecutor>(&mut self, git: &GitRunner<E>) {
        match git.worktree_list() {
            Ok(wts) => {
                self.worktrees = wts;
                if self.selected >= self.worktrees.len() && !self.worktrees.is_empty() {
                    self.selected = self.worktrees.len() - 1;
                }
                self.status_message = None;
            }
            Err(e) => {
                self.status_message = Some(format!("Error: {e}"));
            }
        }
    }

    pub fn next(&mut self) {
        if !self.worktrees.is_empty() {
            self.selected = (self.selected + 1) % self.worktrees.len();
        }
    }

    pub fn previous(&mut self) {
        if !self.worktrees.is_empty() {
            self.selected = self
                .selected
                .checked_sub(1)
                .unwrap_or(self.worktrees.len() - 1);
        }
    }

    pub fn selected_worktree(&self) -> Option<&Worktree> {
        self.worktrees.get(self.selected)
    }

    pub fn remove_selected<E: GitExecutor>(&mut self, git: &GitRunner<E>) {
        let Some(wt) = self.selected_worktree() else {
            return;
        };

        if wt.is_bare {
            self.status_message = Some("Cannot remove bare worktree".to_string());
            return;
        }

        let branch_name = wt.branch.clone().unwrap_or_default();
        let path = wt.path.clone();

        match git.worktree_remove(&path, false) {
            Ok(()) => {
                let _ = git.branch_delete(&branch_name, false);
                self.status_message = Some(format!("Removed: {branch_name}"));
                self.refresh(git);
            }
            Err(e) => {
                self.status_message = Some(format!("Failed to remove: {e}"));
            }
        }
    }
}
