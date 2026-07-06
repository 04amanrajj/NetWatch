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

pub struct RunOptions {
    pub initial_page: Page,
}

impl Default for RunOptions {
    fn default() -> Self {
        Self {
            initial_page: Page::Home,
        }
    }
}

pub async fn run(config: &Config, db: &Database, options: RunOptions) -> Result<()> {
    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    crossterm::execute!(io::stdout(), crossterm::event::DisableMouseCapture)?;

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let theme = theme::Theme::default();

}
