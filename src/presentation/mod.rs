// Ratatui TUI layer and event handling
// Example: ui logic, crossterm events, ratatui widgets

pub mod app;
pub mod ui;
pub mod views;
pub mod ascii_art;

pub use ui::run_tui;
