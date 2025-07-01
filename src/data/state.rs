use crate::ui::OrderBy;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
// use log::error;

#[derive(Debug, Clone, Default)]
pub struct Metadata {
    pub name: String,
    pub version: String,
    pub license: String,
    pub size: u64,
    pub documentation: String,
    pub description: String,
    pub dependencies: Vec<String>,
}


pub struct DataState {
    pub selected_index: usize,
    pub deps_map: HashMap<String, Metadata>,
    pub level1_deps: Vec<Metadata>,
    pub level2_deps: Vec<Metadata>,
    pub selected_package: Vec<Metadata>,
    pub filter_input: String,
    pub sorting_asc: bool,
    pub is_direct: bool,
    order: OrderBy
}

impl DataState {
    pub fn get_filter_deps(&self) -> Vec<Metadata> {
        self.level1_deps.iter()
            .filter(|x| self.filter_input.is_empty() || x.name.contains(self.filter_input.as_str()))
            .cloned()
            .collect()
    }
    
    pub fn get_selected_dep(&self) -> Metadata {
        let filtered_deps = self.get_filter_deps();
        if filtered_deps.len() > self.selected_index {
            filtered_deps[self.selected_index].clone()
        } else { 
            Metadata::default()
        }
    }

    pub fn default() -> DataState {
        DataState {
            selected_index: 0,
            deps_map: HashMap::new(),
            level1_deps: Vec::new(),
            level2_deps: Vec::new(),
            selected_package: Vec::new(),
            filter_input: String::new(),
            is_direct: true,
            sorting_asc: false,
            order: OrderBy::Size
        }
    }



    pub fn get_level2_dep(&mut self) {
        self.level2_deps = self
            .get_deps(self.get_selected_dep())
    }
    pub fn get_deps(&mut self, parent: Metadata) -> Vec<Metadata> {
        let mut dependency_ids = parent.dependencies.clone();
        if !self.is_direct {
            dependency_ids = self.get_deps_ids(parent, HashSet::new()).into_iter().collect();
        }
        let mut res = dependency_ids.iter()
            .map(|id| self.get_metadata(String::from(id)))
            .collect();
        sorting_impl(&mut res, self.order, self.sorting_asc);
        res
    }
    
    fn get_deps_ids(&mut self, parent: Metadata, mut visited: HashSet<String>) -> HashSet<String> {
        for id in parent.dependencies.iter() {
            if !visited.contains(id) {
                visited.insert(String::from(id));
                let target = self.get_metadata(String::from(id));
                visited = self.get_deps_ids(target, visited.clone());
            }
        }
        visited
    }

    pub fn get_metadata(&self, id: String) -> Metadata {
        self.deps_map
            .get(id.as_str())
            .cloned()
            .unwrap_or(Metadata::default())
    }

    pub fn order_by(&mut self, order: OrderBy) {
        self.order = order;
        self.sorting(self.sorting_asc);
    }

    pub fn sorting(&mut self, sorting_asc: bool) {
        self.sorting_asc = sorting_asc;
        sorting_impl(&mut self.level1_deps, self.order, sorting_asc);
        sorting_impl(&mut self.level2_deps, self.order, sorting_asc);
        self.selected_index = self.get_filter_deps().len() - 1 - self.selected_index;
    }

    pub fn switch_mode(&mut self) {
        if let Some(last_dep) = self.selected_package.last() {
            self.selected_index = 0;
            self.level1_deps = self.get_deps(last_dep.clone());
            self.get_level2_dep();
        } else {
            self.level1_deps = Vec::new();
        }
    }
}

fn sorting_impl(vec: &mut Vec<Metadata>, order: OrderBy, sorting_asc: bool) {
    vec.sort_by(|left, right| {
        let compare = |a: &Metadata, b: &Metadata| -> Ordering {
            match order {
                OrderBy::Name => a.name.cmp(&b.name),
                OrderBy::Version => a.version.cmp(&b.version),
                OrderBy::Size => a.size.cmp(&b.size),
            }
        };

        if sorting_asc {
            compare(left, right)
        } else {
            compare(right, left)
        }
    });
}
