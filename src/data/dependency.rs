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

