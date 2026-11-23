use std::fs;
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use serde_json::Value;

pub(crate) fn write_executable(path: &Path, body: &str) -> io::Result<()> {
    fs::write(path, format!("#!/bin/sh\n{body}\n"))?;
    fs::set_permissions(path, fs::Permissions::from_mode(0o755))
}

pub(crate) fn fake_avatar_audio(directory: &Path, status: &Value) -> io::Result<PathBuf> {
    let status_path = directory.join("status.json");
    fs::write(&status_path, status.to_string())?;
    let program = directory.join("avatar-audio");
    let body = format!(
        "printf '%s\\n' \"$*\" >> {log}\nif [ \"$1\" = \"status\" ]; then cat {status}; fi",
        log = directory.join("calls.log").display(),
        status = status_path.display(),
    );
    write_executable(&program, &body)?;
    Ok(program)
}
