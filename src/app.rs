use crate::data::{DataState, Dependency};
use crate::error;
use crate::ui::{DisplayMode, Screen};

use open;
use ratatui::crossterm::event::{KeyCode, KeyEvent};
use ratatui::prelude::*;
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Metadata {
    pub manifest_path: String,
    pub license: String,
    // size: u64,
    pub documentation: String,
    pub description: String,
}

pub struct App {
    state: DataState,
    screen: Screen,
}

impl App {
    fn default() -> App {
        App {
            state: DataState::default(),
            screen: Screen::default(),
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
                                manifest_path: get_metadata(package, "manifest_path"),
                                license: get_metadata(package, "license"),
                                documentation: get_metadata(package, "documentation"),
                                description: get_metadata(package, "description"),
                            },
                        );
                    }
                }
            }
        }

        match get_crate_info(path) {
            Ok(crate_info) => {
                let mut res = Self {
                    state: DataState::default(),
                    screen: Screen::default(),
                };
                res.state.deps_map = meta_data;
                res.state.selected_package = vec![crate_info.clone()];
                res.state.level1_deps = match res.state.get_deps(crate_info) {
                    Ok(level1_deps) => level1_deps,
                    Err(err) => {
                        errors.push(err);
                        Vec::new()
                    }
                };
                res.select_first_row();
                (res, errors)
            }
            Err(e) => {
                errors.push(e);
                (Self::default(), errors)
            }
        }
    }

    fn select_first_row(&mut self) {
        self.state.selected_index = 0;
        self.state.get_level2_dep();
    }

    pub fn draw(&mut self, area: Rect, buf: &mut Buffer) {
        self.screen.display(area, buf, &self.state);
    }

    pub fn update(&mut self, key: KeyEvent) {
        match self.screen.mode {
            DisplayMode::View => {
                match key.code {
                    KeyCode::Left => {
                        if self.state.selected_package.len() > 1 {
                            self.screen.viewport_start = 0;
                            self.state.selected_package.pop();
                            self.state.level2_deps = Vec::new();
                            if let Some(last_dep) = self.state.selected_package.last() {
                                match self.state.get_deps(last_dep.clone()) {
                                    Ok(deps) => self.state.level1_deps = deps,
                                    Err(_) => self.state.level1_deps = Vec::new(),
                                }
                            } else {
                                self.state.level1_deps = Vec::new();
                            }
                            self.select_first_row();
                        }
                    }
                    KeyCode::Right => {
                        if self.state.level2_deps.len() > 0 {
                            self.screen.viewport_start = 0;
                            self.state
                                .selected_package
                                .push(self.state.level1_deps[self.state.selected_index].clone());
                            self.state.level1_deps = self.state.level2_deps.clone();
                            self.state.level2_deps = Vec::new();
                            self.select_first_row();
                        }
                    }
                    KeyCode::Up => {
                        if self.state.selected_index > 0 {
                            self.state.selected_index -= 1;
                            self.state.get_level2_dep();
                        }
                    }
                    KeyCode::Down => {
                        if self.state.selected_index < self.state.level1_deps.len() - 1 {
                            self.state.selected_index += 1;
                            self.state.get_level2_dep();
                        }
                    }
                    KeyCode::Enter => {
                        if let Some(dep) = self.state.level2_deps.get(self.state.selected_index) {
                            let metadata = self.state.get_metadata(dep);
                            if !metadata.documentation.is_empty() {
                                // Open documentation URL
                                if let Err(e) = open::that(&metadata.documentation) {
                                    eprintln!("Failed to open documentation: {}", e);
                                }
                            } else if let Ok(deps) = self.state.get_deps(dep.clone()) {
                                self.state.level2_deps = deps;
                                self.state.selected_index = 0;
                            }
                        }
                    }
                    KeyCode::Char('f' | 'F' | '/') => {
                        self.screen.mode = DisplayMode::Filter;
                    }
                    KeyCode::Char('v' | 'V') | KeyCode::Esc => {
                        self.screen.mode = DisplayMode::View;
                    }
                    KeyCode::Char('h' | 'H') => {
                        self.screen.mode = DisplayMode::Help;
                    }
                    _ => {}
                }
            }
            // Filter mode: Type in filter values
            DisplayMode::Filter => {
                match key.code {
                    // Escape or Enter: Back to main mode
                    KeyCode::Esc | KeyCode::Enter => {
                        self.screen.mode = DisplayMode::View;
                        self.screen.filter(&mut self.state);
                    }
                    // All other key events are passed on to the text area, then the filter is immediately applied
                    _ => {
                        // Else -> Pass on to the text area
                        self.screen.filter_area.input(key);
                        self.screen.filter(&mut self.state);
                    }
                };
            }
            DisplayMode::Help => {
                match key.code {
                    KeyCode::Esc => {
                        self.screen.mode = DisplayMode::View;
                    }
                    _ => {}
                };
            }
        };
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
        .unwrap_or_default();
    let name = name.split('"').nth(1).unwrap_or_default();
    let version = content
        .split('\n')
        .find(|line| line.starts_with("version = "))
        .unwrap_or_default();
    let version = version.split('"').nth(1).unwrap_or_default();
    Ok(Dependency {
        name: name.to_string(),
        version: version.to_string(),
    })
}

fn get_metadata(content: &Value, key: &str) -> String {
    content
        .get(key)
        .map(|v| v.as_str().unwrap_or_default())
        .unwrap_or_default()
        .to_string()
}
