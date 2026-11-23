use std::path::{Path, PathBuf};

#[must_use]
pub fn wake_log_path(state_home: Option<&str>, home: &Path) -> PathBuf {
    let base = match state_home {
        Some(directory) if !directory.is_empty() => PathBuf::from(directory),
        _ => home.join(".local").join("state"),
    };
    base.join("avatar").join("wake.log")
}
