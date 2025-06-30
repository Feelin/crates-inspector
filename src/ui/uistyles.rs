use ratatui::style::*;

/// A struct that holds a collection of styles for a consistent looking UI.
/// This is a pure data struct, having no methods and only public attributes.
#[derive(Copy, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct UiStyles {
    /// For titles of boxes.
    pub title_style: Style,
    /// For table headers etc.
    pub subtitle_style: Style,
    /// For letters that indicate a hotkey within a title.
    pub hotkey_style: Style,
    /// For normal text.
    pub text_style: Style,
    /// For selected list/table rows or other text.
    pub selected_style: Style,
    /// For text in an input area.
    pub input_style: Style,
    /// For link text.
    pub link_style: Style,
    pub bar_chart_style: Style,
    pub unselected_style: Style,
    pub help_style: Style,
}

impl Default for UiStyles {
    fn default() -> Self {
        Self {
            title_style: Style::new()
                .fg(Color::LightBlue)
                .add_modifier(Modifier::BOLD),
            subtitle_style: Style::new()
                .fg(Color::LightGreen)
                .add_modifier(Modifier::ITALIC),
            help_style: Style::new()
                .fg(Color::LightYellow)
                .add_modifier(Modifier::ITALIC),
            hotkey_style: Style::new()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            text_style: Style::new().fg(Color::White),
            link_style: Style::new().fg(Color::Cyan),
            selected_style: Style::new()
                .bg(Color::LightBlue)
                .add_modifier(Modifier::BOLD),
            unselected_style: Style::default(),
            input_style: Style::new().add_modifier(Modifier::ITALIC),
            bar_chart_style: Style::new().fg(Color::Gray),
        }
    }
}
