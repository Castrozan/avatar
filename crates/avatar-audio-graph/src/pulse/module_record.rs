use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModuleRecord {
    pub index: u32,
    pub name: String,
    pub argument: String,
}

impl ModuleRecord {
    #[must_use]
    pub fn new(index: u32, name: &str, argument: String) -> Self {
        Self {
            index,
            name: name.to_owned(),
            argument,
        }
    }

    #[must_use]
    pub fn is_listed_in(&self, listed: &[Self]) -> bool {
        listed.contains(self)
    }
}
