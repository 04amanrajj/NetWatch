use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Cell, Paragraph, Row, Table};
use ratatui::Frame;

use crate::app::{App, Page};
use crate::theme::Theme;
use crate::widgets::{draw_sparkline, titled_block};
use netwatch_stats::{format_bytes, format_rate};

pub fn draw(frame: &mut Frame, app: &App, theme: &Theme) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(frame.area());

    draw_header(frame, chunks[0], app, theme);

    match app.page {
        Page::Home => draw_home(frame, chunks[1], app, theme),
        Page::Interfaces => draw_interfaces(frame, chunks[1], app, theme),
        Page::InterfaceDetail => draw_interface_detail(frame, chunks[1], app, theme),
        Page::History => draw_history(frame, chunks[1], app, theme),
        Page::Graph => draw_graph(frame, chunks[1], app, theme),
        Page::Live => draw_live(frame, chunks[1], app, theme),
        Page::Search => draw_search(frame, chunks[1], app, theme),
    }

    draw_footer(frame, chunks[2], app, theme);

    if app.show_help {
        draw_help(frame, app, theme);
    }
}

fn draw_header(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let alerts = if app.alert_count > 0 {
        format!(" [{} alerts]", app.alert_count)
    } else {
        String::new()
    };
    let title = format!(" NetWatch{alerts} ");
    frame.render_widget(titled_block(&title, theme), area);
}

fn draw_footer(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let keys = match app.page {
        Page::Home => "i:Interfaces h:History g:Graph l:Live /:Search ?:Help q:Quit",
        Page::Interfaces => "Enter:Detail Esc/q:Back ?:Help",
        Page::InterfaceDetail => "Esc/q:Back ?:Help",
        Page::History => "←/→:Range Tab:Next ?:Help q:Quit",
        Page::Graph => "←/→:Resolution ?:Help q:Quit",
        Page::Live => "q:Quit ?:Help",
        Page::Search => "Type to filter Esc:Back",
    };
    frame.render_widget(
        Paragraph::new(keys).style(Style::default().fg(theme.dim)),
        area,
    );
}

fn draw_home(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let units = app.config.units;
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)

}
