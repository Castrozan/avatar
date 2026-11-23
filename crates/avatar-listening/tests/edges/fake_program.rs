use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

pub(crate) fn fake_program(directory: &Path, name: &str, body: &str) -> io::Result<PathBuf> {
    let path = directory.join(name);
    std::fs::write(&path, format!("#!/bin/sh\n{body}\n"))?;
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755))?;
    Ok(path)
}
