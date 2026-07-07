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
    let mut app = App::new(config.clone(), options.initial_page);

    let tick_rate = Duration::from_secs(1);
    let mut last_tick = std::time::Instant::now();

    loop {
        app.refresh(db).await?;

        terminal.draw(|frame| {
            pages::draw(frame, &app, &theme);
        })?;

        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc if app.page != Page::Search => {
                            app.should_quit = true;
                        }
                        KeyCode::Char('?') => app.show_help = !app.show_help,
                        KeyCode::Char('i') => app.page = Page::Interfaces,

}}}}}}
