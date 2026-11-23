use std::fs;
use std::path::{Path, PathBuf};

use super::failure::BrowserFailure;

const LOCK_NAMES: [&str; 3] = ["SingletonLock", "SingletonCookie", "SingletonSocket"];

#[must_use]
pub fn profile_directory(data_home: Option<&str>, home: &Path) -> PathBuf {
    let base = data_home.map_or_else(|| home.join(".local/share"), PathBuf::from);
    base.join("avatar/browser")
}

#[must_use]
pub fn locked_process_identifier(profile: &Path, hostname: &str) -> Option<i64> {
    let target = fs::read_link(profile.join("SingletonLock")).ok()?;
    let text = target.to_string_lossy().into_owned();
    let (host, identifier) = text.rsplit_once('-')?;
    let digits = !identifier.is_empty() && identifier.bytes().all(|byte| byte.is_ascii_digit());
    (host == hostname && digits)
        .then(|| identifier.parse().ok())
        .flatten()
}

pub(super) fn remove_stale_locks(
    profile: &Path,
    proc_root: &Path,
    hostname: &str,
) -> Result<(), BrowserFailure> {
    let Some(identifier) = locked_process_identifier(profile, hostname) else {
        return Ok(());
    };
    let cmdline = proc_root.join(identifier.to_string()).join("cmdline");
    match fs::read(cmdline) {
        Ok(bytes) if !bytes.is_empty() => Ok(()),
        Ok(_) => remove_lock_files(profile),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => remove_lock_files(profile),
        Err(error) => Err(error.into()),
    }
}

fn remove_lock_files(profile: &Path) -> Result<(), BrowserFailure> {
    for name in LOCK_NAMES {
        remove_if_present(&profile.join(name))?;
    }
    Ok(())
}

pub(super) fn remove_if_present(path: &Path) -> Result<(), BrowserFailure> {
    match fs::remove_file(path) {
        Err(error) if error.kind() != std::io::ErrorKind::NotFound => Err(error.into()),
        _ => Ok(()),
    }
}
