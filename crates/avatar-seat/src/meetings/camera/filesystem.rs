use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use super::CameraStore;
use crate::edges::EdgeFailure;

pub struct FileSystemStore;

fn failure(error: &std::io::Error) -> EdgeFailure {
    EdgeFailure::Os(error.to_string())
}

impl CameraStore for FileSystemStore {
    fn exists(&self, path: &str) -> bool {
        Path::new(path).exists()
    }

    fn make_directory(&self, path: &str) -> Result<(), EdgeFailure> {
        fs::create_dir_all(path).map_err(|error| failure(&error))
    }

    fn write_text(&self, path: &str, text: &str) -> Result<(), EdgeFailure> {
        fs::write(path, text).map_err(|error| failure(&error))
    }

    fn read_text(&self, path: &str) -> Option<String> {
        fs::read_to_string(path).ok()
    }

    fn remove(&self, path: &str) -> Result<(), EdgeFailure> {
        match fs::remove_file(path) {
            Err(error) if error.kind() != ErrorKind::NotFound => Err(failure(&error)),
            _ => Ok(()),
        }
    }
}
