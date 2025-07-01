use crate::data::DataState;
use crate::ui::UiStyles;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::prelude::{Color, Line, Span, Style, Text, Widget};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table};
use ratatui::{prelude::*, widgets::*};
use tui_textarea::TextArea;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum DisplayMode {
    /// Selecting a note from the list.
    #[default]
    View,
    Filter,
    Help,
    Sort,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum OrderBy {
    #[default]
    Size,
    Name,
    Version,
}

pub struct Screen {
    pub filter_area: TextArea<'static>,
    styles: UiStyles,
    pub mode: DisplayMode,
    pub viewport_start: usize,
}

impl Screen {
    pub fn default() -> Screen {
        let mut res = Screen {
            mode: DisplayMode::View,
            viewport_start: 0,
            filter_area: TextArea::default(),
            styles: UiStyles::default(),
        };
        res.style_text_area();
        res
    }

    pub fn clear_filter(&mut self, state: &mut DataState) {
        state.selected_index = 0;
        self.filter_area = TextArea::default();
        self.style_text_area();
        state.filter_input = String::from("");
    }

    pub fn display(&mut self, area: Rect, buf: &mut Buffer, state: &DataState) {
        self.render_main(area, buf, state);
        match self.mode {
            DisplayMode::Help => {
                self.render_help(area, buf);
            }
            DisplayMode::Sort => {
                self.render_sort(area, buf);
            }
            _ => {}
        }
    }

    fn render_sort(&mut self, area: Rect, buf: &mut Buffer) {
        let contents = vec![
            ("S", "Sort by size"),
            ("N", "Sort by name"),
            ("V", "Sort by version"),
            ("R", "Reverse sorting"),
        ];

        let popup_areas = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(contents.len() as u16 + 2),
            Constraint::Length(1),
        ])
        .split(area);

        let br_area = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(
                contents
                    .iter()
                    .map(|(_key, desc)| desc.len())
                    .max()
                    .unwrap_or_default() as u16
                    + 5,
            ),
            Constraint::Length(1),
        ])
        .split(popup_areas[1])[1];

        let rows = contents
            .into_iter()
            .map(|(key, description)| {
                Row::new(vec![
                    Span::styled(key, self.styles.hotkey_style),
                    Span::styled(description, self.styles.text_style),
                ])
            })
            .collect::<Vec<_>>();

        let widths = [Constraint::Length(2), Constraint::Fill(1)];

        let popup_table = Table::new(rows, widths)
            .block(Block::bordered())
            .column_spacing(1);

        // Clear the area and then render the widget on top.
        Widget::render(Clear, br_area, buf);
        Widget::render(popup_table, br_area, buf);
    }

    fn render_main(&mut self, area: Rect, buf: &mut Buffer, state: &DataState) {
        // Create layout with header and tables
        let vertical = Layout::vertical([
            Constraint::Length(5), // Header height
            Constraint::Length(3), // Filter height
            Constraint::Fill(1),   // Tables area
        ]);

        let [stat_area, filter_area, table_area] = vertical.areas(area);
        Widget::render(self.to_stats_table(state), stat_area, buf);

        let [left_table, right_table] =
            Layout::horizontal([Constraint::Percentage(65), Constraint::Percentage(35)])
                .areas(table_area);

        let [sub_description_area, filter_area] =
            Layout::horizontal([Constraint::Percentage(65), Constraint::Percentage(35)])
                .areas(filter_area);

        self.render_description(buf, state, sub_description_area);
        Widget::render(&self.filter_area, filter_area, buf);

        let visible_rows = table_area.height.saturating_sub(3);
        if state.selected_index < self.viewport_start {
            self.viewport_start = state.selected_index;
        } else if state.selected_index >= self.viewport_start + visible_rows as usize {
            self.viewport_start += 1;
        }

        let instructions_bot_left = Line::from(vec![
            Span::styled("A", self.styles.hotkey_style),
            Span::styled(": All──", self.styles.text_style),
            Span::styled("D", self.styles.hotkey_style),
            Span::styled(": Direct──", self.styles.text_style),
            Span::styled("▼", self.styles.hotkey_style),
            Span::styled(": Down──", self.styles.text_style),
            Span::styled("▲", self.styles.hotkey_style),
            Span::styled(": Up──", self.styles.text_style),
            Span::styled("►", self.styles.hotkey_style),
            Span::styled(": Right──", self.styles.text_style),
            Span::styled("◄", self.styles.hotkey_style),
            Span::styled(": Left──", self.styles.text_style),
            Span::styled("↵", self.styles.hotkey_style),
            Span::styled(": Open doc─", self.styles.text_style),
        ])
        .left_aligned();

        let title = Line::from(vec![
            Span::styled(
                if state.is_direct { "Direct" } else { "All" },
                Style::new().fg(Color::LightRed),
            ),
            Span::from(" dependencies of "),
            Span::from(
                state
                    .selected_package
                    .last()
                    .map_or("".to_string(), |dep| dep.name.to_string()),
            ),
        ]);
        let total_size = state.get_filter_deps().iter().map(|dep| dep.size).sum();
        let level1_table = Table::new(
            state
                .get_filter_deps()
                .iter()
                .enumerate()
                .skip(self.viewport_start)
                .take(visible_rows as usize)
                .map(|(index, metadata)| {
                    let row_style = if index == state.selected_index {
                        self.styles.selected_style
                    } else {
                        self.styles.unselected_style
                    };

                    let text_style = if !metadata.documentation.is_empty() {
                        self.styles.link_style
                    } else {
                        self.styles.text_style
                    };
                    let percentage = get_percentage(metadata.size, total_size);
                    Row::new(vec![
                        Cell::from((index + 1).to_string()).style(row_style),
                        Cell::from(Text::from(metadata.name.clone()).style(text_style))
                            .style(row_style),
                        Cell::from(metadata.version.clone()).style(row_style),
                        Cell::from(get_size(metadata.size)).style(row_style),
                        Cell::from(format!("{:>7.2}%", percentage.0)).style(row_style),
                        Cell::from(percentage.1.clone())
                            .style(self.styles.bar_chart_style)
                            .style(row_style),
                    ])
                })
                .collect::<Vec<_>>(),
            vec![
                Constraint::Percentage(10),
                Constraint::Percentage(30),
                Constraint::Percentage(10),
                Constraint::Percentage(10),
                Constraint::Percentage(15),
                Constraint::Percentage(15),
            ],
        )
        .header(Row::new(vec![
            "Index",
            "Name",
            "Version",
            "Size",
            "Percentage",
            "",
        ]))
        .block(
            Block::default()
                .title(title)
                .title_bottom(instructions_bot_left)
                .borders(Borders::ALL),
        );

        let instructions_bot_right = Line::from(vec![
            Span::styled("S", self.styles.hotkey_style),
            Span::styled("orting──", self.styles.subtitle_style),
            Span::styled("H", self.styles.hotkey_style),
            Span::styled("elp──", self.styles.subtitle_style),
            Span::styled("Q", self.styles.hotkey_style),
            Span::styled("uit", self.styles.subtitle_style),
        ])
        .right_aligned();

        let title = Line::from(vec![
            Span::styled(
                if state.is_direct { "Direct" } else { "All" },
                Style::new().fg(Color::LightRed),
            ),
            Span::from(" dependencies of "),
            Span::from(state.get_selected_dep().name),
        ]);
        let level2_table = Table::new(
            state
                .level2_deps
                .iter()
                .enumerate()
                .skip(self.viewport_start)
                .take(visible_rows as usize)
                .map(|(index, dep)| {
                    Row::new(vec![
                        Cell::from((index + 1).to_string()),
                        Cell::from(dep.name.clone()),
                        Cell::from(dep.version.clone()),
                        Cell::from(get_size(dep.size)),
                    ])
                })
                .collect::<Vec<_>>(),
            vec![
                Constraint::Percentage(40),
                Constraint::Percentage(30),
                Constraint::Percentage(30),
            ],
        )
        .header(Row::new(vec!["Name", "Version", "Size"]))
        .style(self.styles.subtitle_style)
        .block(
            Block::default()
                .title(title)
                .title_bottom(instructions_bot_right)
                .borders(Borders::ALL),
        );

        Widget::render(level1_table, left_table, buf);
        Widget::render(level2_table, right_table, buf);
    }

    fn render_description(
        &mut self,
        buf: &mut Buffer,
        state: &DataState,
        sub_description_area: Rect,
    ) {
        let data = state.get_selected_dep();
        let title = Line::from(vec![
            Span::from("Description of "),
            Span::from(state.get_selected_dep().name),
        ]);
        let sub_description_text = Paragraph::new(data.description)
            .style(self.styles.subtitle_style)
            .block(Block::default().borders(Borders::ALL).title(title));
        Widget::render(sub_description_text, sub_description_area, buf);
    }

    fn render_help(&mut self, area: Rect, buf: &mut Buffer) {
        let help_widths = [Constraint::Length(9), Constraint::Min(0)];

        let help_rows = [
            Row::new(vec![
                Cell::from("/ or F").style(self.styles.help_style),
                Cell::from("Enter the filter text box.").style(self.styles.text_style),
            ]),
            Row::new(vec![
                Cell::from("⏎ or Esc").style(self.styles.help_style),
                Cell::from("Exit the filter text box").style(self.styles.text_style),
            ]),
        ];

        let help_table = Table::new(help_rows, help_widths).column_spacing(1).block(
            Block::bordered()
                .title(style::Styled::set_style(
                    "Filter Syntax",
                    self.styles.help_style,
                ))
                .title_bottom(
                    Line::from(vec![
                        Span::styled("C", self.styles.hotkey_style),
                        Span::styled("lose", self.styles.text_style),
                    ])
                    .right_aligned(),
                ),
        );

        let popup_areas = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(10),
            Constraint::Fill(1),
        ])
        .split(area);

        let center_area = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(64),
            Constraint::Fill(1),
        ])
        .split(popup_areas[1])[1];

        // Clear the area and then render the help menu on top.
        Widget::render(Clear, center_area, buf);
        Widget::render(help_table, center_area, buf);
    }

    fn style_text_area(&mut self) {
        let title_top = Line::from(vec![
            Span::styled("F", self.styles.hotkey_style),
            Span::styled("ilter", self.styles.text_style),
        ])
        .left_aligned();

        let instructions_bot = Line::from(vec![
            Span::styled("C", self.styles.hotkey_style),
            Span::styled("lear filter", self.styles.text_style),
        ])
        .right_aligned();

        self.filter_area.set_style(self.styles.input_style);
        self.filter_area
            .set_cursor_line_style(self.styles.input_style);

        self.filter_area.set_block(
            Block::bordered()
                .title_top(title_top)
                .title_bottom(instructions_bot),
        );
    }

    pub fn filter(&mut self, state: &mut DataState) {
        state.selected_index = 0;
        let default = String::from("");
        state.filter_input = self
            .filter_area
            .lines()
            .first()
            .unwrap_or(&default)
            .to_string()
    }

    pub fn to_stats_table(&self, state: &DataState) -> Table {
        let stats_widths = [
            Constraint::Length(20),
            Constraint::Length(20),
            Constraint::Length(20),
            Constraint::Min(40),
        ];
        let total_size: u64 = state.get_filter_deps().iter().map(|dep| dep.size).sum();
        let paths: Vec<String> = state
            .selected_package
            .iter()
            .map(|dep| dep.name.clone())
            .collect();
        let joined_path = paths.join("/");

        if let Some(current_crate) = state.selected_package.last() {
            // Stats Area
            let stats_rows = [
                Row::new(vec![
                    Cell::from("Crate:").style(self.styles.text_style),
                    Cell::from(format!("{:5}", current_crate.name)).style(self.styles.title_style),
                    Cell::from("Version:").style(self.styles.text_style),
                    Cell::from(format!("v{:5}", current_crate.version))
                        .style(self.styles.text_style),
                ]),
                Row::new(vec![
                    Cell::from("License:").style(self.styles.text_style),
                    Cell::from(format!("{:5}", current_crate.license))
                        .style(self.styles.text_style),
                    Cell::from("Description:").style(self.styles.text_style),
                    Cell::from(format!("{:5}", current_crate.description))
                        .style(self.styles.text_style),
                ]),
                Row::new(vec![
                    Cell::from("Total count:").style(self.styles.text_style),
                    Cell::from(format!("{:5}", state.get_filter_deps().len().to_string()))
                        .style(self.styles.text_style),
                    Cell::from("Total size:").style(self.styles.text_style),
                    Cell::from(format!("{:5}", get_size(total_size)))
                        .style(self.styles.text_style)
                ]),
            ];
            return Table::new(stats_rows, stats_widths)
                .column_spacing(1)
                .block(Block::default().title(
                    Line::from(vec![
                        Span::styled("Statistics at ", self.styles.text_style),
                        Span::styled(joined_path, self.styles.title_style),
                    ])
                ).borders(Borders::ALL));
        }

        Table::new([Row::new([Cell::from("Error")])], stats_widths)
            .column_spacing(1)
            .block(Block::default().title("Statistics").borders(Borders::ALL))
    }
}

