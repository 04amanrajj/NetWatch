use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::theme::Theme;

pub fn titled_block<'a>(title: &'a str, theme: &Theme) -> Block<'a> {
    Block::default()
        .borders(Borders::ALL)
        .border_style(theme.border_style())
        .title(title)
        .title_style(theme.title_style())
}

pub fn stat_line(label: &str, value: &str, theme: &Theme) -> Paragraph<'static> {
    let text = format!("{label:<16} {value}");
    Paragraph::new(text).style(Style::default().fg(theme.text))
}

pub fn draw_sparkline(
    frame: &mut Frame,
    area: Rect,
    values: &[u64],
    title: &str,
    theme: &Theme,
) {
    let block = titled_block(title, theme);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if values.is_empty() {
        frame.render_widget(
            Paragraph::new("No data").style(Style::default().fg(theme.dim)),
            inner,
        );
        return;
    }

    let max = *values.iter().max().unwrap_or(&1).max(&1);
    let height = inner.height as usize;
    let width = inner.width as usize;
    if height < 2 || width < 2 {
        return;
    }

    let mut lines: Vec<String> = Vec::new();
    for row in 0..height {
        let threshold = max - (max * row as u64 / height as u64);
        let mut line = String::new();
        for col in 0..width {
            let idx = col * values.len() / width;
            let val = values.get(idx).copied().unwrap_or(0);
            if val >= threshold {
                line.push('█');
            } else {
                line.push(' ');
            }
        }
        lines.push(line);
    }

    let label = format!("{} (max {})", title, format_bytes(max));
    let mut output = label;
    output.push('\n');
    for line in lines {
        output.push_str(&line);
        output.push('\n');
    }
    frame.render_widget(Paragraph::new(output), inner);
}

fn format_bytes(bytes: u64) -> String {
    netwatch_stats::format_bytes(bytes, netwatch_core::Units::Auto)
}
