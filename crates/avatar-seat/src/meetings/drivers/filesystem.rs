use std::fs;
use std::io;
use std::os::unix::fs::symlink;
use std::path::Path;

use super::DriverStore;
use crate::edges::EdgeFailure;

pub struct FileSystemDrivers;

fn copy_entry(source: &Path, target: &Path) -> io::Result<()> {
    let kind = fs::symlink_metadata(source)?.file_type();
    if kind.is_symlink() {
        return symlink(fs::read_link(source)?, target);
    }
    if kind.is_dir() {
        return copy_tree(source, target);
    }
    fs::copy(source, target).map(drop)
}

fn copy_tree(source: &Path, target: &Path) -> io::Result<()> {
    fs::create_dir_all(target)?;
    for entry in fs::read_dir(source)? {
        let name = entry?.file_name();
        copy_entry(&source.join(&name), &target.join(&name))?;
    }
    fs::set_permissions(target, fs::metadata(source)?.permissions())
}

fn replace_tree(packaged: &Path, installed: &Path) -> io::Result<()> {
    fs::remove_dir_all(installed).unwrap_or_default();
    if let Some(parent) = installed.parent() {
        fs::create_dir_all(parent)?;
    }
    copy_tree(packaged, installed)
}

impl DriverStore for FileSystemDrivers {
    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn replace(&self, packaged: &Path, installed: &Path) -> Result<(), EdgeFailure> {
        replace_tree(packaged, installed).map_err(|error| EdgeFailure::Os(error.to_string()))
    }
}
