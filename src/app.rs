use crate::data::Dependency;
use crate::error;
use crate::ui::UiStyles;
use open;
use ratatui::crossterm::event::{KeyCode, KeyEvent};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Cell, Row, Table, Paragraph};
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tui_textarea::TextArea;

#[derive(Debug, Clone)]
pub struct Metadata {
    pub manifest_path: String,
    pub license: String,
    // size: u64,
    pub documentation: String,
    pub description: String,
}

pub struct App {
    selected_index: usize,
    viewport_start: usize,
    deps_map: HashMap<String, Metadata>,
    level1_deps: Vec<Dependency>,
    level2_deps: Vec<Dependency>,
    selected_package: Vec<Dependency>,
    filter_area: TextArea<'static>,
    styles: UiStyles,
}

impl App {
    fn default() -> App {
        App {
            selected_index: 0,
            viewport_start: 0,
            deps_map: HashMap::new(),
            level1_deps: Vec::new(),
            level2_deps: Vec::new(),
            selected_package: Vec::new(),
            filter_area: TextArea::default(),
            styles: UiStyles::default(),
        }
    }
    pub fn new<F: FnMut(&str) -> error::Result<()>>(
        path: &str,
        mut loading_screen_callback: F,
    ) -> (Self, Vec<error::Errors>) {
        let mut errors = Vec::new();
        errors.extend(loading_screen_callback("Loading...").err());

        let output = std::process::Command::new("cargo")
            .arg("metadata")
            .arg("--format-version")
            .arg("1")
            .current_dir(path)
            .output();
        let output = match output {
            Ok(output) => output,
            Err(_err) => {
                errors.push(error::Errors::RunCargoMetadata);
                return (Self::default(), errors);
            }
        };

        let metadata: Value = match serde_json::from_slice(&output.stdout) {
            Ok(metadata) => metadata,
            Err(_err) => {
                errors.push(error::Errors::ParseMetadata);
                Value::Null
            }
        };
        let mut meta_data = HashMap::new();

        if let Some(Value::Array(packages)) = metadata.get("packages") {
            for package in packages {
                if let Some(name) = package.get("name") {
                    if let Some(version) = package.get("version") {
                        let key = Dependency::get_key(&Dependency {
                            name: name.to_string(),
                            version: version.to_string(),
                        });
                        if meta_data.get(key.as_str()).is_some() {
                            continue;
                        }
                        meta_data.insert(
                            key,
                            Metadata {
                                manifest_path: get_metadta_value(package, "manifest_path"),
                                license: get_metadta_value(package, "license"),
                                documentation: get_metadta_value(package, "documentation"),
                                description: get_metadta_value(package, "description"),
                            },
                        );
                    }
                }
            }
        }

        match get_crate_info(path) {
            Ok(crate_info) => {
                let mut res = Self {
                    filter_area: TextArea::default(),
                    selected_index: 0,
                    viewport_start: 0,
                    deps_map: meta_data,
                    level1_deps: Vec::new(),
                    level2_deps: Vec::new(),
                    selected_package: vec![crate_info.clone()],
                    styles: UiStyles::default()
                };
                res.level1_deps = match res.get_deps(crate_info) {
                    Ok(level1_deps) => level1_deps,
                    Err(err) => {
                        errors.push(err);
                        Vec::new()
                    }
                };
                res.select_first_row();
                res.style_text_area();
                (res, errors)
            }
            Err(e) => {
                errors.push(e);
                (Self::default(), errors)
            }
        }
    }

    fn select_first_row(&mut self) {
        self.selected_index = 0;
        self.get_level2_dep();
    }

    fn get_level2_dep(&mut self) {
        self.level2_deps = self
            .get_deps(self.level1_deps[self.selected_index].clone())
            .unwrap_or_else(|_| Vec::new());
    }

    fn get_deps(&mut self, dep: Dependency) -> Result<Vec<Dependency>, error::Errors> {
        let meta_data = self.get_metadata(&dep);
        if meta_data.manifest_path.is_empty() {
            // get CARGO_HOME
            let cargo_home = std::env::var("CARGO_HOME").unwrap_or("~/.cargo".to_string());
            // Find the crates.io index directory under registry/src/
            let src_dir = Path::new(&cargo_home).join("registry/src");
            let index_dir = std::fs::read_dir(src_dir)?
                .filter_map(|entry| entry.ok())
                .find(|entry| entry.file_name().to_string_lossy().starts_with("index"))
                .ok_or_else(|| error::Errors::CratesIoIndexDirectoryNotFound);
            let dep_key = Dependency::get_key(&dep);
            parse_lock_file(
                index_dir?
                    .path()
                    .join(&dep_key)
                    .display()
                    .to_string()
                    .as_str(),
            )
        } else {
            parse_lock_file(&meta_data.manifest_path)
        }
    }

