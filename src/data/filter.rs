
#[derive(Debug, Default, Clone)]
pub struct Filter {
    pub title: String,
}

impl Filter {
    pub fn new(filter_string: &str, any: bool) -> Self {
        let mut title = String::new();
        Self {
            title
        }
    }
}
