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

}
