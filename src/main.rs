use std::collections::HashMap;
use ratatui::{
    prelude::{CrosstermBackend},
    Terminal,
    widgets::{Block, Borders, Table, Row, Cell},
    layout::{Layout, Constraint, Direction},
    style::{Style, Color},
};
use serde_json::Value;
use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
    ExecutableCommand,
};
use std::io::{stdout};
use std::path::{Path, PathBuf};

use clap::Parser;
use ratatui::text::Text;
use ratatui::widgets::Paragraph;

#[derive(Debug, Clone)]
struct Dependency {
    name: String,
    version: String,
}

impl Dependency {
    fn get_key(dep: Dependency) -> String {
        format!("{}-{}", dep.name, dep.version).replace('"', "")
    }
}


#[derive(Debug, Clone)]
struct Metadata {
    manifest_path: String,
    // license: String,
    // size: u64,
    // documentation: String,
}

struct App {
    selected_index: usize,
    viewport_start: usize,
    deps_map: HashMap<String, Metadata>,
    level1_deps: Vec<Dependency>,
    level2_deps: Vec<Dependency>,
    selected_package: Vec<Dependency>,
}

impl App {
    fn new() -> Result<Self> {
        let args = Args::parse();
        let output = std::process::Command::new("cargo")
            .arg("metadata")
            .arg("--format-version")
            .arg("1")
            .current_dir(args.path.clone())
            .output()?;

        
        let crate_info = get_crate_info(args.path);
        let metadata: Value = serde_json::from_slice(&output.stdout)?;
        let mut meta_data = HashMap::new();

        if let Some(Value::Array(packages)) = metadata.get("packages") {
            for package in packages {
                if let Some(name) = package.get("name") {
                    if let Some(version) = package.get("version") {
                        let key = Dependency::get_key(Dependency {
                            name: name.to_string(),
                            version: version.to_string(),
                        });
                        if meta_data.get(key.as_str()).is_some() {
                            continue;
                        }
                        meta_data.insert(key, Metadata {
                            manifest_path: package.get("manifest_path").unwrap().as_str().unwrap_or("unknown").to_string(),
                            // license: package.get("license").unwrap().as_str().unwrap_or("unknown").to_string(),
                            // size: 0,
                            // documentation: package.get("documentation").unwrap().as_str().unwrap_or("unknown").to_string(),
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
        self.selected_package.push(self.level1_deps[0].clone());
        self.get_level2_dep();
    }
    
    fn get_level2_dep(&mut self) {
        self.level2_deps = self.get_deps(self.selected_package.last().unwrap().clone()).unwrap();
    }

    fn get_deps(&mut self, dep: Dependency) -> Result<Vec<Dependency>> {
        if let Some(dep) = self.deps_map.get(Dependency::get_key(dep.clone()).as_str()) {
            Ok(parse_lock_file(&dep.manifest_path))
        } else {
            // get CARGO_HOME
            let cargo_home = std::env::var("CARGO_HOME").unwrap_or("~/.cargo".to_string());
            // Find the crates.io index directory under registry/src/
            let src_dir = Path::new(&cargo_home).join("registry/src");
            let index_dir = std::fs::read_dir(src_dir)?
                .filter_map(|entry| entry.ok())
                .find(|entry| entry.file_name().to_string_lossy().starts_with("index"))
                .ok_or_else(|| anyhow::anyhow!("Could not find crates.io index directory"))?;
            
            let dep_key = Dependency::get_key(dep.clone());
            Ok(parse_lock_file(&format!("{}", index_dir.path().join(&dep_key).display())))
        }
    }
}


#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = ".")]
    path: String,
}

fn main() -> Result<()> {
    enable_raw_mode()?;
    
    let mut stdout = stdout();
    stdout.execute(crossterm::terminal::EnterAlternateScreen)?;
    
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    
    let mut app = App::new()?;

    loop {
        terminal.draw(|f| {
            // Create layout with header and tables
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),  // Header height
                    Constraint::Min(0),      // Tables area
                ])
                .split(f.size());

            // Header showing selected package path
            let header = Block::default()
                .title("Dependency path")
                .borders(Borders::ALL);
            f.render_widget(header, chunks[0]);
            
            let paths: Vec<String> = app.selected_package.iter().map(|dep| dep.name.clone()).collect();
            let joined_path = paths.join(" > ");
            let text = Text::from(joined_path);
            f.render_widget(Paragraph::new(text), chunks[0]);
            
            // Split the tables area horizontally
            let table_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(50),
                    Constraint::Percentage(50),
                ])
                .split(chunks[1]);
            
            let visible_rows = chunks[0].height.saturating_sub(3);
            if app.selected_index < app.viewport_start {
                app.viewport_start = app.selected_index;
            } else if app.selected_index >= app.viewport_start + visible_rows as usize {
                app.viewport_start += 1;
            }
            let mut title = app.selected_package.last().unwrap().name.to_string();
            if app.selected_package.len() > 1 {
                title = app.selected_package[app.selected_package.len() - 2].name.to_string()
            }
            let level1_table = Table::new(
                app.level1_deps.iter()
                    .enumerate()
                    .skip(app.viewport_start)
                    .take(visible_rows as usize)
                    .map(|(index, dep)| {
                        let style = if index == app.selected_index {
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
            .block(Block::default().title(format!("Dependencies of {}", title)).borders(Borders::ALL));

            let level2_table = Table::new(
                app.level2_deps.iter().enumerate().skip(app.viewport_start).take(visible_rows as usize).map(|(index, dep)| {
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
            .block(Block::default().title(format!("Dependencies of {}", app.selected_package.last().unwrap().name)).borders(Borders::ALL));

            f.render_widget(level1_table, table_chunks[0]);
            f.render_widget(level2_table, table_chunks[1]);
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q'|'Q') => break,
                KeyCode::Left => {
                    if app.selected_package.len() > 2 {
                        app.selected_package.pop();
                        app.level2_deps = Vec::new();
                        app.level1_deps = app.get_deps(app.selected_package[app.selected_package.len() - 2].clone())?;
                        app.select_first_row();
                    }
                },
                KeyCode::Right => {
                    if app.level2_deps.len() > 0 {
                        app.level1_deps = app.level2_deps;
                        app.level2_deps = Vec::new();
                        app.select_first_row();
                    }
                },
                KeyCode::Up => {
                    if app.selected_index > 0 {
                        app.selected_index -= 1;
                        app.selected_package.pop();
                        app.selected_package.push(app.level1_deps[app.selected_index].clone());
                        app.get_level2_dep();
                    }
                }
                KeyCode::Down => {
                    if app.selected_index < app.level1_deps.len() - 1 {
                        app.selected_index += 1;
                        app.selected_package.pop();
                        app.selected_package.push(app.level1_deps[app.selected_index].clone());
                        app.get_level2_dep();
                    }
                }
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    std::io::stdout().execute(crossterm::terminal::LeaveAlternateScreen)?;
    Ok(())
}

fn get_crate_info(path: String) -> Dependency {
    let cargo_toml_path = PathBuf::from(path).join("Cargo.toml");
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

fn parse_lock_file(path: &str) -> Vec<Dependency> {
    let cargo_lock_path = path.replace("Cargo.toml", "Cargo.lock");
    if !PathBuf::from(&cargo_lock_path).exists() {
        return Vec::new();
    }
    let content = std::fs::read_to_string(&cargo_lock_path).unwrap();
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

    crates
}