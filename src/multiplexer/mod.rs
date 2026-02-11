pub mod wezterm;

use std::path::Path;

use crate::error::OuError;

#[derive(Debug)]
pub struct TabInfo {
    pub id: String,
    pub title: String,
    pub cwd: Option<String>,
}

pub trait Multiplexer: Send + Sync {
    fn name(&self) -> &'static str;
    fn is_available(&self) -> bool;
    fn open_tab(&self, cwd: &Path, title: Option<&str>) -> Result<String, OuError>;
    fn list_tabs(&self) -> Result<Vec<TabInfo>, OuError>;
    fn activate_tab(&self, tab_id: &str) -> Result<(), OuError>;
    fn close_tab(&self, tab_id: &str) -> Result<(), OuError>;
}

pub fn detect_multiplexer() -> Option<Box<dyn Multiplexer>> {
    let wez = wezterm::WeztermMultiplexer;
    if wez.is_available() {
        return Some(Box::new(wez));
    }
    None
}
