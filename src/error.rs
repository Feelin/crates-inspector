use ratatui::{style, widgets::*};
use thiserror::Error;
pub type Result<T> = std::result::Result<T, Errors>;

#[derive(Error, Debug)]
pub enum Errors {
    #[error("An IO operation failed: {0}")]
    IO(#[from] std::io::Error),
    #[error("Area too small, main window might not display correctly.")]
    SmallArea,
    #[error("Failed to run cargo metadata.")]
    RunCargoMetadata,
    #[error("Failed to parse metadata.")]
    ParseMetadata,
}

impl Errors {
    pub fn to_ratatui(&self) -> Paragraph<'_> {
        Paragraph::new(format!("{}", &self)).style(style::Style::new().fg(style::Color::Red))
    }
}
