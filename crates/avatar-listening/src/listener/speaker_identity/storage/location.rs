use std::path::{Path, PathBuf};

#[must_use]
pub fn voices_directory(state_home: Option<&str>, home: &Path) -> PathBuf {
    let base = match state_home {
        Some(directory) if !directory.is_empty() => PathBuf::from(directory),
        _ => home.join(".local").join("state"),
    };
    base.join("avatar").join("voices")
}
