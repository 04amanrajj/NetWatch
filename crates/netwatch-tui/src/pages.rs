use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table};
use ratatui::Frame;

use crate::app::{App, Page};
use crate::theme::Theme;
use crate::widgets::titled_block;
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
        Page::Settings => draw_settings(frame, chunks[1], app, theme),
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

    let left_text = format!(" NetWatch{alerts}");

    let daemon_status_text = if app.daemon_status.running {
        "Daemon ● Running"
    } else {
        "Daemon ● Stopped"
    };
    let daemon_color = if app.daemon_status.running {
        theme.up
    } else {
        theme.down
    };

    let now = chrono::Local::now().format("%d %b %H:%M:%S").to_string();

    let mut spans = vec![
        Span::styled(left_text, Style::default().fg(theme.title).add_modifier(Modifier::BOLD)),
    ];

    let right_text_len = daemon_status_text.len() + now.len() + 8;
    let total_width = area.width as usize;
    let spaces = total_width.saturating_sub(12 + alerts.len() + right_text_len).saturating_sub(4);

    spans.push(Span::raw(" ".repeat(spaces)));
    spans.push(Span::raw("Daemon "));
    spans.push(Span::styled("●", Style::default().fg(daemon_color)));
    spans.push(Span::raw(if app.daemon_status.running { " Running" } else { " Stopped" }));
    spans.push(Span::styled("  │  ", Style::default().fg(theme.dim)));
    spans.push(Span::styled(now, Style::default().fg(theme.title)));

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.dim));

    let paragraph = Paragraph::new(Line::from(spans)).block(block);
    frame.render_widget(paragraph, area);
}

fn draw_footer(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let keys = match app.page {
        Page::Home => " F1:Help │ Tab:Switch │ Enter:Details │ /:Search │ q:Quit ",
        Page::Interfaces => " F1:Help │ Tab:Switch │ Enter:Detail │ Esc:Back │ q:Quit ",
        Page::InterfaceDetail => " F1:Help │ Esc/q:Back ",
        Page::History => " F1:Help │ Tab:Switch │ ←/→:Range │ q:Quit ",
        Page::Graph => " F1:Help │ Tab:Switch │ ←/→:Resolution │ q:Quit ",
        Page::Live => " F1:Help │ Tab:Switch │ q:Quit ",
        Page::Search => " Type to filter │ Esc:Back ",
        Page::Settings => " ↑/↓:Navigate │ ←/→:Adjust │ Enter:Save/Cancel │ Esc:Back ",
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
        .constraints([
            Constraint::Length(4), // Top row: cards
            Constraint::Min(0),    // Bottom row: graph and top list
        ])
        .split(area);

    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(34),
            Constraint::Percentage(33),
        ])
        .split(chunks[0]);

    // Widget 1: Today
    let today_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.dim))
        .title(Span::styled(" Today ", Style::default().fg(theme.title).add_modifier(Modifier::BOLD)));
        
    let today_text = vec![
        Line::from(vec![
            Span::styled(" ↓ ", Style::default().fg(theme.up)),
            Span::styled(format_bytes(app.totals.download, units), Style::default().fg(theme.text).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled(" ↑ ", Style::default().fg(theme.accent)),
            Span::styled(format_bytes(app.totals.upload, units), Style::default().fg(theme.text).add_modifier(Modifier::BOLD)),
        ]),
    ];
    frame.render_widget(Paragraph::new(today_text).block(today_block), top_chunks[0]);

    // Widget 2: Current Speed
    let speed_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.dim))
        .title(Span::styled(" Current Speed ", Style::default().fg(theme.title).add_modifier(Modifier::BOLD)));
        
    let speed_text = vec![
        Line::from(vec![
            Span::styled(" ↓ ", Style::default().fg(theme.up)),
            Span::styled(format_rate(app.speeds.rx_rate, units), Style::default().fg(theme.text).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled(" ↑ ", Style::default().fg(theme.accent)),
            Span::styled(format_rate(app.speeds.tx_rate, units), Style::default().fg(theme.text).add_modifier(Modifier::BOLD)),
        ]),
    ];
    frame.render_widget(Paragraph::new(speed_text).block(speed_block), top_chunks[1]);

    // Widget 3: Interfaces
    let mut up_count = 0;
    let mut down_count = 0;
    for iface in &app.interfaces {
        if iface.operstate == "UP" {
            up_count += 1;
        } else {
            down_count += 1;
        }
    }
    
    let iface_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.dim))
        .title(Span::styled(" Interfaces ", Style::default().fg(theme.title).add_modifier(Modifier::BOLD)));
        
    let iface_text = vec![
        Line::from(vec![
            Span::raw(" Active: "),
            Span::styled(app.interfaces.len().to_string(), Style::default().fg(theme.title).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::raw(" Up: "),
            Span::styled(up_count.to_string(), Style::default().fg(theme.up).add_modifier(Modifier::BOLD)),
            Span::raw("   Down: "),
            Span::styled(down_count.to_string(), Style::default().fg(theme.down).add_modifier(Modifier::BOLD)),
        ]),
    ];
    frame.render_widget(Paragraph::new(iface_text).block(iface_block), top_chunks[2]);

    // Bottom row: Graph & Top Interfaces side-by-side
    let bottom_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(70),
            Constraint::Percentage(30),
        ])
        .split(chunks[1]);

    // Widget 4: Graph
    let rx: Vec<u64> = app.graph_points.iter().map(|p| p.rx_rate).collect();
    let graph_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.dim))
        .title(Span::styled(" Download Graph ", Style::default().fg(theme.title).add_modifier(Modifier::BOLD)));
        
    let sparkline = ratatui::widgets::Sparkline::default()
        .block(graph_block)
        .data(&rx)
        .style(Style::default().fg(theme.up));
    frame.render_widget(sparkline, bottom_chunks[0]);

    // Widget 5: Top Interfaces
    let mut sorted_ifaces = app.interfaces.clone();
    sorted_ifaces.sort_by(|a, b| b.download.cmp(&a.download));
    
    let top_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.dim))
        .title(Span::styled(" Top Interfaces ", Style::default().fg(theme.title).add_modifier(Modifier::BOLD)));
        
    let mut top_lines = Vec::new();
    for iface in sorted_ifaces.iter().take(5) {
        let name = &iface.name;
        let traffic = format_bytes(iface.download, units);
        let name_span = Span::styled(format!("  {:<10}  ", name), Style::default().fg(theme.text));
        let traffic_span = Span::styled(traffic, Style::default().fg(theme.accent).add_modifier(Modifier::BOLD));
        top_lines.push(Line::from(vec![name_span, traffic_span]));
    }
    frame.render_widget(Paragraph::new(top_lines).block(top_block), bottom_chunks[1]);
}

