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

    let tick_rate = Duration::from_millis(1000);
    let mut last_tick = std::time::Instant::now();

    // Initial database fetch before starting the loop
    app.refresh(db).await?;

    loop {
        terminal.draw(|frame| {
            pages::draw(frame, &app, &theme);
        })?;

        let mut should_refresh = false;
        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    if app.page == Page::Settings {
                        match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => {
                                app.page = app.previous_page;
                            }
                            KeyCode::Up => {
                                app.move_settings_selection(-1);
                            }
                            KeyCode::Down => {
                                app.move_settings_selection(1);
                            }
                            KeyCode::Left => {
                                app.adjust_setting(-1);
                            }
                            KeyCode::Right => {
                                app.adjust_setting(1);
                            }
                            KeyCode::Enter => {
                                if app.handle_settings_enter()? {
                                    should_refresh = true;
                                }
                            }
                            _ => {}
                        }
                    } else {
                        match key.code {
                            KeyCode::Char('q') | KeyCode::Esc if app.page != Page::Search => {
                                app.should_quit = true;
                            }
                            KeyCode::Char('?') => app.show_help = !app.show_help,
                            KeyCode::Char('s') => {
                                app.enter_settings();
                            }
                            KeyCode::Char('i') => {
                                app.page = Page::Interfaces;
                                should_refresh = true;
                            }
                            KeyCode::Char('h') => {
                                app.page = Page::History;
                                should_refresh = true;
                            }
                            KeyCode::Char('g') => {
                                app.page = Page::Graph;
                                should_refresh = true;
                            }
                            KeyCode::Char('l') => {
                                app.page = Page::Live;
                                should_refresh = true;
                            }
                            KeyCode::Char('/') => {
                                app.page = Page::Search;
                                app.search_query.clear();
                            }
                            KeyCode::Char('1') => {
                                app.page = Page::Home;
                                should_refresh = true;
                            }
                            KeyCode::Enter => {
                                app.handle_enter();
                                should_refresh = true;
                            }
                            KeyCode::Up => app.move_selection(-1),
                            KeyCode::Down => app.move_selection(1),
                            KeyCode::Left => {
                                app.adjust_range(-1);
                                should_refresh = true;
                            }
                            KeyCode::Right => {
                                app.adjust_range(1);
                                should_refresh = true;
                            }
                            KeyCode::Char(c) if app.page == Page::Search => {
                                app.search_query.push(c);
                                app.apply_search();
                            }
                            KeyCode::Backspace if app.page == Page::Search => {
                                app.search_query.pop();
                                app.apply_search();
                            }
                            KeyCode::Tab => {
                                app.next_history_range();
                                should_refresh = true;
                            }
                            _ => {}
                        }
                    }
                    if key.modifiers.contains(KeyModifiers::CONTROL)
                        && key.code == KeyCode::Char('c')
                    {
                        app.should_quit = true;
                    }
                }
            }
        }

        // Refresh database stats only on timer tick or if user requested a page/range change
        if last_tick.elapsed() >= tick_rate || should_refresh {
            app.refresh(db).await?;
            if last_tick.elapsed() >= tick_rate {
                last_tick = std::time::Instant::now();
            }
        }

        if app.should_quit {
            break;
        }
    }

    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
