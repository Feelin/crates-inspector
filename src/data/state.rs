use std::cmp::Ordering;
use crate::ui::OrderBy;
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct Metadata {
    pub name: String,
    pub version: String,
    pub license: String,
    pub size: u64,
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
    pub sorting_asc: bool,
    order: OrderBy
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
            filter: String::new(),
            sorting_asc: true,
            order: OrderBy::Name
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