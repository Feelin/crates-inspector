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
enum Sorting {
    #[default]
    ASC,
    DESC
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
enum OrderBy {
    #[default]
    Default,
    Name,
    Size,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Order {
    order_by: OrderBy,
    sort: Sorting
}



pub struct Screen {
    pub filter_area: TextArea<'static>,
    styles: UiStyles,
    pub mode: DisplayMode,
    pub order: Order,
    pub viewport_start: usize,
}

impl Screen {
    pub fn default() -> Screen {
        let mut res = Screen {
            mode: DisplayMode::View,
            order: Order::default(),
            viewport_start: 0,
            filter_area: TextArea::default(),
            styles: UiStyles::default(),
        };
        res.style_text_area();
        res
    }

    pub fn display(&mut self, area: Rect, buf: &mut Buffer, state: &DataState) {
        self.render_list(area, buf, state);
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
            ("D", "Sort by default"),
            ("N", "Sort by name"),
            ("S", "Sort by size"),
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
    

    fn render_list(&mut self, area: Rect, buf: &mut Buffer, state: &DataState) {
        // Create layout with header and tables
        let vertical = Layout::vertical([
            Constraint::Length(5), // Header height
            Constraint::Length(3),    // Filter height
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
        let data = state.get_filter_deps()[state.selected_index].clone();
        let sub_description_text = Paragraph::new(data.description)
            .style(Style::default().fg(Color::Yellow))
            .block(Block::default().borders(Borders::ALL).title(format!("Description of {}", state.get_filter_deps()[state.selected_index].name)));
        Widget::render(sub_description_text, sub_description_area, buf);
        Widget::render(&self.filter_area, filter_area, buf);
        let visible_rows = table_area.height.saturating_sub(3);
        if state.selected_index < self.viewport_start {
            self.viewport_start = state.selected_index;
        } else if state.selected_index >= self.viewport_start + visible_rows as usize {
            self.viewport_start += 1;
        }


        let instructions_bot_left = Line::from(vec![
            Span::styled("J", self.styles.hotkey_style),
            Span::styled("/", self.styles.text_style),
            Span::styled("▼", self.styles.hotkey_style),
            Span::styled(": Down──", self.styles.text_style),
            Span::styled("K", self.styles.hotkey_style),
            Span::styled("/", self.styles.text_style),
            Span::styled("▲", self.styles.hotkey_style),
            Span::styled(": Up──", self.styles.text_style),
            Span::styled("L", self.styles.hotkey_style),
            Span::styled("/", self.styles.text_style),
            Span::styled("►", self.styles.hotkey_style),
            Span::styled(": Right──", self.styles.text_style),
            Span::styled("◄", self.styles.hotkey_style),
            Span::styled(": Left──", self.styles.text_style),
            Span::styled("↵", self.styles.hotkey_style),
            Span::styled(": Open──", self.styles.text_style),
        ]).left_aligned();

        let level1_table = Table::new(
            state.get_filter_deps()
                .iter()
                .enumerate()
                .skip(self.viewport_start)
                .take(visible_rows as usize)
                .map(|(index, metadata)| {
                    let row_style = if index == state.selected_index {
                        self.styles.selected_style
                    } else {
                        self.styles.text_style
                    };
                    
                    let text_style = if metadata.documentation.is_empty() {
                        self.styles.link_style
                    } else {
                        self.styles.text_style
                    };

                    Row::new(vec![
                        Cell::from((index + 1).to_string()).style(row_style),
                        Cell::from(Text::from(metadata.name.clone()).style(text_style)).style(row_style),
                        Cell::from(metadata.version.clone()).style(row_style),
                    ])
                })
                .collect::<Vec<_>>(),
            vec![
                Constraint::Percentage(20),
                Constraint::Percentage(50),
                Constraint::Percentage(30),
            ],
        )
            .header(Row::new(vec!["Index", "Name", "Version"]))
            .block(
                Block::default()
                    .title(format!(
                        "Dependencies of {}",
                        state.selected_package
                            .last()
                            .map_or("".to_string(), |dep| dep.name.to_string())
                    ))
                    .title_bottom(instructions_bot_left)
                    .borders(Borders::ALL),
            );

        let instructions_bot_right = Line::from(vec![
            Span::styled("S", self.styles.hotkey_style),
            Span::styled("orting──", self.styles.text_style),
            Span::styled("H", self.styles.hotkey_style),
            Span::styled("elp──", self.styles.text_style),
            Span::styled("Q", self.styles.hotkey_style),
            Span::styled("uit", self.styles.text_style),
        ]).right_aligned();

        let level2_table = Table::new(
            state.level2_deps
                .iter()
                .enumerate()
                .skip(self.viewport_start)
                .take(visible_rows as usize)
                .map(|(index, dep)| {
                    Row::new(vec![
                        Cell::from((index + 1).to_string()),
                        Cell::from(dep.name.clone()),
                        Cell::from(dep.version.clone()),
                    ])
                })
                .collect::<Vec<_>>(),
            vec![
                Constraint::Percentage(20),
                Constraint::Percentage(50),
                Constraint::Percentage(30),
            ],
        )
            .header(Row::new(vec!["Index", "Name", "Version"]))
            .block(
                Block::default()
                    .title(format!(
                        "Dependencies of {}",
                        state.get_filter_deps()[state.selected_index].name
                    ))
                    .title_bottom(instructions_bot_right)
                    .borders(Borders::ALL),
            );

        Widget::render(level1_table, left_table, buf);
        Widget::render(level2_table, right_table, buf);
    }

    fn render_help(&mut self, area: Rect, buf: &mut Buffer) {
        let help_widths = [Constraint::Length(9), Constraint::Min(0)];

        let help_rows = [
            Row::new(vec![
                Cell::from("/ or F").style(self.styles.subtitle_style),
                Cell::from("Enter the filter text box.").style(self.styles.text_style),
            ]),
            Row::new(vec![
                Cell::from("⏎ or Esc").style(self.styles.subtitle_style),
                Cell::from("Exit the filter text box").style(self.styles.text_style),
            ]),
        ];

        let help_table = Table::new(help_rows, help_widths).column_spacing(1).block(
            Block::bordered()
                .title(style::Styled::set_style(
                    "Filter Syntax",
                    self.styles.title_style,
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
        ]).split(area);

        let center_area = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(64),
            Constraint::Fill(1),
        ]).split(popup_areas[1])[1];

        // Clear the area and then render the help menu on top.
        Widget::render(Clear, center_area, buf);
        Widget::render(help_table, center_area, buf);
    }

    fn style_text_area(&mut self) {
        let title_top = Line::from(vec![
            Span::styled("F", self.styles.hotkey_style),
            Span::styled("ilter", self.styles.title_style),
        ])
            .left_aligned();


        let instructions_bot = Line::from(vec![
            Span::styled("C", self.styles.hotkey_style),
            Span::styled("lear filter", self.styles.text_style),
        ]).right_aligned();

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
        let default = String::from("");
        state.filter = self.filter_area
            .lines()
            .first()
            .unwrap_or(&default).to_string()
    }

    pub fn to_stats_table(&self, state: &DataState) -> Table {
        let stats_widths = [
            Constraint::Length(20),
            Constraint::Length(20),
            Constraint::Length(20),
            Constraint::Min(40),
        ];

        let paths: Vec<String> = state
            .selected_package
            .iter()
            .map(|dep| dep.name.clone())
            .collect();
        let joined_path = paths.join(" > ");

        if let Some(current_crate) = state.selected_package.last() {

            // Stats Area
            let stats_rows = [
                Row::new(vec![
                    Cell::from("Crate:").style(self.styles.text_style),
                    Cell::from(format!("{:5}", current_crate.name)).style(self.styles.text_style),
                    Cell::from("Version:").style(self.styles.text_style),
                    Cell::from(format!("v{:5}", current_crate.version)).style(self.styles.text_style),
                ]),
                Row::new(vec![
                    Cell::from("License:").style(self.styles.text_style),
                    Cell::from(format!("{:5}", current_crate.license)).style(self.styles.text_style),
                    Cell::from("Description:").style(self.styles.text_style),
                    Cell::from(format!("{:5}", current_crate.description)).style(self.styles.text_style),
                ]),
                Row::new(vec![
                    Cell::from("Dependencies count:").style(self.styles.text_style),
                    Cell::from(format!("{:5}", state.get_filter_deps().len())).style(self.styles.text_style),
                    Cell::from("Path:").style(self.styles.text_style),
                    Cell::from(joined_path).style(self.styles.text_style),
                ]),
            ];
            return Table::new(stats_rows, stats_widths)
                .column_spacing(1)
                .block(Block::default().title("Statistics").borders(Borders::ALL));
        }

        Table::new([Row::new([Cell::from("Error")])], stats_widths)
            .column_spacing(1)
            .block(Block::default().title("Statistics").borders(Borders::ALL))
    }
}