fn draw_interfaces(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let units = app.config.units;
    let header = Row::new(vec!["  Name", "↓ Today", "↑ Today", "Speed", "Status"])
        .style(theme.title_style())
        .bottom_margin(1);

    let rows: Vec<Row> = app
        .filtered_interfaces
        .iter()
        .enumerate()
        .map(|(vis_idx, &idx)| {
            let iface = &app.interfaces[idx];
            let is_selected = vis_idx == app.selection;
            let cursor = if is_selected { "▶ " } else { "  " };

            let status_dot = if iface.operstate == "UP" {
                Span::styled("●", Style::default().fg(theme.up))
            } else {
                Span::styled("○", Style::default().fg(theme.down))
            };

            let row_style = if is_selected {
                Style::default().fg(theme.border).add_modifier(Modifier::BOLD)
            } else if vis_idx % 2 == 1 {
                Style::default().bg(Color::Rgb(25, 25, 25))
            } else {
                Style::default().fg(theme.text)
            };

            let speed = format_rate(iface.rx_rate, units);

            Row::new(vec![
                Cell::from(format!("{}{}", cursor, iface.name)),
                Cell::from(format_bytes(iface.download, units)),
                Cell::from(format_bytes(iface.upload, units)),
                Cell::from(speed),
                Cell::from(status_dot),
            ])
            .style(row_style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(25),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(15),
        ],
    )
    .header(header)
    .block(Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.dim))
        .title(Span::styled(" Interfaces ", theme.title_style()))
    );

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
    let title = format!(" History — {} ", app.history_range_label());
    let header = Row::new(vec![
        "  Date", "Download", "Upload", "Total", "Peak DL", "Peak UL",
    ])
    .style(theme.title_style())
    .bottom_margin(1);

    let rows: Vec<Row> = app
        .history
        .iter()
        .enumerate()
        .map(|(idx, entry)| {
            let is_selected = idx == app.selection;
            let cursor = if is_selected { "▶ " } else { "  " };

            let row_style = if is_selected {
                Style::default().fg(theme.border).add_modifier(Modifier::BOLD)
            } else if idx % 2 == 1 {
                Style::default().bg(Color::Rgb(25, 25, 25))
            } else {
                Style::default().fg(theme.text)
            };

            Row::new(vec![
                Cell::from(format!("{}{}", cursor, entry.date)),
                Cell::from(format_bytes(entry.download, units)),
                Cell::from(format_bytes(entry.upload, units)),
                Cell::from(format_bytes(entry.total, units)),
                Cell::from(format_bytes(entry.peak_download, units)),
                Cell::from(format_bytes(entry.peak_upload, units)),
            ])
            .style(row_style)
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
    .block(Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.dim))
        .title(Span::styled(title, theme.title_style()))
    );

    frame.render_widget(table, area);
}

