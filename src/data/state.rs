use std::collections::HashMap;


#[derive(Debug, Clone, Default)]
pub struct Metadata {
    pub id: String,
    pub name: String,
    pub version: String,
    pub manifest_path: String,
    pub license: String,
    // size: u64,
    pub documentation: String,
    pub description: String,
    pub dependencies: Vec<String>,
    // pub depth: u8
}


pub struct DataState {
    pub selected_index: usize,
    pub deps_map: HashMap<String, Metadata>,
    pub level1_deps: Vec<Metadata>,
    pub level2_deps: Vec<Metadata>,
    pub selected_package: Vec<Metadata>,
    pub filter: String,
}

impl DataState {
    pub fn get_filter_deps(&self) -> Vec<Metadata> {
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
            .get_deps(&self.get_filter_deps()[self.selected_index].clone())
    }

    pub fn get_deps(&mut self, parent: &Metadata) -> Vec<Metadata> {
        parent.dependencies.iter()
            .map(|id| self.get_metadata(String::from(id)))
            .collect()
    }

    pub fn get_metadata(&self, id: String) -> Metadata {
        self.deps_map
            .get(id.as_str())
            .cloned()
            .unwrap_or(Metadata::default())
    }
}
