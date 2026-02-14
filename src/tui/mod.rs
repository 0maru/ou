pub mod app;
pub mod event;
pub mod ui;

use std::io;
use std::time::Duration;

use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

use crate::error::OuError;
use crate::git::executor::GitExecutor;
use crate::git::runner::GitRunner;
use crate::multiplexer;

use self::app::App;

pub fn run_dashboard<E: GitExecutor>(git: &GitRunner<E>) -> Result<(), OuError> {
    enable_raw_mode().map_err(OuError::Io)?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).map_err(OuError::Io)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).map_err(OuError::Io)?;

    let mut app = App::new();
    app.refresh(git);

    let tick_rate = Duration::from_millis(250);

    loop {
        terminal.draw(|f| ui::draw(f, &app)).map_err(OuError::Io)?;

        if let Some(evt) = event::poll_event(tick_rate) {
            match evt {
                event::AppEvent::Key(key) => {
                    if event::is_quit(&key) {
                        break;
                    } else if event::is_down(&key) {
                        app.next();
                    } else if event::is_up(&key) {
                        app.previous();
                    } else if event::is_refresh(&key) {
                        app.refresh(git);
                    } else if event::is_delete(&key) {
                        app.remove_selected(git);
                    } else if event::is_enter(&key)
                        && let Some(wt) = app.selected_worktree()
                    {
                        let path = wt.path.clone();
                        let branch = wt.branch.clone().unwrap_or_default();
                        if let Some(mux) = multiplexer::detect_multiplexer() {
                            match mux.open_tab(&path, Some(&branch)) {
                                Ok(_) => {
                                    app.status_message =
                                        Some(format!("Opened {branch} in {}", mux.name()));
                                }
                                Err(e) => {
                                    app.status_message = Some(format!("Failed to open tab: {e}"));
                                }
                            }
                        } else {
                            app.status_message =
                                Some(format!("No multiplexer detected. Path: {}", path.display()));
                        }
                    }
                }
                event::AppEvent::Tick => {}
            }
        }
    }

    disable_raw_mode().map_err(OuError::Io)?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen).map_err(OuError::Io)?;
    terminal.show_cursor().map_err(OuError::Io)?;

    Ok(())
}