fn draw_graph(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let title = format!(" Graph — {} ", app.graph_resolution_label());
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(area);

    let rx: Vec<u64> = app.graph_points.iter().map(|p| p.rx_rate).collect();
    let tx: Vec<u64> = app.graph_points.iter().map(|p| p.tx_rate).collect();

    let dl_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.dim))
        .title(Span::styled(format!(" Download ({}) ", title), Style::default().fg(theme.title).add_modifier(Modifier::BOLD)));
        
    let dl_sparkline = ratatui::widgets::Sparkline::default()
        .block(dl_block)
        .data(&rx)
        .style(Style::default().fg(theme.up));

    let ul_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.dim))
        .title(Span::styled(" Upload ", Style::default().fg(theme.title).add_modifier(Modifier::BOLD)));
        
    let ul_sparkline = ratatui::widgets::Sparkline::default()
        .block(ul_block)
        .data(&tx)
        .style(Style::default().fg(theme.accent));

    frame.render_widget(dl_sparkline, chunks[0]);
    frame.render_widget(ul_sparkline, chunks[1]);
}

fn draw_live(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let units = app.config.units;
    let header = Row::new(vec![
        "  Interface",
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
        .enumerate()
        .map(|(idx, iface)| {
            let is_selected = idx == app.selection;
            let cursor = if is_selected { "▶ " } else { "  " };

            let status_dot = if iface.operstate == "UP" {
                Span::styled("●", Style::default().fg(theme.up))
            } else {
                Span::styled("○", Style::default().fg(theme.down))
            };

            let row_style = if is_selected {
                Style::default().fg(theme.border).add_modifier(Modifier::BOLD)
            } else if idx % 2 == 1 {
                Style::default().bg(Color::Rgb(25, 25, 25))
            } else {
                Style::default().fg(theme.text)
            };

            let peak = format_rate(iface.rx_rate.max(iface.tx_rate), units);
            let avg = format_rate((iface.rx_rate + iface.tx_rate) / 2, units);

            Row::new(vec![
                Cell::from(format!("{}{}", cursor, iface.name)),
                Cell::from(format_rate(iface.rx_rate, units)),
                Cell::from(format_rate(iface.tx_rate, units)),
                Cell::from(peak),
                Cell::from(avg),
                Cell::from(status_dot),
            ])
            .style(row_style)
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
    .block(Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.dim))
        .title(Span::styled(" Live Monitor ", theme.title_style()))
    );

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
    let area = centered_rect(60, 42, frame.area());
    let text = "\
NetWatch — keyboard shortcuts\n\n\
q/Esc   Quit or go back\n\
i       Interfaces\n\
h       History\n\
g       Graph\n\
l       Live monitor\n\
s       Settings\n\
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

fn draw_settings(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(14),
            Constraint::Min(0),
        ])
        .split(area);

    let mut lines = Vec::new();
    
    let options = [
        ("Display Units", format!("{:?}", app.temp_config.units)),
        ("Ignore Loopback Interface", if app.temp_config.skip_loopback { "Yes".to_string() } else { "No".to_string() }),
        ("Sample Interval (Seconds)", format!("{}s", app.temp_config.sample_interval)),
        ("Database Write Batching (Seconds)", format!("{}s", app.temp_config.batch_write_interval)),
        ("Data Retention (Days)", format!("{} days", app.temp_config.history_days)),
        ("Save & Apply Settings", "".to_string()),
        ("Cancel & Discard Changes", "".to_string()),
    ];

    for (idx, (label, val)) in options.iter().enumerate() {
        let is_selected = idx == app.settings_selection;
        let prefix = if is_selected { " > " } else { "   " };
        let style = if is_selected {
            Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.text)
        };
        
        let line = if val.is_empty() {
            format!("{prefix}{label}")
        } else {
            format!("{prefix}{:<35} : {}", label, val)
        };
        
        lines.push(ratatui::text::Line::styled(line, style));
    }

    lines.push(ratatui::text::Line::from(""));
    lines.push(ratatui::text::Line::styled(
        " Note: Setting changes to intervals will require restarting the background daemon (netwatchd) to take full effect.",
        Style::default().fg(theme.dim),
    ));

    frame.render_widget(
        Paragraph::new(lines).block(titled_block("Settings", theme)),
        chunks[0],
    );
}