fn get_size(size: u64) -> String {
    let mut size = size;
    let mut unit = "B";
    if size >= 1024 {
        size /= 1024;
        unit = "KB";
    }
    if size >= 1024 {
        size /= 1024;
        unit = "MB";
    }
    if size >= 1024 {
        size /= 1024;
        unit = "GB";
    }
    format!("{} {}", size, unit)
}

/// Creates a bar chart representation of a percentage using Unicode block characters
fn create_bar_chart(percentage: f64) -> String {
    let mut bars = String::new();
    let full_blocks = (percentage / 10.0).floor() as usize;
    let partial_block = (percentage % 10.0) / 10.0;
    if percentage < 1.0 {
        return "".to_string();
    }
    // Add full blocks
    for _ in 0..full_blocks {
        bars.push('█'); // Full block
    }

    // Add partial block if needed
    if partial_block > 0.0 {
        bars.push(match (partial_block * 8.0).round() as u8 {
            1 => '▏',
            2 => '▎',
            3 => '▍',
            4 => '▌',
            5 => '▋',
            6 => '▊',
            7 => '▉',
            _ => '█',
        });
    }

    bars
}

/// Calculates percentage and returns both the numerical value and bar chart representation
fn get_percentage(size: u64, total: u64) -> (f64, String) {
    let percentage = (size as f64 / total as f64) * 100.0;
    (percentage, create_bar_chart(percentage))
}
