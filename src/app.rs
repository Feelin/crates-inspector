use crate::data::Dependency;
use crate::ui::UiStyles;
use ratatui::crossterm::event::{KeyCode, KeyEvent};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Cell, Row, Table};
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};


#[derive(Debug, Clone)]
pub struct Metadata {
    pub manifest_path: String,
    pub license: String,
    // size: u64,
    pub documentation: String,
    pub description: String
}


pub struct App {
    selected_index: usize,
    viewport_start: usize,
    deps_map: HashMap<String, Metadata>,
    level1_deps: Vec<Dependency>,
    level2_deps: Vec<Dependency>,
    selected_package: Vec<Dependency>,
}

impl App {
    pub fn new(path: String) -> anyhow::Result<Self> {
        
        let output = std::process::Command::new("cargo")
            .arg("metadata")
            .arg("--format-version")
            .arg("1")
            .current_dir(path.clone())
            .output()?;


        let crate_info = get_crate_info(path);
        let metadata: Value = serde_json::from_slice(&output.stdout)?;
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
                        meta_data.insert(key, Metadata {
                            manifest_path: package.get("manifest_path").unwrap().as_str().unwrap_or("unknown").to_string(),
                            license: package.get("license").unwrap().as_str().unwrap_or("unknown").to_string(),
                            documentation: package.get("documentation").unwrap().as_str().unwrap_or("unknown").to_string(),
                            description: package.get("description").unwrap().as_str().unwrap_or("unknown").to_string(),
                        });
                    }
                }
            }
        }


        let mut res = Self {
            selected_index: 0,
            viewport_start: 0,
            deps_map: meta_data,
            level1_deps: Vec::new(),
            level2_deps: Vec::new(),
            selected_package: vec![crate_info.clone()],
        };
        res.level1_deps = res.get_deps(crate_info)?;
        res.select_first_row();
        Ok(res)
    }

    fn select_first_row(&mut self) {
        self.selected_index = 0;
        self.get_level2_dep();
    }

    fn get_level2_dep(&mut self) {
        self.level2_deps = self.get_deps(self.level1_deps[self.selected_index].clone()).unwrap();
    }

    fn get_deps(&mut self, dep: Dependency) -> Result<Vec<Dependency>, std::io::Error> {
        if let Some(dep) = self.deps_map.get(Dependency::get_key(&dep).as_str()) {
            parse_lock_file(&dep.manifest_path)
        } else {
            // get CARGO_HOME
            let cargo_home = std::env::var("CARGO_HOME").unwrap_or("~/.cargo".to_string());
            // Find the crates.io index directory under registry/src/
            let src_dir = Path::new(&cargo_home).join("registry/src");
            let index_dir = std::fs::read_dir(src_dir).unwrap()
                .filter_map(|entry| entry.ok())
                .find(|entry| entry.file_name().to_string_lossy().starts_with("index"))
                .ok_or_else(|| anyhow::anyhow!("Could not find crates.io index directory"));
            let dep_key = Dependency::get_key(&dep);
            parse_lock_file(index_dir.unwrap().path().join(&dep_key).display().to_string().as_str())
        }
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        // Create layout with header and tables
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(5),  // Header height
                Constraint::Min(0),      // Tables area
            ])
            .split(frame.area());

        
        frame.render_widget(self.to_stats_table(), chunks[0]);

        // Split the tables area horizontally
        let table_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ])
            .split(chunks[1]);

        let visible_rows = chunks[1].height.saturating_sub(3);
        if self.selected_index < self.viewport_start {
            self.viewport_start = self.selected_index;
        } else if self.selected_index >= self.viewport_start + visible_rows as usize {
            self.viewport_start += 1;
        }
        
        let level1_table = Table::new(
            self.level1_deps.iter()
                .enumerate()
                .skip(self.viewport_start)
                .take(visible_rows as usize)
                .map(|(index, dep)| {
                    let style = if index == self.selected_index {
                        Style::default().bg(Color::Blue).fg(Color::White)
                    } else {
                        Style::default()
                    };
                    Row::new(vec![
                        Cell::from((index + 1).to_string()).style(style),
                        Cell::from(dep.name.clone()).style(style),
                        Cell::from(dep.version.clone()).style(style),
                    ])
                })
                .collect::<Vec<_>>(),
            vec![
                Constraint::Percentage(20),
                Constraint::Percentage(50),
                Constraint::Percentage(30),
            ]
        )
            .header(Row::new(vec!["Index", "Name", "Version"]))
            .block(Block::default().title(format!("Dependencies of {}", self.selected_package.last().unwrap().name.to_string()))
                .borders(Borders::ALL));

        let level2_table = Table::new(
            self.level2_deps.iter().enumerate().skip(self.viewport_start).take(visible_rows as usize).map(|(index, dep)| {
                Row::new(vec![
                    Cell::from((index + 1).to_string()),
                    Cell::from(dep.name.clone()),
                    Cell::from(dep.version.clone()),
                ])
            }).collect::<Vec<_>>(),
            vec![
                Constraint::Percentage(20),
                Constraint::Percentage(50),
                Constraint::Percentage(30),
            ]
        )
            .header(Row::new(vec!["Index", "Name", "Version"]))
            .block(Block::default().title(format!("Dependencies of {}", self.level1_deps[self.selected_index].name)).borders(Borders::ALL));

        frame.render_widget(level1_table, table_chunks[0]);
        frame.render_widget(level2_table, table_chunks[1]);
    }
    
    pub fn update(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Left => {
                if self.selected_package.len() > 1 {
                    self.viewport_start = 0;
                    self.selected_package.pop();
                    self.level2_deps = Vec::new();
                    self.level1_deps = self.get_deps(self.selected_package.last().unwrap().clone()).unwrap();
                    self.select_first_row();
                }
            },
            KeyCode::Right => {
                if self.level2_deps.len() > 0 {
                    self.viewport_start = 0;
                    self.selected_package.push(self.level1_deps[self.selected_index].clone());
                    self.level1_deps = self.level2_deps.clone();
                    self.level2_deps = Vec::new();
                    self.select_first_row();
                }
            },
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
            _ => {}
        }
    }

    pub fn to_stats_table(&self) -> Table {
        let stats_widths = [
            Constraint::Length(20),
            Constraint::Length(20),
            Constraint::Length(20),
            Constraint::Min(40),
        ];
        let styles = UiStyles::default();

        let paths: Vec<String> = self.selected_package.iter().map(|dep| dep.name.clone()).collect();
        let joined_path = paths.join(" > ");
        let mut description = String::from("");
        let mut license = String::from("");
        
        if let Some(current_crate) = self.selected_package.last() {
            if let Some(meta_data) = self.deps_map.get(Dependency::get_key(current_crate).as_str()) {
                description = meta_data.description.clone();
                license = meta_data.license.clone();
            }    
        }
        
        // Stats Area
        let stats_rows = [
            Row::new(vec![
                Cell::from("Crate:").style(styles.text_style),
                Cell::from(format!("{:5}", self.selected_package.last().unwrap().name)).style(styles.text_style),
                Cell::from("Version:").style(styles.text_style),
                Cell::from(format!("v{:5}", self.selected_package.last().unwrap().version)).style(styles.text_style),
            ]),
            Row::new(vec![
                Cell::from("License:").style(styles.text_style),
                Cell::from(format!("{:5}", license)).style(styles.text_style),
                Cell::from("Description:").style(styles.text_style),
                Cell::from(format!("{:5}", description)).style(styles.text_style),
            ]),
            Row::new(vec![
                Cell::from("Dependencies count:").style(styles.text_style),
                Cell::from(format!("{:5}", self.level1_deps.len())).style(styles.text_style),
                Cell::from("Path:").style(styles.text_style),
                Cell::from(joined_path).style(styles.text_style),
            ]),

        ];

        Table::new(stats_rows, stats_widths).column_spacing(1)
            .block(Block::default().title("Statistics").borders(Borders::ALL))
    }
}




fn get_crate_info(path: String) -> Dependency {
    let cargo_toml_path = PathBuf::from(path).join("Cargo.toml");
    if !cargo_toml_path.exists() {
        panic!("Cargo.toml not found");
    }
    let content = std::fs::read_to_string(&cargo_toml_path);
    let content = content.unwrap();
    let name = content.split('\n').find(|line| line.starts_with("name = ")).unwrap();
    let name = name.split('"').nth(1).unwrap();
    let version = content.split('\n').find(|line| line.starts_with("version = ")).unwrap();
    let version = version.split('"').nth(1).unwrap();
    Dependency {
        name: name.to_string(),
        version: version.to_string(),
    }
}

fn parse_lock_file(path: &str) -> Result<Vec<Dependency>, std::io::Error> {
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
            let value = value.trim().trim_matches(|c: char| c.is_whitespace() || c == '"');

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
                },
                _ => {}
            }
        }
    }

    Ok(crates)
}