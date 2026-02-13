use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};

pub enum AppEvent {
    Key(KeyEvent),
    Tick,
}

pub fn poll_event(tick_rate: Duration) -> Option<AppEvent> {
    if event::poll(tick_rate).ok()? {
        if let Event::Key(key) = event::read().ok()? {
            return Some(AppEvent::Key(key));
        }
    }
    Some(AppEvent::Tick)
}

pub fn is_quit(key: &KeyEvent) -> bool {
    matches!(key.code, KeyCode::Char('q'))
        || (key.modifiers.contains(KeyModifiers::CONTROL) && matches!(key.code, KeyCode::Char('c')))
}

pub fn is_up(key: &KeyEvent) -> bool {
    matches!(key.code, KeyCode::Up | KeyCode::Char('k'))
}

pub fn is_down(key: &KeyEvent) -> bool {
    matches!(key.code, KeyCode::Down | KeyCode::Char('j'))
}

pub fn is_enter(key: &KeyEvent) -> bool {
    matches!(key.code, KeyCode::Enter)
}

pub fn is_delete(key: &KeyEvent) -> bool {
    matches!(key.code, KeyCode::Char('d'))
}

pub fn is_refresh(key: &KeyEvent) -> bool {
    matches!(key.code, KeyCode::Char('r'))
}
