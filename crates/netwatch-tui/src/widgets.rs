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

}}
