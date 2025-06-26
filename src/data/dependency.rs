use crate::app::Metadata;
use crate::error;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Dependency {
    pub name: String,
    pub version: String,
}

impl Dependency {
    pub fn get_key(dep: &Dependency) -> String {
        format!("{}-{}", dep.name, dep.version).replace('"', "")
    }
}


pub struct DataState {
    pub selected_index: usize,
    pub deps_map: HashMap<String, Metadata>,
    pub level1_deps: Vec<Dependency>,
    pub level2_deps: Vec<Dependency>,
    pub selected_package: Vec<Dependency>,
    pub filter: String,
}

impl DataState {
    pub fn get_filter_deps(&self) -> Vec<Dependency> {
        self.level1_deps.iter()
            .filter(|x| self.filter.is_empty() || x.name.contains(self.filter.as_str()))
            .cloned()
            .collect()
    }

    pub fn default() -> DataState {
        DataState {
            selected_index: 0,
            deps_map: HashMap::new(),
            level1_deps: Vec::new(),
            level2_deps: Vec::new(),
            selected_package: Vec::new(),
            filter: String::new()
        }
    }



    pub fn get_level2_dep(&mut self) {
        self.level2_deps = self
            .get_deps(self.get_filter_deps()[self.selected_index].clone())
            .unwrap_or_else(|_| Vec::new());
    }

    pub fn get_deps(&mut self, dep: Dependency) -> Result<Vec<Dependency>, error::Errors> {
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

    pub fn get_metadata(&self, dep: &Dependency) -> Metadata {
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
