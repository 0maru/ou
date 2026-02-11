use std::path::Path;
use std::process::Command;

use crate::error::OuError;
use crate::multiplexer::{Multiplexer, TabInfo};

pub struct WeztermMultiplexer;

impl Multiplexer for WeztermMultiplexer {
    fn name(&self) -> &'static str {
        "WezTerm"
    }

    fn is_available(&self) -> bool {
        std::env::var("WEZTERM_PANE").is_ok()
    }

    fn open_tab(&self, cwd: &Path, title: Option<&str>) -> Result<String, OuError> {
        let cwd_str = cwd.to_string_lossy();
        let args = vec!["cli", "spawn", "--cwd", &cwd_str];

        let output = Command::new("wezterm")
            .args(&args)
            .output()
            .map_err(|e| OuError::Multiplexer(format!("failed to run wezterm cli: {e}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(OuError::Multiplexer(format!(
                "wezterm cli spawn failed: {stderr}"
            )));
        }

        let pane_id = String::from_utf8_lossy(&output.stdout).trim().to_string();

        if let Some(title) = title {
            let _ = Command::new("wezterm")
                .args(["cli", "set-tab-title", "--pane-id", &pane_id, title])
                .output();
        }

        Ok(pane_id)
    }

    fn list_tabs(&self) -> Result<Vec<TabInfo>, OuError> {
        let output = Command::new("wezterm")
            .args(["cli", "list", "--format", "json"])
            .output()
            .map_err(|e| OuError::Multiplexer(format!("failed to run wezterm cli: {e}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(OuError::Multiplexer(format!(
                "wezterm cli list failed: {stderr}"
            )));
        }

        let json: Vec<serde_json::Value> =
            serde_json::from_slice(&output.stdout).map_err(|e| {
                OuError::Multiplexer(format!("failed to parse wezterm output: {e}"))
            })?;

        let tabs = json
            .into_iter()
            .map(|v| TabInfo {
                id: v["pane_id"].as_u64().unwrap_or(0).to_string(),
                title: v["title"].as_str().unwrap_or("").to_string(),
                cwd: v["cwd"].as_str().map(|s| s.to_string()),
            })
            .collect();

        Ok(tabs)
    }

    fn activate_tab(&self, tab_id: &str) -> Result<(), OuError> {
        let output = Command::new("wezterm")
            .args(["cli", "activate-pane", "--pane-id", tab_id])
            .output()
            .map_err(|e| OuError::Multiplexer(format!("failed to run wezterm cli: {e}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(OuError::Multiplexer(format!(
                "wezterm cli activate failed: {stderr}"
            )));
        }

        Ok(())
    }

    fn close_tab(&self, tab_id: &str) -> Result<(), OuError> {
        let output = Command::new("wezterm")
            .args(["cli", "kill-pane", "--pane-id", tab_id])
            .output()
            .map_err(|e| OuError::Multiplexer(format!("failed to run wezterm cli: {e}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(OuError::Multiplexer(format!(
                "wezterm cli kill failed: {stderr}"
            )));
        }

        Ok(())
    }
}
