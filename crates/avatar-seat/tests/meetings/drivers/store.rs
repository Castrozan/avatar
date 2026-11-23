use std::cell::RefCell;
use std::collections::BTreeSet;
use std::path::Path;

use avatar_seat::edges::EdgeFailure;
use avatar_seat::meetings::drivers::DriverStore;

pub(crate) struct NamedStore(RefCell<BTreeSet<String>>);

fn name_of(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default()
}

impl NamedStore {
    pub(crate) fn new(installed: &[String]) -> Self {
        Self(RefCell::new(installed.iter().cloned().collect()))
    }

    pub(crate) fn names(&self) -> Vec<String> {
        self.0.borrow().iter().cloned().collect()
    }
}

impl DriverStore for NamedStore {
    fn exists(&self, path: &Path) -> bool {
        self.0.borrow().contains(&name_of(path))
    }

    fn replace(&self, _packaged: &Path, installed: &Path) -> Result<(), EdgeFailure> {
        self.0.borrow_mut().insert(name_of(installed));
        Ok(())
    }
}
