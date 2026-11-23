use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub(super) fn roll_over_if_full(path: &Path, backup_count: u32, full: bool) -> io::Result<()> {
    if backup_count == 0 || !full {
        return Ok(());
    }
    for generation in (1..backup_count).rev() {
        move_over(&numbered(path, generation), &numbered(path, generation + 1))?;
    }
    move_over(path, &numbered(path, 1))
}

fn numbered(path: &Path, generation: u32) -> PathBuf {
    let mut name = path.as_os_str().to_owned();
    name.push(format!(".{generation}"));
    PathBuf::from(name)
}

fn move_over(source: &Path, destination: &Path) -> io::Result<()> {
    if !source.exists() {
        return Ok(());
    }
    if destination.exists() {
        fs::remove_file(destination)?;
    }
    fs::rename(source, destination)
}