    pub fn draw(&mut self, area: Rect, buf: &mut Buffer) {
        // Create layout with header and tables
        let vertical = Layout::vertical([
            Constraint::Length(5), // Header height
            Constraint::Length(3),    // Filter height
            Constraint::Fill(1),   // Tables area
        ]);

        let [stat_area, filter_area, table_area] = vertical.areas(area);
        Widget::render(self.to_stats_table(), stat_area, buf);
        

        let [left_table, right_table] =
            Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                .areas(table_area);

        let [sub_description_area, filter_area] =
            Layout::horizontal([Constraint::Percentage(75), Constraint::Percentage(25)])
                .areas(filter_area);
        let data = self.get_metadata(&self.level1_deps[self.selected_index]);
        let sub_description_text = Paragraph::new(data.description)
                    .style(Style::default().fg(Color::Yellow))
                    .block(Block::default().borders(Borders::ALL).title(format!("Description of {}", self.level1_deps[self.selected_index].name)));
        Widget::render(sub_description_text, sub_description_area, buf);
        Widget::render(&self.filter_area, filter_area, buf);
        let visible_rows = table_area.height.saturating_sub(3);
        if self.selected_index < self.viewport_start {
            self.viewport_start = self.selected_index;
        } else if self.selected_index >= self.viewport_start + visible_rows as usize {
            self.viewport_start += 1;
        }

        let level1_table = Table::new(
            self.level1_deps
                .iter()
                .enumerate()
                .skip(self.viewport_start)
                .take(visible_rows as usize)
                .map(|(index, dep)| {
                    let row_style = if index == self.selected_index {
                        self.styles.selected_style
                    } else {
                        self.styles.text_style
                    };
                    let metadata = self.get_metadata(dep);
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
                    self.selected_package
                        .last()
                        .map_or("".to_string(), |dep| dep.name.to_string())
                ))
                .borders(Borders::ALL),
        );

        let level2_table = Table::new(
            self.level2_deps
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
                    self.level1_deps[self.selected_index].name
                ))
                .borders(Borders::ALL),
        );

        Widget::render(level1_table, left_table, buf);
        Widget::render(level2_table, right_table, buf);
    }

    pub fn update(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Left | KeyCode::Esc => {
                if self.selected_package.len() > 1 {
                    self.viewport_start = 0;
                    self.selected_package.pop();
                    self.level2_deps = Vec::new();
                    if let Some(last_dep) = self.selected_package.last() {
                        match self.get_deps(last_dep.clone()) {
                            Ok(deps) => self.level1_deps = deps,
                            Err(_) => self.level1_deps = Vec::new(),
                        }
                    } else {
                        self.level1_deps = Vec::new();
                    }
                    self.select_first_row();
                }
            }
            KeyCode::Right => {
                if self.level2_deps.len() > 0 {
                    self.viewport_start = 0;
                    self.selected_package
                        .push(self.level1_deps[self.selected_index].clone());
                    self.level1_deps = self.level2_deps.clone();
                    self.level2_deps = Vec::new();
                    self.select_first_row();
                }
            }
            KeyCode::Up => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                    self.get_level2_dep();
                }
            }
            KeyCode::Down => {
                if self.selected_index < self.level1_deps.len() - 1 {
                    self.selected_index += 1;
                    self.get_level2_dep();
                }
            }
            KeyCode::Enter => {
                if let Some(dep) = self.level2_deps.get(self.selected_index) {
                    let metadata = self.get_metadata(dep);
                    if !metadata.documentation.is_empty() {
                        // Open documentation URL
                        if let Err(e) = open::that(&metadata.documentation) {
                            eprintln!("Failed to open documentation: {}", e);
                        }
                    } else if let Ok(deps) = self.get_deps(dep.clone()) {
                        self.level2_deps = deps;
                        self.selected_index = 0;
                    }
                }
            }
            _ => {}
        }
    }

    fn get_metadata(&self, dep: &Dependency) -> Metadata {
        self.deps_map
            .get(Dependency::get_key(dep).as_str())
            .cloned()
            .unwrap_or_else(|| Metadata {
                description: String::new(),
                license: String::new(),
                documentation: String::new(),
                manifest_path: String::new(),
            })
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

    pub fn to_stats_table(&self) -> Table {
        let stats_widths = [
            Constraint::Length(20),
            Constraint::Length(20),
            Constraint::Length(20),
            Constraint::Min(40),
        ];

        let paths: Vec<String> = self
            .selected_package
            .iter()
            .map(|dep| dep.name.clone())
            .collect();
        let joined_path = paths.join(" > ");

        if let Some(current_crate) = self.selected_package.last() {
            let meta_data = self.get_metadata(current_crate);

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
                    Cell::from(format!("{:5}", self.level1_deps.len())).style(self.styles.text_style),
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

fn get_crate_info(path: &str) -> error::Result<Dependency> {
    let cargo_toml_path = PathBuf::from(path).join("Cargo.toml");
    if !cargo_toml_path.exists() {
        panic!("Cargo.toml not found");
    }
    let content = std::fs::read_to_string(&cargo_toml_path);
    let content = content?;
    let name = content
        .split('\n')
        .find(|line| line.starts_with("name = "))
        .unwrap_or("");
    let name = name.split('"').nth(1).unwrap_or("");
    let version = content
        .split('\n')
        .find(|line| line.starts_with("version = "))
        .unwrap_or("");
    let version = version.split('"').nth(1).unwrap_or("");
    Ok(Dependency {
        name: name.to_string(),
        version: version.to_string(),
    })
}

fn parse_lock_file(path: &str) -> Result<Vec<Dependency>, error::Errors> {
    let cargo_lock_path = path.replace("/Cargo.toml", "/Cargo.lock");
    let path = Path::new(&cargo_lock_path);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = std::fs::read_to_string(&path)?;
    let mut crates = Vec::new();
    let mut name = String::new();

    for line in content.lines() {
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value
                .trim()
                .trim_matches(|c: char| c.is_whitespace() || c == '"');

            match key {
                "name" => name = value.to_string(),
                "version" => {
                    let version = value.to_string();
                    if !name.is_empty() && !version.is_empty() {
                        crates.push(Dependency {
                            name: name.clone(),
                            version: version,
                        });
                        name = String::new();
                    }
                }
                _ => {}
            }
        }
    }

    Ok(crates)
}

fn get_metadta_value(content: &Value, key: &str) -> String {
    content
        .get(key)
        .map(|v| v.as_str().unwrap_or(""))
        .unwrap_or("")
        .to_string()
}
