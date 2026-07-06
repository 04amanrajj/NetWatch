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
        .constraints([Constraint::Length(12), Constraint::Min(0)])
        .split(area);

    let text = format!(
        "Today\n\
         Download      {}\n\
         Upload        {}\n\n\
         Current Speed\n\
         ↓ {}\n\
         ↑ {}\n\n\
         Interfaces Active  {}\n\
         Database Size      {}\n\
         Daemon             {}\n\
         Sampling           {} sec",
        format_bytes(app.totals.download, units),
        format_bytes(app.totals.upload, units),
        format_rate(app.speeds.rx_rate, units),
        format_rate(app.speeds.tx_rate, units),
        app.interfaces.len(),
        format_bytes(app.db_size, units),
        if app.daemon_status.running {
            "Running"
        } else {
            "Stopped"
        },
        app.config.sample_interval,
    );
    frame.render_widget(
        Paragraph::new(text).block(titled_block("Home", theme)),
        chunks[0],
    );

    let rx: Vec<u64> = app.graph_points.iter().map(|p| p.rx_rate).collect();
    draw_sparkline(frame, chunks[1], &rx, "Download (recent)", theme);
}

fn draw_interfaces(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let units = app.config.units;
    let header = Row::new(vec!["Interface", "Download", "Upload", "Status"])
        .style(theme.title_style())
        .bottom_margin(1);

    let rows: Vec<Row> = app
        .filtered_interfaces
        .iter()
        .enumerate()
        .map(|(vis_idx, &idx)| {
            let iface = &app.interfaces[idx];
            let style = if vis_idx == app.selection {
                Style::default().add_modifier(Modifier::REVERSED)
            } else if iface.operstate == "UP" {
                Style::default().fg(theme.up)
            } else {
                Style::default().fg(theme.down)
            };
            Row::new(vec![
                Cell::from(iface.name.clone()),
                Cell::from(format_bytes(iface.download, units)),
                Cell::from(format_bytes(iface.upload, units)),
                Cell::from(iface.operstate.clone()),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(30),
            Constraint::Percentage(25),
            Constraint::Percentage(25),

}
