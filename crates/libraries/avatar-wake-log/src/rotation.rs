use std::fs;
use std::io;
use std::path::{Path, PathBuf};

fn numbered(path: &Path, index: u32) -> PathBuf {
    let mut name = path.as_os_str().to_owned();
    name.push(format!(".{index}"));
    PathBuf::from(name)
}

fn replace_by_rename(source: &Path, destination: &Path) -> io::Result<()> {
    if !source.exists() {
        return Ok(());
    }
    if destination.exists() {
        fs::remove_file(destination)?;
    }
    fs::rename(source, destination)
}

pub(crate) fn rotate_backups(path: &Path, backups: u32) -> io::Result<()> {
    if backups == 0 {
        return Ok(());
    }
    for index in (1..backups).rev() {
        replace_by_rename(&numbered(path, index), &numbered(path, index + 1))?;
    }
    replace_by_rename(path, &numbered(path, 1))
}
