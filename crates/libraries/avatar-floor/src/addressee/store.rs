use super::lock::exclusive;
use super::record::Addressee;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;

const ADDRESSEE_FILE: &str = "addressee.json";
const STAGED_FILE: &str = ".addressee.json";

pub fn current(directory: &Path) -> io::Result<Option<Addressee>> {
    match fs::read_to_string(directory.join(ADDRESSEE_FILE)) {
        Ok(text) => Ok(Some(serde_json::from_str(&text)?)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

pub fn write(directory: &Path, addressee: &Addressee) -> io::Result<()> {
    let staged = directory.join(STAGED_FILE);
    let mut output = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(&staged)?;
    output.write_all(&serde_json::to_vec(addressee)?)?;
    fs::rename(staged, directory.join(ADDRESSEE_FILE))
}

pub fn hand_to(directory: &Path, addressee: &Addressee) -> io::Result<()> {
    let _lock = exclusive(directory)?;
    write(directory, addressee)
}

pub fn forget(directory: &Path) -> io::Result<()> {
    let _lock = exclusive(directory)?;
    match fs::remove_file(directory.join(ADDRESSEE_FILE)) {
        Err(error) if error.kind() != io::ErrorKind::NotFound => Err(error),
        _ => Ok(()),
    }
}

pub fn addresses(directory: &Path, pane: Option<&str>) -> io::Result<bool> {
    let holder = current(directory)?;
    Ok(holder.is_none_or(|held| pane.is_none_or(|asked| held.pane == asked)))
}
