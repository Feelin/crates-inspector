use crate::data::DataState;
use crate::ui::UiStyles;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::prelude::{Color, Line, Span, Style, Text, Widget};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table};
use tui_textarea::TextArea;

pub struct Screen {
    filter_area: TextArea<'static>,
    styles: UiStyles,
    pub viewport_start: usize,
}

impl Screen {
    pub fn default() -> Screen {
        let mut res = Screen {
            viewport_start: 0,
            filter_area: TextArea::default(),
            styles: UiStyles::default(),
        };
        res.style_text_area();
        res
    }
    pub fn display(&mut self, area: Rect, buf: &mut Buffer, state: &DataState) {
        // Create layout with header and tables
        let vertical = Layout::vertical([
            Constraint::Length(5), // Header height
            Constraint::Length(3),    // Filter height
            Constraint::Fill(1),   // Tables area
        ]);

        let [stat_area, filter_area, table_area] = vertical.areas(area);
        Widget::render(self.to_stats_table(state), stat_area, buf);
        
        let [left_table, right_table] =
            Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                .areas(table_area);

        let [sub_description_area, filter_area] =
            Layout::horizontal([Constraint::Percentage(75), Constraint::Percentage(25)])
                .areas(filter_area);
        let data = state.get_metadata(&state.level1_deps[state.selected_index]);
        let sub_description_text = Paragraph::new(data.description)
            .style(Style::default().fg(Color::Yellow))
            .block(Block::default().borders(Borders::ALL).title(format!("Description of {}", state.level1_deps[state.selected_index].name)));
        Widget::render(sub_description_text, sub_description_area, buf);
        Widget::render(&self.filter_area, filter_area, buf);
        let visible_rows = table_area.height.saturating_sub(3);
        if state.selected_index < self.viewport_start {
            self.viewport_start = state.selected_index;
        } else if state.selected_index >= self.viewport_start + visible_rows as usize {
            self.viewport_start += 1;
        }

        let level1_table = Table::new(
            state.level1_deps
                .iter()
                .enumerate()
                .skip(self.viewport_start)
                .take(visible_rows as usize)
                .map(|(index, dep)| {
                    let row_style = if index == state.selected_index {
                        self.styles.selected_style
                    } else {
                        self.styles.text_style
                    };
                    let metadata = state.get_metadata(dep);
                    let text_style = if metadata.documentation.is_empty() {
                        self.styles.link_style
                    } else {
                        self.styles.text_style
                    };

                    Row::new(vec![
                        Cell::from((index + 1).to_string()).style(row_style),
                        Cell::from(Text::from(dep.name.clone()).style(text_style)).style(row_style),
                        Cell::from(dep.version.clone()).style(row_style),
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
                    .borders(Borders::ALL),
            );

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
                        state.level1_deps[state.selected_index].name
                    ))
                    .borders(Borders::ALL),
            );

        Widget::render(level1_table, left_table, buf);
        Widget::render(level2_table, right_table, buf);
    }


    fn style_text_area(&mut self) {
        let title_top = Line::from(vec![
            Span::styled("F", self.styles.hotkey_style),
            Span::styled("ilter", self.styles.title_style),
        ])
            .left_aligned();

        // The hotkey instructions at the bottom.
        let instructions = Line::from(vec![
            Span::styled("C", self.styles.hotkey_style),
            Span::styled("lear filter", self.styles.text_style),
        ])
            .right_aligned();

        let instructions_bot = Line::from(vec![
            Span::styled("H", self.styles.hotkey_style),
            Span::styled("elp", self.styles.text_style),
        ])
            .right_aligned();

        self.filter_area.set_style(self.styles.input_style);
        self.filter_area
            .set_cursor_line_style(self.styles.input_style);

        self.filter_area.set_block(
            Block::bordered()
                .title_top(title_top)
                .title_top(instructions)
                .title_bottom(instructions_bot),
        );
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
            let meta_data = state.get_metadata(current_crate);

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
                    Cell::from(format!("{:5}", meta_data.license)).style(self.styles.text_style),
                    Cell::from("Description:").style(self.styles.text_style),
                    Cell::from(format!("{:5}", meta_data.description)).style(self.styles.text_style),
                ]),
                Row::new(vec![
                    Cell::from("Dependencies count:").style(self.styles.text_style),
                    Cell::from(format!("{:5}", state.level1_deps.len())).style(self.styles.text_style),
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