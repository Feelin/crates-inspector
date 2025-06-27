use crate::data::{DataState, Metadata};
use crate::error;
use crate::ui::{DisplayMode, OrderBy, Screen};

use open;
use ratatui::crossterm::event::{KeyCode, KeyEvent};
use ratatui::prelude::*;
use serde_json::Value;
use std::collections::HashMap;

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
        let mut deps_map = HashMap::new();

        if let Some(Value::Array(nodes)) = metadata
            .get("resolve")
            .and_then(|resolve| resolve.get("nodes")) {
            if let Some(Value::Array(packages)) = metadata.get("packages") {
                for package in packages {
                    if let Some(name) = package.get("name") {
                        if let Some(version) = package.get("version") {
                            let id = get_string_from(package, "id");
                            if deps_map.get(id.as_str()).is_some() {
                                continue;
                            }
                            
                            deps_map.insert(
                                id.clone(),
                                Metadata {
                                    size: get_size_by_manifest_path(get_string_from(package, "manifest_path")).unwrap_or(0),
                                    name: trim_value(name),
                                    version: trim_value(version),
                                    license: get_string_from(package, "license"),
                                    documentation: get_string_from(package, "documentation"),
                                    description: get_string_from(package, "description"),
                                    dependencies: nodes
                                        .iter()
                                        .find(|node| get_string_from(node, "id") == id)
                                        .map(|node| get_vec_from(node, "dependencies"))
                                        .unwrap_or_default(),
                                },
                            );
                        }
                    }
                }
            }
        }



        // get root from workspace_default_members
        let root_id = metadata
            .get("workspace_default_members")
            .and_then(|members| members.get(0))
            .and_then(|member| member.as_str())
            .unwrap_or_default()
            .to_string();
        
        let mut res = Self {
            state: DataState::default(),
            screen: Screen::default(),
        };
        res.state.deps_map = deps_map;
        let root = res.state.get_metadata(root_id);
        res.state.selected_package = vec![root.clone()];
        error!("root: {}", root.dependencies.join(","));
        res.state.level1_deps = res.state.get_deps(&root);
        res.select_first_row();
        (res, errors)
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
                                self.state.level1_deps = self.state.get_deps(&last_dep.clone())
                            } else {
                                self.state.level1_deps = Vec::new();
                            }
                            self.select_first_row();
                        }
                    }
                    KeyCode::Right | KeyCode::Char('l' | 'L') => {
                        if self.state.level2_deps.len() > 0 {
                            self.screen.viewport_start = 0;
                            self.state.selected_package.push(
                                self.state.get_filter_deps()[self.state.selected_index].clone(),
                            );
                            self.state.level1_deps = self.state.level2_deps.clone();
                            self.state.level2_deps = Vec::new();
                            self.select_first_row();
                        }
                    }
                    KeyCode::Up | KeyCode::Char('K' | 'k') => {
                        if self.state.selected_index > 0 {
                            self.state.selected_index -= 1;
                            self.state.get_level2_dep();
                        }
                    }
                    KeyCode::Down | KeyCode::Char('J' | 'j') => {
                        if self.state.selected_index < self.state.get_filter_deps().len() - 1 {
                            self.state.selected_index += 1;
                            self.state.get_level2_dep();
                        }
                    }
                    KeyCode::Enter => {
                        if let Some(metadata) =
                            self.state.level1_deps.get(self.state.selected_index)
                        {
                            if !metadata.documentation.is_empty() {
                                // Open documentation URL
                                if let Err(e) = open::that(&metadata.documentation) {
                                    eprintln!("Failed to open documentation: {}", e);
                                }
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
                    KeyCode::Char('s' | 'S') => {
                        self.screen.mode = DisplayMode::Sort;
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
                    KeyCode::Esc | KeyCode::Char('c' | 'C') => {
                        self.screen.mode = DisplayMode::View;
                    }
                    _ => {}
                };
            }
            DisplayMode::Sort => match key.code {
                KeyCode::Char('n' | 'N') => {
                    self.state.order_by(OrderBy::Name);
                    self.screen.mode = DisplayMode::View;
                }
                KeyCode::Char('s' | 'S') => {
                    self.state.order_by(OrderBy::Size);
                    self.screen.mode = DisplayMode::View;
                }
                KeyCode::Char('v' | 'V') => {
                    self.state.order_by(OrderBy::Version);
                    self.screen.mode = DisplayMode::View;
                }
                KeyCode::Char('r' | 'R') => {
                    self.state.sorting(!self.state.sorting_asc);
                    self.screen.mode = DisplayMode::View;
                }
                KeyCode::Esc => {
                    self.screen.mode = DisplayMode::View;
                }
                _ => {}
            },
        };
    }
}

fn get_vec_from(content: &Value, key: &str) -> Vec<String> {
    content
        .get(key)
        .and_then(|v| match v {
            Value::Array(arr) => {
                let strings: Vec<String> = arr
                    .iter()
                    .filter_map(|item| item.as_str().map(|s| s.to_string()))
                    .collect();
                Some(strings)
            }
            _ => None,
        })
        .unwrap_or(Vec::new())
}

fn get_string_from(content: &Value, key: &str) -> String {
    content
        .get(key)
        .map(|v| v.as_str().unwrap_or_default())
        .unwrap_or_default()
        .to_string()
}

fn trim_value(value: &Value) -> String {
    value.to_string().trim()
        .trim_matches(|c: char| c.is_whitespace() || c == '"').to_string()
}

pub fn get_size_by_manifest_path(path: String) -> Result<u64, std::io::Error> {
    // "/Users/yulin.fyl/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rust-argon2-0.8.3/Cargo.toml"
    let crate_path = path
        .replace("/Cargo.toml", ".crate")
        .replace("/src/", "/cache/");
    
    let metadata = std::fs::metadata(&crate_path)?;
    Ok(metadata.len())
}