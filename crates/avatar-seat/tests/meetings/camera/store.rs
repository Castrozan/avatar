use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};

use avatar_seat::edges::EdgeFailure;
use avatar_seat::meetings::camera::{CameraConfig, CameraStore};

pub(crate) const HOME: &str = "<home>";
pub(crate) const APPLICATION: &str = "/nonexistent/OBS.app";
pub(crate) const REPORT_DIRECTORY: &str = "<home>/.local/share/avatar";
pub(crate) const REPORT: &str = "<home>/.local/share/avatar/virtual-camera-report";

pub(crate) struct MemoryStore {
    home: String,
    files: RefCell<BTreeMap<String, String>>,
    directories: RefCell<BTreeSet<String>>,
}

pub(crate) fn config_for(home: &str) -> CameraConfig {
    CameraConfig {
        home: home.to_owned(),
        application: APPLICATION.to_owned(),
        start_script: format!("{home}/start_virtual_camera.lua"),
    }
}

fn ancestors(path: &str) -> Vec<String> {
    let parts: Vec<&str> = path.split('/').collect();
    (2..=parts.len())
        .map(|count| parts[..count].join("/"))
        .collect()
}

impl MemoryStore {
    pub(crate) fn new(home: &str) -> Self {
        Self {
            home: home.to_owned(),
            files: RefCell::default(),
            directories: RefCell::default(),
        }
    }

    pub(crate) fn file(&self, path: &str) -> Option<String> {
        self.files.borrow().get(path).cloned()
    }

    pub(crate) fn has_directory(&self, path: &str) -> bool {
        self.directories.borrow().contains(path)
    }

    pub(crate) fn relative_directories(&self) -> Vec<String> {
        let inside = self.directories.borrow().clone();
        let prefix = format!("{}/", self.home);
        let relative = inside
            .iter()
            .filter_map(|path| path.strip_prefix(prefix.as_str()));
        relative.map(str::to_owned).collect()
    }

    pub(crate) fn publish_report(&self, text: &str) {
        self.make_directory(REPORT_DIRECTORY).unwrap_or_default();
        self.write_text(REPORT, text).unwrap_or_default();
    }

    pub(crate) fn install_application(&self) {
        self.directories.borrow_mut().insert(APPLICATION.to_owned());
    }
}

impl CameraStore for MemoryStore {
    fn exists(&self, path: &str) -> bool {
        self.files.borrow().contains_key(path) || self.has_directory(path)
    }

    fn make_directory(&self, path: &str) -> Result<(), EdgeFailure> {
        self.directories.borrow_mut().extend(ancestors(path));
        Ok(())
    }

    fn write_text(&self, path: &str, text: &str) -> Result<(), EdgeFailure> {
        self.files
            .borrow_mut()
            .insert(path.to_owned(), text.to_owned());
        Ok(())
    }

    fn read_text(&self, path: &str) -> Option<String> {
        self.file(path)
    }

    fn remove(&self, path: &str) -> Result<(), EdgeFailure> {
        self.files.borrow_mut().remove(path);
        Ok(())
    }
}
