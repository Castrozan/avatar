use std::fs::{self, File};
use std::io;
use std::path::{Component, Path, PathBuf};

use flate2::read::GzDecoder;
use tar::{Archive, Entry};

use crate::platform::files::create_parent;

pub(super) fn unpack_tar_gzip(part: &Path, target: &Path) -> io::Result<()> {
    fresh_directory(target)?;
    let mut archive = Archive::new(GzDecoder::new(File::open(part)?));
    for entry in archive.entries()? {
        unpack_entry(&mut entry?, target)?;
    }
    Ok(())
}

fn fresh_directory(target: &Path) -> io::Result<()> {
    match fs::remove_dir_all(target) {
        Err(error) if error.kind() != io::ErrorKind::NotFound => Err(error),
        _ => fs::create_dir_all(target),
    }
}

fn unpack_entry(entry: &mut Entry<'_, impl io::Read>, target: &Path) -> io::Result<()> {
    if let Some(destination) = destination_of(entry, target)? {
        create_parent(&destination)?;
        entry.unpack(destination)?;
    }
    Ok(())
}

fn destination_of(entry: &Entry<'_, impl io::Read>, target: &Path) -> io::Result<Option<PathBuf>> {
    let kind = entry.header().entry_type();
    let inside = inside_of(&entry.path()?)?;
    let wanted = (kind.is_file() || kind.is_dir()) && !inside.as_os_str().is_empty();
    Ok(wanted.then(|| target.join(inside)))
}

fn inside_of(path: &Path) -> io::Result<PathBuf> {
    path.components()
        .skip(1)
        .filter(|part| *part != Component::CurDir)
        .map(|part| normal(part).ok_or_else(|| unsafe_path(path)))
        .collect()
}

fn normal(part: Component<'_>) -> Option<&std::ffi::OsStr> {
    match part {
        Component::Normal(name) => Some(name),
        _ => None,
    }
}

fn unsafe_path(path: &Path) -> io::Error {
    io::Error::other(format!("unsafe archive path {}", path.display()))
}
