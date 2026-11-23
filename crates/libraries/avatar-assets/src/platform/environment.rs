use std::collections::HashMap;
use std::ffi::OsString;
use std::hash::BuildHasher;

pub trait Environment {
    fn variable(&self, name: &str) -> Option<OsString>;
}

pub struct ProcessEnvironment;

impl Environment for ProcessEnvironment {
    fn variable(&self, name: &str) -> Option<OsString> {
        std::env::var_os(name)
    }
}

impl<Hasher: BuildHasher> Environment for HashMap<String, OsString, Hasher> {
    fn variable(&self, name: &str) -> Option<OsString> {
        self.get(name).cloned()
    }
}
