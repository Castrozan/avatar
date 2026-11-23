use super::json::claim_record;
use super::location::{claim_path, claims_directory};
use super::refusal::refused;
use super::site::Site;
use std::fs::{DirBuilder, File, OpenOptions, TryLockError};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::Path;

pub fn claim(site: &Site<'_>, pane: &str, room: &str, now: f64) -> io::Result<bool> {
    let Some(mut handle) = try_exclusive(open_claim_file(site.directory, pane)?)? else {
        return Ok(false);
    };
    let recorded = read_recorded(&mut handle)?;
    if refused(site, pane, room, now, &recorded)? {
        return Ok(false);
    }
    write_claim(&mut handle, &claim_record(room, now))?;
    Ok(true)
}

fn open_claim_file(directory: &Path, pane: &str) -> io::Result<File> {
    DirBuilder::new()
        .recursive(true)
        .create(claims_directory(directory))?;
    OpenOptions::new()
        .append(true)
        .read(true)
        .create(true)
        .open(claim_path(directory, pane))
}

fn try_exclusive(file: File) -> io::Result<Option<File>> {
    match file.try_lock() {
        Ok(()) => Ok(Some(file)),
        Err(TryLockError::WouldBlock) => Ok(None),
        Err(TryLockError::Error(error)) => Err(error),
    }
}

fn read_recorded(handle: &mut File) -> io::Result<String> {
    handle.seek(SeekFrom::Start(0))?;
    let mut recorded = String::new();
    handle.read_to_string(&mut recorded)?;
    Ok(recorded)
}

fn write_claim(handle: &mut File, record: &str) -> io::Result<()> {
    handle.set_len(0)?;
    handle.write_all(record.as_bytes())
}
