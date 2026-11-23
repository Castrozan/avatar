use std::fs::{self, File};
use std::io;
use std::path::{Component, Path, PathBuf};

use super::fetch::fetch_verified;
use super::{Outcome, Transport, archive};
use crate::integrity::Integrity;
use crate::manifest::{ArchiveKind, Download};
use crate::platform::layout::DataDirectory;

const ARCHIVE_MARKER: &str = ".avatar-integrity";

pub fn install_one(
    download: &Download,
    data: &DataDirectory,
    transport: &dyn Transport,
) -> io::Result<Outcome> {
    let integrity = Integrity::parse(&download.integrity)?;
    let target = target_of(download, data)?;
    if is_present(download, &integrity, &target)? {
        return Ok(Outcome::AlreadyPresent);
    }
    replace(download, &integrity, &target, transport)?;
    Ok(Outcome::Installed)
}

fn replace(
    download: &Download,
    integrity: &Integrity,
    target: &Path,
    transport: &dyn Transport,
) -> io::Result<()> {
    let part = partial_path(target);
    let fetched = fetch_verified(download, integrity, &part, transport);
    let placed = fetched.and_then(|()| place(download, &part, target));
    fs::remove_file(&part).ok();
    placed
}

fn target_of(download: &Download, data: &DataDirectory) -> io::Result<PathBuf> {
    if stays_inside(Path::new(&download.destination)) {
        return Ok(data.join(&download.destination));
    }
    Err(io::Error::other(format!(
        "{} leaves the data directory",
        download.name
    )))
}

fn stays_inside(relative: &Path) -> bool {
    relative
        .components()
        .all(|part| matches!(part, Component::Normal(_)))
}

fn partial_path(target: &Path) -> PathBuf {
    let mut name = target.as_os_str().to_owned();
    name.push(".partial");
    PathBuf::from(name)
}

fn is_present(download: &Download, integrity: &Integrity, target: &Path) -> io::Result<bool> {
    match download.archive {
        Some(_) => Ok(fs::read_to_string(target.join(ARCHIVE_MARKER))
            .is_ok_and(|marker| marker == download.integrity)),
        None => file_matches(integrity, target),
    }
}

fn file_matches(integrity: &Integrity, target: &Path) -> io::Result<bool> {
    let Ok(mut file) = File::open(target) else {
        return Ok(false);
    };
    let mut verifier = integrity.verifier(io::sink());
    io::copy(&mut file, &mut verifier)?;
    Ok(integrity.matches(&verifier.finish()))
}

fn place(download: &Download, part: &Path, target: &Path) -> io::Result<()> {
    match download.archive {
        Some(ArchiveKind::TarGzip) => {
            archive::unpack_tar_gzip(part, target)?;
            fs::write(target.join(ARCHIVE_MARKER), &download.integrity)
        }
        None => fs::rename(part, target),
    }
}
