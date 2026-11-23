use std::error::Error;
use std::fs;
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, PoisonError};

pub(crate) type TestResult = Result<(), Box<dyn Error>>;

pub(crate) fn executable(directory: &Path, name: &str, body: &str) -> io::Result<PathBuf> {
    let path = directory.join(name);
    fs::write(&path, format!("#!/bin/sh\n{body}\n"))?;
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755))?;
    Ok(path)
}

pub(crate) fn logged(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_default()
}

#[derive(Clone, Default)]
pub(crate) struct SharedOutput(Arc<Mutex<Vec<u8>>>);

impl SharedOutput {
    pub(crate) fn bytes(&self) -> Vec<u8> {
        self.0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
}

impl Write for SharedOutput {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let mut held = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        held.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
