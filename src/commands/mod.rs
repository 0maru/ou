//! Command implementations for the `ou` CLI.
//!
//! Each submodule corresponds to a single CLI subcommand and exposes a
//! `pub fn run()` entry point. Commands receive their dependencies
//! (git runner, filesystem, config) via trait-based dependency injection,
//! making them testable with mock implementations.

pub mod add;
pub mod clean;
pub mod init;
pub mod list;
pub mod open;
pub mod remove;
pub mod sync;
