pub mod app;
pub mod pages;
pub mod theme;
pub mod widgets;

pub use app::{App, Page};

use std::io;
use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use netwatch_core::Config;
use netwatch_db::Database;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

