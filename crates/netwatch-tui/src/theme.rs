use ratatui::style::{Color, Modifier, Style};

#[derive(Debug, Clone)]
pub struct Theme {
    pub border: Color,
    pub title: Color,
    pub accent: Color,
    pub text: Color,
    pub dim: Color,
    pub up: Color,
    pub down: Color,
    pub alert: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            border: Color::Cyan,
            title: Color::White,
            accent: Color::Yellow,
            text: Color::Gray,
            dim: Color::DarkGray,
            up: Color::Green,
            down: Color::Red,
            alert: Color::Red,
        }
    }
}

impl Theme {
    pub fn title_style(&self) -> Style {
        Style::default()
            .fg(self.title)

}}
