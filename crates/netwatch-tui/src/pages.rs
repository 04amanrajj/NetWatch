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
            Constraint::Percentage(20),
        ],
    )
    .header(header)
    .block(titled_block("Interfaces", theme));

    frame.render_widget(table, area);
}

fn draw_interface_detail(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let units = app.config.units;
    let Some(detail) = &app.interface_detail else {
        frame.render_widget(
            Paragraph::new("No interface selected").block(titled_block("Interface Detail", theme)),
            area,
        );
        return;
    };

    let lines = vec![
        format!("Interface: {}", detail.name),
        format!(
            "Current Speed: ↓ {}  ↑ {}",
            format_rate(detail.current_rx_rate, units),
            format_rate(detail.current_tx_rate, units)
        ),
        format!(
            "Peak Speed: ↓ {}  ↑ {}",
            format_rate(detail.peak_rx_rate, units),
            format_rate(detail.peak_tx_rate, units)
        ),
        format!(
            "Average Speed: ↓ {}  ↑ {}",
            format_rate(detail.avg_rx_rate, units),
            format_rate(detail.avg_tx_rate, units)
        ),
        String::new(),
        format!(
            "Today: ↓ {}  ↑ {}",
            format_bytes(detail.today_download, units),
            format_bytes(detail.today_upload, units)
        ),
        format!(
            "Yesterday: ↓ {}  ↑ {}",
            format_bytes(detail.yesterday_download, units),
            format_bytes(detail.yesterday_upload, units)
        ),
        format!(
            "This Week: ↓ {}  ↑ {}",
            format_bytes(detail.this_week_download, units),
            format_bytes(detail.this_week_upload, units)
        ),
        format!(
            "Last Week: ↓ {}  ↑ {}",
            format_bytes(detail.last_week_download, units),
            format_bytes(detail.last_week_upload, units)
        ),
        format!(
            "This Month: ↓ {}  ↑ {}",
            format_bytes(detail.this_month_download, units),
            format_bytes(detail.this_month_upload, units)
        ),
        format!(
            "Last Month: ↓ {}  ↑ {}",
            format_bytes(detail.last_month_download, units),
            format_bytes(detail.last_month_upload, units)
        ),
        format!(
            "Year: ↓ {}  ↑ {}",
            format_bytes(detail.this_year_download, units),
            format_bytes(detail.this_year_upload, units)
        ),
        format!(
            "Total Since Installed: ↓ {}  ↑ {}",
            format_bytes(detail.total_download, units),
            format_bytes(detail.total_upload, units)
        ),
    ];

    frame.render_widget(
        Paragraph::new(lines.join("\n")).block(titled_block("Interface Detail", theme)),
        area,
    );
}

fn draw_history(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let units = app.config.units;
    let title = format!("History — {}", app.history_range_label());
    let header = Row::new(vec![
        "Date",
        "Download",
        "Upload",
        "Total",
        "Peak DL",
        "Peak UL",
    ])
    .style(theme.title_style())
    .bottom_margin(1);

    let rows: Vec<Row> = app
        .history
        .iter()
        .enumerate()
        .map(|(idx, entry)| {
            let style = if idx == app.selection {
                Style::default().add_modifier(Modifier::REVERSED)
            } else {
                Style::default().fg(theme.text)
            };
            Row::new(vec![
                Cell::from(entry.date.clone()),
                Cell::from(format_bytes(entry.download, units)),
                Cell::from(format_bytes(entry.upload, units)),
                Cell::from(format_bytes(entry.total, units)),
                Cell::from(format_bytes(entry.peak_download, units)),
                Cell::from(format_bytes(entry.peak_upload, units)),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(20),
            Constraint::Percentage(16),
            Constraint::Percentage(16),
            Constraint::Percentage(16),
            Constraint::Percentage(16),
            Constraint::Percentage(16),
        ],
    )
    .header(header)
    .block(titled_block(&title, theme));

    frame.render_widget(table, area);
}

fn draw_graph(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let title = format!("Graph — {}", app.graph_resolution_label());
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .margin(1)
        .split(area);

    let block = titled_block(&title, theme);
    frame.render_widget(block, area);

    let inner = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(Rect {
            x: area.x + 1,
            y: area.y + 1,
            width: area.width.saturating_sub(2),
            height: area.height.saturating_sub(2),
        });

    let rx: Vec<u64> = app.graph_points.iter().map(|p| p.rx_rate).collect();
    let tx: Vec<u64> = app.graph_points.iter().map(|p| p.tx_rate).collect();
    draw_sparkline(frame, inner[0], &rx, "Download", theme);
    draw_sparkline(frame, inner[1], &tx, "Upload", theme);

    let _ = chunks;
}

fn draw_live(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let units = app.config.units;
    let header = Row::new(vec![
        "Interface",
        "Download",
        "Upload",
        "Peak",
        "Average",
        "Status",
    ])
    .style(theme.title_style())
    .bottom_margin(1);

    let rows: Vec<Row> = app
        .interfaces
        .iter()
        .map(|iface| {
            let peak = format_rate(
                iface.rx_rate.max(iface.tx_rate),
                units,
            );
            let avg = format_rate(
                (iface.rx_rate + iface.tx_rate) / 2,
                units,
            );
            Row::new(vec![
                Cell::from(iface.name.clone()),
                Cell::from(format_rate(iface.rx_rate, units)),
                Cell::from(format_rate(iface.tx_rate, units)),
                Cell::from(peak),
                Cell::from(avg),
                Cell::from(iface.operstate.clone()),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(22),
            Constraint::Percentage(16),
            Constraint::Percentage(16),
            Constraint::Percentage(16),
            Constraint::Percentage(16),
            Constraint::Percentage(14),
        ],
    )
    .header(header)
    .block(titled_block("Live Monitor", theme));

    frame.render_widget(table, area);
}

fn draw_search(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let prompt = format!("Search: {}_", app.search_query);
    frame.render_widget(
        Paragraph::new(prompt).block(titled_block("Search", theme)),
        Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: 3,
        },
    );
    draw_interfaces(
        frame,
        Rect {
            x: area.x,
            y: area.y + 3,
            width: area.width,
            height: area.height.saturating_sub(3),
        },
        app,
        theme,
    );
}

fn draw_help(frame: &mut Frame, _app: &App, theme: &Theme) {
    let area = centered_rect(60, 40, frame.area());
    let text = "\
NetWatch — keyboard shortcuts\n\n\
q/Esc   Quit or go back\n\
i       Interfaces\n\
h       History\n\
g       Graph\n\
l       Live monitor\n\
/       Search\n\
Enter   Open interface detail\n\
↑/↓     Navigate lists\n\
←/→     Change range/resolution\n\
?       Toggle this help\n";
    frame.render_widget(
        Paragraph::new(text).block(titled_block("Help", theme)),
        area,
    );
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
